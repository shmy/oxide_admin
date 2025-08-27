use std::hash::Hash;

use serde::Deserialize;
use serde_with::{DisplayFromStr, PickFirst, serde_as};

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
pub struct PagingQuery {
    #[serde_as(as = "PickFirst<(_, DisplayFromStr)>")]
    #[serde(default = "PagingQuery::default_page")]
    page: i64,
    #[serde_as(as = "PickFirst<(_, DisplayFromStr)>")]
    #[serde(default = "PagingQuery::default_page_size")]
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
