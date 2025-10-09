use bg_worker_kit::{
    Worker,
    error::{Result, WorkerError},
};
use bon::Builder;
use nject::injectable;
use serde::{Deserialize, Serialize};

use crate::{
    shared::command_handler::CommandHandler,
    system::command::create_access_log::{CreateAccessLogCommand, CreateAccessLogCommandHandler},
};

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct RecordAccessLogParams {
    user_id: String,
    method: String,
    uri: String,
    user_agent: Option<String>,
    ip: Option<String>,
    status: i16,
    elapsed: i64,
}

#[derive(Clone)]
#[injectable]
pub struct RecordAccessLog {
    command_handler: CreateAccessLogCommandHandler,
}

impl Worker for RecordAccessLog {
    const KIND: &'static str = "record_access_log";
    type Params = RecordAccessLogParams;

    async fn run(&self, params: Self::Params) -> Result<()> {
        let command = CreateAccessLogCommand::builder()
            .user_id(params.user_id)
            .method(params.method)
            .uri(params.uri)
            .maybe_user_agent(params.user_agent)
            .maybe_ip(params.ip)
            .status(params.status)
            .elapsed(params.elapsed)
            .build();
        self.command_handler
            .handle(command)
            .await
            .map_err(|e| WorkerError::Custom(e.to_string()))?;
        Ok(())
    }
}
