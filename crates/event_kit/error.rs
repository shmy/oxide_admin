use thiserror::Error;

pub type Result<T> = std::result::Result<T, EventError>;

#[derive(Debug, Error)]
pub enum EventError {}
