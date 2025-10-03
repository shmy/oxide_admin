use crate::shared::command_handler::{CommandHandler, CommandResult};
use bon::Builder;
use domain::auth::error::AuthError;
use domain::auth::event::AuthEvent;
use domain::auth::port::token_store::TokenStoreTrait;
use domain::organization::value_object::user_id::UserId;
use domain::shared::port::domain_repository::DomainRepository;
use futures_util::TryFutureExt;
use infrastructure::port::token_store_impl::TokenStoreImpl;
use infrastructure::repository::organization::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;

#[derive(Debug, Deserialize, Builder)]
pub struct SignOutCommand {
    id: UserId,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct SignOutCommandHandler {
    user_repository: UserRepositoryImpl,
    token_store: TokenStoreImpl,
}

impl CommandHandler for SignOutCommandHandler {
    type Command = SignOutCommand;
    type Output = ();
    type Event = AuthEvent;
    type Error = AuthError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let id = cmd.id;
        if let Ok(mut user) = self.user_repository.by_id(&id).await {
            user.update_refresh_token(None, None);

            tokio::try_join!(
                self.token_store.delete(user.id.to_string()),
                self.user_repository.save(user).map_err(Into::into),
            )?;
        }
        Ok(CommandResult::with_event(
            (),
            AuthEvent::UserLogoutSucceeded { id },
        ))
    }
}
