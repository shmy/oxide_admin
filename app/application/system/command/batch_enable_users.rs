use bon::Builder;
use domain::system::port::user_repository::UserRepository;
use domain::system::{error::SystemError, event::SystemEvent, value_object::user_id::UserId};
use infrastructure::repository::system::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

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
    type Event = SystemEvent;
    type Error = SystemError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let items = self.user_repository.toggle_enabled(&cmd.ids, true).await?;
        Ok(CommandResult::with_event(
            (),
            SystemEvent::UsersUpdated { items },
        ))
    }
}
