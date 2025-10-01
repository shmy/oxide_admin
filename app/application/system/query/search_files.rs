use crate::shared::query_handler::QueryHandler;
use crate::{
    shared::{paging_query::PagingQuery, paging_result::PagingResult},
    system::dto::file::FileDto,
};
use bon::Builder;
use domain::system::error::SystemError;
use infrastructure::shared::pg_pool::PgPool;
use nject::injectable;
use serde::Deserialize;
use serde_with::{NoneAsEmptyString, serde_as};
use single_flight::single_flight;
use utoipa::IntoParams;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, IntoParams, Builder)]
pub struct SearchFilesQuery {
    #[serde(flatten)]
    #[param(inline)]
    paging: PagingQuery,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    name: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    used: Option<bool>,
}

#[derive(Debug, Clone)]
#[injectable]
pub struct SearchFilesQueryHandler {
    pool: PgPool,
}

impl QueryHandler for SearchFilesQueryHandler {
    type Query = SearchFilesQuery;
    type Output = PagingResult<FileDto>;
    type Error = SystemError;

    #[single_flight]
    #[tracing::instrument]
    async fn query(&self, query: SearchFilesQuery) -> Result<PagingResult<FileDto>, SystemError> {
        let total_future = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM _files
            WHERE ($1::text IS NULL OR name LIKE CONCAT('%', $1, '%'))
            AND ($2::boolean IS NULL OR used = $2)
            "#,
            query.name,
            query.used,
        )
        .fetch_one(&self.pool);
        let page = query.paging.page();
        let page_size = query.paging.page_size();
        let offset = (page - 1) * page_size;
        let rows_future = sqlx::query_as!(
            FileDto,
            r#"
        SELECT id, name, path, size, used, created_at, updated_at
        FROM _files
        WHERE ($1::text IS NULL OR name LIKE CONCAT('%', $1, '%'))
        AND ($2::boolean IS NULL OR used = $2)
        ORDER BY created_at DESC LIMIT $3 OFFSET $4 
        "#,
            query.name,
            query.used,
            page_size,
            offset,
        )
        .fetch_all(&self.pool);
        let (total, rows) = tokio::try_join!(total_future, rows_future)?;
        Ok(PagingResult { total, items: rows })
    }
}
