pub mod error;
use serde::{Serialize, de::DeserializeOwned};

use crate::error::WorkerError;

#[cfg(feature = "faktory")]
mod faktory;
#[cfg(feature = "faktory")]
pub use faktory::*;

#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "sqlite")]
pub use sqlite::*;

pub trait Worker {
    type Params: Serialize + DeserializeOwned;
    fn run(&self, params: Self::Params) -> impl Future<Output = Result<(), WorkerError>> + Send;
}
