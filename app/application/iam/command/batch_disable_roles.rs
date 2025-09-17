use crate::shared::command_handler::{CommandHandler, CommandResult};
use anyhow::Result;
use bon::Builder;
use domain::iam::port::role_repository::RoleRepository;
use domain::iam::value_object::role_id::RoleId;
use domain::iam::{error::IamError, event::IamEvent};
use infrastructure::repository::iam::role_repository::RoleRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct BatchDisableRolesCommand {
    ids: Vec<RoleId>,
}

#[derive(Debug)]
#[injectable]
pub struct BatchDisableRolesCommandHandler {
    role_repository: RoleRepositoryImpl,
}

impl CommandHandler for BatchDisableRolesCommandHandler {
    type Command = BatchDisableRolesCommand;
    type Output = ();
    type Event = IamEvent;
    type Error = IamError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let items = self.role_repository.toggle_enabled(&cmd.ids, false).await?;
        Ok(CommandResult::with_event(
            (),
            IamEvent::RolesUpdated { items },
        ))
    }
}
