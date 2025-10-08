use bon::Builder;
use domain::organization::port::user_repository::UserRepository;
use domain::organization::{event::OrganizationEvent, value_object::user_id::UserId};
use infrastructure::repository::organization::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::error::ApplicationError;
use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct BatchEnableUsersCommand {
    ids: Vec<UserId>,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct BatchEnableUsersCommandHandler {
    user_repository: UserRepositoryImpl,
}

impl CommandHandler for BatchEnableUsersCommandHandler {
    type Command = BatchEnableUsersCommand;
    type Output = ();
    type Event = OrganizationEvent;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, ApplicationError> {
        let items = self.user_repository.toggle_enabled(&cmd.ids, true).await?;
        Ok(CommandResult::with_event(
            (),
            OrganizationEvent::UsersUpdated { items },
        ))
    }
}
