use bon::Builder;
use domain::{
    organization::{
        error::OrganizationError, event::OrganizationEvent, value_object::user_id::UserId,
    },
    shared::port::domain_repository::DomainRepository,
};
use infrastructure::repository::organization::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct BatchDeleteUsersCommand {
    ids: Vec<UserId>,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct BatchDeleteUsersCommandHandler {
    user_repository: UserRepositoryImpl,
}

impl CommandHandler for BatchDeleteUsersCommandHandler {
    type Command = BatchDeleteUsersCommand;
    type Output = ();
    type Event = OrganizationEvent;
    type Error = OrganizationError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let items = self.user_repository.batch_delete(&cmd.ids).await?;
        Ok(CommandResult::with_event(
            (),
            OrganizationEvent::UsersDeleted { items },
        ))
    }
}
