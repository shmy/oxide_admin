#[derive(Debug, Clone)]
pub struct UpdatedEvent<T> {
    pub before: T,
    pub after: T,
}
