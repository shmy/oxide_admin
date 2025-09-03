use anyhow::Result;
use bon::Builder;
use domain::{
    iam::{error::IamError, event::IamEvent, value_object::user_id::UserId},
    shared::port::domain_repository::DomainRepository,
};
use infrastructure::repository::iam::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Deserialize, Builder)]
pub struct BatchDeleteUsersCommand {
    ids: Vec<UserId>,
}

#[injectable]
pub struct BatchDeleteUsersCommandHandler {
    user_repository: UserRepositoryImpl,
}

impl CommandHandler for BatchDeleteUsersCommandHandler {
    type Command = BatchDeleteUsersCommand;
    type Output = ();
    type Event = IamEvent;
    type Error = IamError;

    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let items = self.user_repository.batch_delete(&cmd.ids).await?;
        Ok(CommandResult::with_event(
            (),
            IamEvent::UsersDeleted { items },
        ))
    }
}
