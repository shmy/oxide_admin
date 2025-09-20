use std::hash::Hash;

use serde::Deserialize;
use serde_with::{DisplayFromStr, PickFirst, serde_as};
use utoipa::ToSchema;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, ToSchema)]
pub struct PagingQuery {
    #[serde_as(as = "PickFirst<(_, DisplayFromStr)>")]
    #[serde(default = "PagingQuery::default_page")]
    #[schema(default = 1)]
    page: i64,
    #[serde_as(as = "PickFirst<(_, DisplayFromStr)>")]
    #[serde(default = "PagingQuery::default_page_size")]
    #[schema(default = 10)]
    page_size: i64,
}

impl PagingQuery {
    const DEFAULT_PAGE: i64 = 1;
    const DEFAULT_PAGE_SIZE: i64 = 10;

    fn default_page() -> i64 {
        Self::DEFAULT_PAGE
    }

    fn default_page_size() -> i64 {
        Self::DEFAULT_PAGE_SIZE
    }

    pub fn page(&self) -> i64 {
        self.page.max(1)
    }

    pub fn page_size(&self) -> i64 {
        self.page_size.clamp(1, 100)
    }
}

impl Default for PagingQuery {
    fn default() -> Self {
        Self {
            page: Self::DEFAULT_PAGE,
            page_size: Self::DEFAULT_PAGE_SIZE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paging_query() {
        let json = r#"{}"#;
        let query: PagingQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.page(), 1);
        assert_eq!(query.page_size(), 10);
    }
}
