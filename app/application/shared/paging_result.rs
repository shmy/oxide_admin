use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PagingResult<T> {
    pub total: i64,
    pub items: Vec<T>,
}
