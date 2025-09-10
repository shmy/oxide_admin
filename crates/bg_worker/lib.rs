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

pub trait JobRunner {
    type Params: Serialize + DeserializeOwned;
    fn run(&self, params: Self::Params) -> impl Future<Output = Result<(), RunnerError>> + Send;
}

#[cfg(feature = "faktory")]
struct RunnerWrapper<T>(pub T)
where
    T: JobRunner;

#[cfg(feature = "faktory")]
#[async_trait::async_trait]
impl<T> ::faktory::JobRunner for RunnerWrapper<T>
where
    T: JobRunner + Send + Sync + 'static,
{
    type Error = RunnerError;
    async fn run(&self, job: ::faktory::Job) -> Result<(), Self::Error> {
        if let Some(arg) = job.args().first() {
            let params: T::Params = serde_json::from_value(arg.clone())?;
            return self.0.run(params).await;
        }
        Err(RunnerError::Custom("No params".to_string()))
    }
}
