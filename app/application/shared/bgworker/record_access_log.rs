use bg_worker_kit::Worker;
use bg_worker_kit::error::{Result, WorkerError};
use bon::Builder;
use infrastructure::shared::provider::Provider;
use serde::{Deserialize, Serialize};

use crate::{
    shared::command_handler::CommandHandler,
    system::command::create_access_log::{CreateAccessLogCommand, CreateAccessLogCommandHandler},
};

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct RecordAccessLog {
    user_id: String,
    method: String,
    uri: String,
    user_agent: Option<String>,
    ip: Option<String>,
    status: i16,
    elapsed: i64,
    occurred_at: chrono::NaiveDateTime,
}

impl Worker for RecordAccessLog {
    type State = Provider;

    const NAME: &'static str = "record_access_log";

    const CONCURRENCY: usize = 1;

    const RETRIES: usize = 3;

    const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

    async fn execute(params: Self, state: &Self::State) -> Result<()> {
        let command = CreateAccessLogCommand::builder()
            .user_id(params.user_id)
            .method(params.method)
            .uri(params.uri)
            .maybe_user_agent(params.user_agent)
            .maybe_ip(params.ip)
            .status(params.status)
            .elapsed(params.elapsed)
            .occurred_at(params.occurred_at)
            .build();
        let command_handler = state.provide::<CreateAccessLogCommandHandler>();
        command_handler
            .handle(command)
            .await
            .map_err(|e| WorkerError::Custom(e.to_string()))?;
        Ok(())
    }
}
