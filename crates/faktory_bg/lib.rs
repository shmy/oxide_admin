pub mod error;
pub mod queuer;
pub mod worker_manager;
pub use faktory::Job;
use serde::{Serialize, de::DeserializeOwned};

use crate::error::RunnerError;

pub trait JobRunner {
    type Params: Serialize + DeserializeOwned;
    fn run(&self, params: Self::Params) -> impl Future<Output = Result<(), RunnerError>> + Send;
}
struct RunnerWrapper<T>(pub T)
where
    T: JobRunner;

#[async_trait::async_trait]
impl<T> faktory::JobRunner for RunnerWrapper<T>
where
    T: JobRunner + Send + Sync + 'static,
{
    type Error = RunnerError;
    async fn run(&self, job: Job) -> Result<(), Self::Error> {
        if let Some(arg) = job.args().first() {
            let params: T::Params = serde_json::from_value(arg.clone())?;
            return self.0.run(params).await;
        }
        Err(RunnerError::Custom("No params".to_string()))
    }
}
