pub mod error;
use serde::{Serialize, de::DeserializeOwned};

use crate::error::RunnerError;

#[cfg(feature = "faktory")]
mod faktory;
#[cfg(feature = "faktory")]
pub use faktory::*;

#[cfg(feature = "dummy")]
mod dummy;
#[cfg(feature = "dummy")]
pub use dummy::*;

#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "sqlite")]
pub use sqlite::*;

pub trait JobRunner {
    type Params: Serialize + DeserializeOwned;
    fn run(&self, params: Self::Params) -> impl Future<Output = Result<(), RunnerError>> + Send;
}
