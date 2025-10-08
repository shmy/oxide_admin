use bon::Builder;
use domain::{
    shared::port::domain_repository::DomainRepository,
    system::{event::SystemEvent, value_object::sched_id::SchedId},
};
use infrastructure::repository::system::sched_repository::SchedRepositoryImpl;
use nject::injectable;
use serde::Deserialize;

use crate::{
    error::ApplicationError,
    shared::command_handler::{CommandHandler, CommandResult},
};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct BatchDeleteSchedsCommand {
    ids: Vec<SchedId>,
}

#[derive(Debug)]
#[injectable]
pub struct BatchDeleteSchedsCommandHandler {
    sched_repo: SchedRepositoryImpl,
}

impl CommandHandler for BatchDeleteSchedsCommandHandler {
    type Command = BatchDeleteSchedsCommand;
    type Output = ();
    type Event = SystemEvent;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, ApplicationError> {
        let items = self.sched_repo.batch_delete(&cmd.ids).await?;
        Ok(CommandResult::with_event(
            (),
            SystemEvent::SchedsDeleted { items },
        ))
    }
}
