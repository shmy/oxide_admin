use bon::Builder;
use domain::{
    organization::{event::OrganizationEvent, value_object::role_id::RoleId},
    shared::port::domain_repository::DomainRepository,
};
use infrastructure::repository::organization::role_repository::RoleRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    error::ApplicationError,
    shared::command_handler::{CommandHandler, CommandResult},
};

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
    type Event = OrganizationEvent;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, ApplicationError> {
        let items = self.role_repository.batch_delete(&cmd.ids).await?;
        Ok(CommandResult::with_event(
            (),
            OrganizationEvent::RolesDeleted { items },
        ))
    }
}
