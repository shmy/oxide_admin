use std::time::Duration;

use bon::Builder;
use domain::iam::{
    error::IamError,
    value_object::{role_id::RoleId, user_id::UserId},
};
use infrastructure::{shared::cloneable_error::CloneableError, shared::pool::Pool};
use nject::injectable;
use serde::Deserialize;
use serde_with::{NoneAsEmptyString, serde_as};
use single_flight_derive::single_flight;

use crate::{
    iam::dto::user::UserDto,
    impl_static_cache,
    shared::{
        cache_type::{CacheType, hash_encode},
        dto::OptionDto,
        paging_query::PagingQuery,
        paging_result::PagingResult,
    },
};

const CACHE_CAPACITY: u64 = 100;
const CACHE_TTL: Duration = Duration::from_secs(15 * 60);

impl_static_cache!(
    SEARCH_CACHE,
    PagingResult<UserDto>,
    CACHE_CAPACITY,
    CACHE_TTL
);

#[derive(Clone)]
#[injectable]
pub struct UserService {
    pool: Pool,
    #[inject(&SEARCH_CACHE)]
    cache: &'static CacheType<PagingResult<UserDto>>,
}

impl UserService {
    pub fn clean_cache(&self) {
        self.cache.invalidate_all();
    }

    pub async fn search_cached(
        &self,
        query: SearchUsersQuery,
    ) -> Result<PagingResult<UserDto>, CloneableError> {
        let key = hash_encode(&query);
        self.cache.get_with(key, self.search(query)).await
    }

    #[single_flight]
    pub async fn search(
        &self,
        query: SearchUsersQuery,
    ) -> Result<PagingResult<UserDto>, CloneableError> {
        let total_future = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM _users
            WHERE ($1::text IS NULL OR account LIKE CONCAT('%', $1, '%'))
                AND ($2::text IS NULL OR name LIKE CONCAT('%', $2, '%'))
                AND ($3::boolean IS NULL OR privileged = $3)
                AND ($4::boolean IS NULL OR enabled = $4)
                AND ($5::text IS NULL OR $5 = ANY(role_ids))
            "#,
            query.account,
            query.name,
            query.privileged,
            query.enabled,
            query.role_id,
        )
        .fetch_one(&self.pool);
        let page = query.paging.page();
        let page_size = query.paging.page_size();
        let offset = (page - 1) * page_size;
        let rows_future = sqlx::query_as!(
            UserDto,
            r#"
        SELECT
            u.id as id,
            u.account as account,
            u.portrait as portrait,
            u.name as name,
            u.role_ids as "role_ids: Vec<RoleId>",
            u.privileged as privileged,
            u.enabled as enabled,
            u.created_at as created_at,
            u.updated_at as updated_at,
            COALESCE(array_agg(r.name) FILTER (WHERE r.name IS NOT NULL), '{}') as "role_names!: Vec<String>"
        FROM _users as u
        LEFT JOIN _roles as r ON r.id = ANY(u.role_ids)
        WHERE ($1::text IS NULL OR u.account LIKE CONCAT('%', $1, '%'))
            AND ($2::text IS NULL OR u.name LIKE CONCAT('%', $2, '%'))
            AND ($3::boolean IS NULL OR u.privileged = $3)
            AND ($4::boolean IS NULL OR u.enabled = $4)
            AND ($5::text IS NULL OR $5 = ANY(u.role_ids))
        GROUP BY u.id
        ORDER BY u.updated_at DESC
        LIMIT $6 OFFSET $7

        "#,
            query.account,
            query.name,
            query.privileged,
            query.enabled,
            query.role_id,
            page_size,
            offset,
        )
        .fetch_all(&self.pool);
        let (total, rows) = tokio::try_join!(total_future, rows_future)?;
        Ok(PagingResult { total, items: rows })
    }

    #[single_flight]
    pub async fn retrieve(&self, id: UserId) -> Result<UserDto, CloneableError> {
        let row_opt = sqlx::query_as!(
            UserDto,
            r#"
        SELECT
            u.id as id,
            u.account as account,
            u.portrait as portrait,
            u.name as name,
            u.role_ids as "role_ids: Vec<RoleId>",
            u.privileged as privileged,
            u.enabled as enabled,
            u.created_at as created_at,
            u.updated_at as updated_at,
            COALESCE(array_agg(r.name) FILTER (WHERE r.name IS NOT NULL), '{}') as "role_names!: Vec<String>"
        FROM _users as u
        LEFT JOIN _roles as r ON r.id = ANY(u.role_ids)
        WHERE u.id = $1
        GROUP BY u.id
        LIMIT 1
        "#,
            &id,
        )
        .fetch_optional(&self.pool)
        .await?;
        row_opt.ok_or(CloneableError::from(anyhow::anyhow!(
            IamError::UserNotFound
        )))
    }

    #[single_flight]
    pub async fn list_options(&self) -> Result<Vec<OptionDto>, CloneableError> {
        let options = sqlx::query_as!(
            OptionDto::<String>,
            r#"
        SELECT name as label, id as value FROM _users ORDER BY updated_at DESC
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(options)
    }
}

#[serde_as]
#[derive(Clone, PartialEq, Eq, Hash, Deserialize, Builder)]
pub struct SearchUsersQuery {
    #[serde(flatten)]
    paging: PagingQuery,
    account: Option<String>,
    name: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    privileged: Option<bool>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    enabled: Option<bool>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    role_id: Option<String>,
}
