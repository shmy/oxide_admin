use crate::error::ApplicationError;
use crate::shared::command_handler::{CommandHandler, CommandResult};
use bon::Builder;
use domain::organization::event::OrganizationEvent;
use domain::organization::port::role_repository::RoleRepository;
use domain::organization::value_object::role_id::RoleId;
use infrastructure::repository::organization::role_repository::RoleRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct BatchEnableRolesCommand {
    ids: Vec<RoleId>,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct BatchEnableRolesCommandHandler {
    role_repository: RoleRepositoryImpl,
}

impl CommandHandler for BatchEnableRolesCommandHandler {
    type Command = BatchEnableRolesCommand;
    type Output = ();
    type Event = OrganizationEvent;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, ApplicationError> {
        let items = self.role_repository.toggle_enabled(&cmd.ids, true).await?;
        Ok(CommandResult::with_event(
            (),
            OrganizationEvent::RolesUpdated { items },
        ))
    }
}
