#[derive(Clone)]
pub struct PagingResult<T> {
    pub total: i64,
    pub items: Vec<T>,
}
