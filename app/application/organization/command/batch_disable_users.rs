use crate::auth::command::sign_out::{SignOutCommand, SignOutCommandHandler};
use crate::error::ApplicationError;
use crate::shared::command_handler::{CommandHandler, CommandResult};
use bon::Builder;
use domain::organization::port::user_repository::UserRepository;
use domain::organization::{event::OrganizationEvent, value_object::user_id::UserId};
use futures_util::StreamExt as _;
use infrastructure::repository::organization::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;
#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct BatchDisableUsersCommand {
    ids: Vec<UserId>,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct BatchDisableUsersCommandHandler {
    user_repository: UserRepositoryImpl,
    sign_out_command_handler: SignOutCommandHandler,
}

impl CommandHandler for BatchDisableUsersCommandHandler {
    type Command = BatchDisableUsersCommand;
    type Output = ();
    type Event = OrganizationEvent;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, ApplicationError> {
        let items = self.user_repository.toggle_enabled(&cmd.ids, false).await?;
        let ids = cmd.ids.clone();
        tokio_stream::iter(ids)
            .for_each_concurrent(5, |id| async move {
                let command = SignOutCommand::builder().id(id).build();
                if let Err(err) = self.sign_out_command_handler.handle(command).await {
                    tracing::error!(%err, "Failed to sign out user");
                }
            })
            .await;
        Ok(CommandResult::with_event(
            (),
            OrganizationEvent::UsersUpdated { items },
        ))
    }
}
