use crate::shared::command_handler::{CommandHandler, CommandResult};
use bon::Builder;
use domain::organization::port::role_repository::RoleRepository;
use domain::organization::value_object::role_id::RoleId;
use domain::organization::{error::OrganizationError, event::OrganizationEvent};
use infrastructure::repository::organization::role_repository::RoleRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct BatchDisableRolesCommand {
    ids: Vec<RoleId>,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct BatchDisableRolesCommandHandler {
    role_repository: RoleRepositoryImpl,
}

impl CommandHandler for BatchDisableRolesCommandHandler {
    type Command = BatchDisableRolesCommand;
    type Output = ();
    type Event = OrganizationEvent;
    type Error = OrganizationError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let items = self.role_repository.toggle_enabled(&cmd.ids, false).await?;
        Ok(CommandResult::with_event(
            (),
            OrganizationEvent::RolesUpdated { items },
        ))
    }
}
