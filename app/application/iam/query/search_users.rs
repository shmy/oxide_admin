use std::time::Duration;

use bon::Builder;
use domain::iam::{error::IamError, value_object::role_id::RoleId};
use infrastructure::shared::pg_pool::PgPool;
use kvdb_kit::Kvdb;
use nject::injectable;
use serde::Deserialize;
use serde_with::{NoneAsEmptyString, serde_as};
use single_flight::single_flight;
use utoipa::IntoParams;

use crate::{
    error::ApplicationResult,
    iam::dto::user::UserDto,
    shared::{
        cache_provider::CacheProvider, paging_query::PagingQuery, paging_result::PagingResult,
        query_handler::QueryHandler,
    },
};

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, IntoParams, Builder)]
pub struct SearchUsersQuery {
    #[serde(flatten)]
    #[param(inline)]
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

#[derive(Debug, Clone, Builder)]
#[injectable]
pub struct SearchUsersQueryHandler {
    pool: PgPool,
    #[inject(|kvdb: Kvdb| CacheProvider::builder().key("iam_search_users:").ttl(Duration::from_secs(15 * 60)).kvdb(kvdb).build())]
    cache_provider: CacheProvider,
}

impl QueryHandler for SearchUsersQueryHandler {
    type Query = SearchUsersQuery;
    type Output = PagingResult<UserDto>;
    type Error = IamError;

    #[single_flight]
    #[tracing::instrument]
    async fn query(&self, query: SearchUsersQuery) -> Result<PagingResult<UserDto>, IamError> {
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
}

impl SearchUsersQueryHandler {
    #[tracing::instrument]
    pub async fn clean_cache(&self) -> ApplicationResult<()> {
        self.cache_provider.clear().await
    }

    #[tracing::instrument]
    pub async fn query_cached(
        &self,
        query: SearchUsersQuery,
    ) -> Result<PagingResult<UserDto>, IamError> {
        self.cache_provider.get_with(query, |q| self.query(q)).await
    }
}
