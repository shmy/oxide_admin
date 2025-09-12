use crate::shared::command_handler::{CommandHandler, CommandResult};
use anyhow::Result;
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::event::IamEvent;
use domain::iam::value_object::user_id::UserId;
use domain::shared::port::domain_repository::DomainRepository;
use domain::shared::port::token_store::TokenStoreTrait;
use infrastructure::port::token_store_impl::TokenStoreImpl;
use infrastructure::repository::iam::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
#[derive(Deserialize, Builder)]
pub struct SignOutCommand {
    id: UserId,
}

#[derive(Debug)]
#[injectable]
pub struct SignOutCommandHandler {
    user_repository: UserRepositoryImpl,
    token_store: TokenStoreImpl,
}

impl CommandHandler for SignOutCommandHandler {
    type Command = SignOutCommand;
    type Output = ();
    type Event = IamEvent;
    type Error = IamError;

    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let id = cmd.id;
        if let Ok(mut user) = self.user_repository.by_id(&id).await {
            user.update_refresh_token(None, None);

            tokio::try_join!(
                self.token_store.delete(user.id.to_string()),
                self.user_repository.save(user),
            )?;
        }
        Ok(CommandResult::with_event(
            (),
            IamEvent::UserLogoutSucceeded { id },
        ))
    }
}
