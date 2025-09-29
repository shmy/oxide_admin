use bon::Builder;
use domain::{
    shared::port::domain_repository::DomainRepository,
    system::{error::SystemError, event::IamEvent, value_object::role_id::RoleId},
};
use infrastructure::repository::system::role_repository::RoleRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct BatchDeleteRolesCommand {
    ids: Vec<RoleId>,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct BatchDeleteRolesCommandHandler {
    role_repository: RoleRepositoryImpl,
}

impl CommandHandler for BatchDeleteRolesCommandHandler {
    type Command = BatchDeleteRolesCommand;
    type Output = ();
    type Event = IamEvent;
    type Error = SystemError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let items = self.role_repository.batch_delete(&cmd.ids).await?;
        Ok(CommandResult::with_event(
            (),
            IamEvent::RolesDeleted { items },
        ))
    }
}
