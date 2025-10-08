use thiserror::Error;

pub type Result<T> = std::result::Result<T, KvdbError>;

#[derive(Debug, Error)]
pub enum KvdbError {
    #[cfg(feature = "redb")]
    #[error("{0}")]
    Redb(#[from] redb::DatabaseError),

    #[cfg(feature = "redb")]
    #[error("{0}")]
    RedbTransaction(#[from] redb::TransactionError),

    #[cfg(feature = "redb")]
    #[error("{0}")]
    RedbTable(#[from] redb::TableError),

    #[cfg(feature = "redb")]
    #[error("{0}")]
    RedbStorage(#[from] redb::StorageError),

    #[cfg(feature = "redb")]
    #[error("{0}")]
    RedbCommit(#[from] redb::CommitError),

    #[cfg(feature = "redb")]
    #[error("{0}")]
    CronTab(#[from] cron_tab::CronError),

    #[cfg(feature = "redis")]
    #[error("{0}")]
    Redis(#[from] bb8_redis::redis::RedisError),

    #[cfg(feature = "redis")]
    #[error("{0}")]
    RunRedis(#[from] bb8_redis::bb8::RunError<bb8_redis::redis::RedisError>),

    #[error("{0}")]
    CborEncode(#[from] minicbor_serde::error::EncodeError<core::convert::Infallible>),

    #[error("{0}")]
    CborDecode(#[from] minicbor_serde::error::DecodeError),
}
