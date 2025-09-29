use crate::shared::command_handler::{CommandHandler, CommandResult};
use bon::Builder;
use domain::iam::event::IamEvent;
use domain::iam::port::user_repository::UserRepository;
use domain::shared::port::domain_repository::DomainRepository;
use domain::shared::port::token_issuer::TokenIssuerTrait;
use domain::shared::port::token_store::TokenStoreTrait;
use domain::{iam::error::IamError, shared::port::token_issuer::TokenIssuerOutput};
use infrastructure::port::token_issuer_impl::TokenIssuerImpl;
use infrastructure::port::token_store_impl::TokenStoreImpl;
use infrastructure::repository::system::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct RefreshTokenCommand {
    refresh_token: String,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct RefreshTokenCommandHandler {
    user_repository: UserRepositoryImpl,
    token_issuer: TokenIssuerImpl,
    token_store: TokenStoreImpl,
}

impl CommandHandler for RefreshTokenCommandHandler {
    type Command = RefreshTokenCommand;
    type Output = TokenIssuerOutput;
    type Event = IamEvent;
    type Error = IamError;
    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let mut user = self
            .user_repository
            .by_refresh_token(cmd.refresh_token)
            .await?;
        user.assert_activated()?;
        user.assert_refresh_token_valid_period()?;
        let token_output = self.token_issuer.generate(user.id.to_string())?;
        user.update_refresh_token(
            Some(token_output.refresh_token.clone()),
            Some(token_output.refresh_token_expires_at.naive_utc()),
        );
        let id = user.id.clone();
        tokio::try_join!(
            self.token_store.store(
                user.id.to_string(),
                token_output.access_token.clone(),
                token_output.access_token_expires_at,
            ),
            self.user_repository.save(user),
        )?;
        Ok(CommandResult::with_event(
            token_output,
            IamEvent::UserRefreshTokenSucceeded { id },
        ))
    }
}
