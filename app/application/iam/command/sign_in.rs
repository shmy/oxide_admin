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

#[derive(Debug, Builder)]
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

#[cfg(test)]
mod tests {
    use domain::iam::{
        entity::user::User,
        value_object::{hashed_password::HashedPassword, user_id::UserId},
    };
    use infrastructure::{
        shared::{chrono_tz::ChronoTz, config::ConfigRef, pg_pool::PgPool},
        test_utils::{setup_database, setup_kvdb},
    };

    use super::*;

    async fn build_command_handler(pool: PgPool) -> SignInCommandHandler {
        setup_database(pool.clone()).await;
        let kvdb = setup_kvdb().await;
        let captcha_issuer = CaptchaIssuerImpl::builder().kvdb(kvdb.clone()).build();
        let user_repository = UserRepositoryImpl::builder()
            .pool(pool)
            .ct(ChronoTz::default())
            .build();
        let token_issuer = TokenIssuerImpl::builder()
            .config(ConfigRef::default())
            .ct(ChronoTz::default())
            .build();
        let token_store = TokenStoreImpl::builder().kvdb(kvdb).build();
        SignInCommandHandler::builder()
            .captcha_issuer(captcha_issuer)
            .user_repository(user_repository)
            .token_issuer(token_issuer)
            .token_store(token_store)
            .build()
    }

    #[sqlx::test]
    async fn test_sign_in_return_err_given_captcha_invalid(pool: PgPool) {
        let command_handler = build_command_handler(pool).await;
        let user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .password(HashedPassword::try_new("123123".to_string()).unwrap())
            .privileged(false)
            .role_ids(vec![])
            .enabled(true)
            .build();
        assert!(command_handler.user_repository.save(user).await.is_ok());

        assert_eq!(
            command_handler
                .handle(
                    SignInCommand::builder()
                        .account("test".to_string())
                        .password("123123".to_string())
                        .captcha_key("test-key".to_string())
                        .captcha_value("test-value".to_string())
                        .build()
                )
                .await
                .err(),
            Some(IamError::CaptchaInvalid)
        );
    }

    #[test]
    fn test_debug_command() {
        let command = SignInCommand::builder()
            .account("test".to_string())
            .password("123123".to_string())
            .captcha_key("test-key".to_string())
            .captcha_value("test-value".to_string())
            .build();
        assert_eq!(
            format!("{:?}", command),
            "SignInCommand { account: \"test\", password: \"<RESERVED>\", captcha_key: \"test-key\", captcha_value: \"test-value\" }"
        );
    }
}
