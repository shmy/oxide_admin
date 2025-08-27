use anyhow::Result;
use bon::Builder;
use domain::iam::repository::user_repository::UserRepository;
use domain::iam::{error::IamError, event::IamEvent, value_object::user_id::UserId};
use infrastructure::repository::iam::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Deserialize, Builder)]
pub struct BatchEnableUsersCommand {
    ids: Vec<UserId>,
}

#[injectable]
pub struct BatchEnableUsersCommandHandler {
    user_repository: UserRepositoryImpl,
}

impl CommandHandler for BatchEnableUsersCommandHandler {
    type Command = BatchEnableUsersCommand;
    type Output = ();
    type Event = IamEvent;
    type Error = IamError;

    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let items = self.user_repository.toggle_enabled(&cmd.ids, true).await?;
        Ok(CommandResult::with_event(
            (),
            IamEvent::UsersUpdated { items },
        ))
    }
}
