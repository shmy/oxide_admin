use serde::{Serialize, de::DeserializeOwned};
use tracing::warn;

use crate::{JobRunner, error::RunnerError};

pub struct WorkerManager {}

impl Default for WorkerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn register<K, P, H>(&mut self, _kind: K, _runner: H)
    where
        K: Into<String>,
        H: JobRunner<Params = P> + Send + Sync + 'static,
    {
        warn!("Dummy worker manager is used, this is not a real worker manager");
    }

    pub fn register_fn<K, H, P, Fut>(&mut self, _kind: K, _handler: H)
    where
        K: Into<String>,
        H: Fn(P) -> Fut + Send + Sync + 'static,
        P: Serialize + DeserializeOwned,
        Fut: Future<Output = Result<(), RunnerError>> + Send,
    {
        warn!("Dummy worker manager is used, this is not a real worker manager");
    }

    pub fn register_blocking_fn<K, H, P>(self, _kind: K, _handler: H)
    where
        K: Into<String>,
        H: Fn(P) -> Result<(), RunnerError> + Send + Sync + 'static,
        P: Serialize + DeserializeOwned,
    {
        warn!("Dummy worker manager is used, this is not a real worker manager");
    }

    pub async fn run_with_signal<S>(&mut self, signal: S) -> anyhow::Result<()>
    where
        S: Future<Output = ()> + 'static + Send,
    {
        signal.await;
        Ok(())
    }
}
