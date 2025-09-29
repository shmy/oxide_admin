use bon::Builder;
use domain::{
    shared::port::domain_repository::DomainRepository,
    system::{error::SystemError, event::IamEvent, value_object::user_id::UserId},
};
use infrastructure::repository::system::user_repository::UserRepositoryImpl;
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
    type Event = IamEvent;
    type Error = SystemError;

    #[tracing::instrument]
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
