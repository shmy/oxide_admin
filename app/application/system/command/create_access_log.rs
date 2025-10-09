use crate::error::ApplicationError;
use crate::shared::command_handler::{CommandHandler, CommandResult};
use bon::Builder;
use domain::shared::port::domain_repository::DomainRepository;
use domain::system::entity::access_log::AccessLog;
use domain::system::event::SystemEvent;
use domain::system::value_object::access_log_id::AccessLogId;
use infrastructure::repository::system::access_log_repository::AccessLogRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct CreateAccessLogCommand {
    user_id: String,
    method: String,
    uri: String,
    user_agent: Option<String>,
    ip: Option<String>,
    ip_region: Option<String>,
    status: i16,
    elapsed: i64,
}

#[derive(Clone, Debug)]
#[injectable]
pub struct CreateAccessLogCommandHandler {
    access_log_repo: AccessLogRepositoryImpl,
}

impl CommandHandler for CreateAccessLogCommandHandler {
    type Command = CreateAccessLogCommand;
    type Output = AccessLog;
    type Event = SystemEvent;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, ApplicationError> {
        let access_log = AccessLog::builder()
            .id(AccessLogId::generate())
            .user_id(cmd.user_id)
            .method(cmd.method)
            .uri(cmd.uri)
            .maybe_user_agent(cmd.user_agent)
            .maybe_ip(cmd.ip)
            .maybe_ip_region(cmd.ip_region)
            .status(cmd.status)
            .elapsed(cmd.elapsed)
            .build();
        let access_log = self.access_log_repo.save(access_log).await?;
        Ok(CommandResult::with_event(
            access_log.clone(),
            SystemEvent::AccessLogsCreated {
                items: vec![access_log],
            },
        ))
    }
}
