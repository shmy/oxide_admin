use std::fmt::Debug;

use crate::shared::command_handler::{CommandHandler, CommandResult};
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::event::IamEvent;
use domain::iam::port::user_repository::UserRepository;
use domain::shared::port::captcha_issuer::CaptchaIssuerTrait as _;
use domain::shared::port::domain_repository::DomainRepository;
use domain::shared::port::token_issuer::{TokenIssuerOutput, TokenIssuerTrait};
use domain::shared::port::token_store::TokenStoreTrait;
use infrastructure::port::captcha_issuer_impl::CaptchaIssuerImpl;
use infrastructure::port::token_issuer_impl::TokenIssuerImpl;
use infrastructure::port::token_store_impl::TokenStoreImpl;
use infrastructure::repository::iam::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, Builder, ToSchema)]
pub struct SignInCommand {
    account: String,
    password: String,
    captcha_key: String,
    captcha_value: String,
}

impl Debug for SignInCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignInCommand")
            .field("account", &self.account)
            .field("password", &"<RESERVED>")
            .field("captcha_key", &self.captcha_key)
            .field("captcha_value", &self.captcha_value)
            .finish()
    }
}

#[derive(Debug)]
#[injectable]
pub struct SignInCommandHandler {
    captcha_issuer: CaptchaIssuerImpl,
    user_repository: UserRepositoryImpl,
    token_issuer: TokenIssuerImpl,
    token_store: TokenStoreImpl,
}

impl CommandHandler for SignInCommandHandler {
    type Command = SignInCommand;
    type Output = TokenIssuerOutput;
    type Event = IamEvent;
    type Error = IamError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        self.captcha_issuer
            .verify(&cmd.captcha_key, &cmd.captcha_value)
            .await?;
        let mut user = self.user_repository.by_account(cmd.account).await?;
        user.assert_activated()?;
        user.password.verify(&cmd.password)?;
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
            IamEvent::UserLoginSucceeded { id },
        ))
    }
}
