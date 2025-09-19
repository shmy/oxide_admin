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
use infrastructure::repository::iam::user_repository::UserRepositoryImpl;
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use domain::iam::{
        entity::user::User,
        value_object::{hashed_password::HashedPassword, user_id::UserId},
    };
    use infrastructure::{
        shared::{chrono_tz::ChronoTz, config::Config, pg_pool::PgPool},
        test_utils::{setup_database, setup_kvdb},
    };
    use sqlx::types::chrono::Utc;

    use super::*;
    async fn build_command_handler(pool: PgPool) -> RefreshTokenCommandHandler {
        setup_database(pool.clone()).await;
        let kvdb = setup_kvdb().await;
        let user_repository = UserRepositoryImpl::builder()
            .pool(pool)
            .ct(ChronoTz::default())
            .build();
        let token_issuer = TokenIssuerImpl::builder()
            .config(Config::default())
            .ct(ChronoTz::default())
            .build();
        let token_store = TokenStoreImpl::builder().kvdb(kvdb).build();
        RefreshTokenCommandHandler::builder()
            .user_repository(user_repository)
            .token_issuer(token_issuer)
            .token_store(token_store)
            .build()
    }

    #[sqlx::test]
    async fn test_refresh_token(pool: PgPool) {
        let command_handler = build_command_handler(pool).await;
        let user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .password(HashedPassword::try_new("123123".to_string()).unwrap())
            .privileged(false)
            .role_ids(vec![])
            .refresh_token_expired_at(Utc::now().naive_utc() + Duration::from_secs(60))
            .refresh_token("test-token".to_string())
            .enabled(true)
            .build();
        assert!(command_handler.user_repository.save(user).await.is_ok());

        assert!(
            command_handler
                .handle(
                    RefreshTokenCommand::builder()
                        .refresh_token("test-token".to_string())
                        .build(),
                )
                .await
                .is_ok()
        );
    }

    #[sqlx::test]
    async fn test_refresh_token_return_err_given_user_not_found(pool: PgPool) {
        let command_handler = build_command_handler(pool).await;
        assert_eq!(
            command_handler
                .handle(
                    RefreshTokenCommand::builder()
                        .refresh_token("fake-token".to_string())
                        .build()
                )
                .await
                .err(),
            Some(IamError::UserNotFound)
        );
    }

    #[sqlx::test]
    async fn test_refresh_token_return_err_given_user_disabled(pool: PgPool) {
        let command_handler = build_command_handler(pool).await;
        let user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .password(HashedPassword::try_new("123123".to_string()).unwrap())
            .privileged(false)
            .role_ids(vec![])
            .refresh_token_expired_at(Utc::now().naive_utc() + Duration::from_secs(60))
            .refresh_token("test-token".to_string())
            .enabled(false)
            .build();
        assert!(command_handler.user_repository.save(user).await.is_ok());

        assert_eq!(
            command_handler
                .handle(
                    RefreshTokenCommand::builder()
                        .refresh_token("test-token".to_string())
                        .build()
                )
                .await
                .err(),
            Some(IamError::UserDisabled)
        );
    }

    #[sqlx::test]
    async fn test_refresh_token_return_err_given_user_refreh_token_expired(pool: PgPool) {
        let command_handler = build_command_handler(pool).await;
        let user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .password(HashedPassword::try_new("123123".to_string()).unwrap())
            .privileged(false)
            .role_ids(vec![])
            .refresh_token_expired_at(Utc::now().naive_utc() - Duration::from_secs(60))
            .refresh_token("test-token".to_string())
            .enabled(true)
            .build();
        assert!(command_handler.user_repository.save(user).await.is_ok());

        assert_eq!(
            command_handler
                .handle(
                    RefreshTokenCommand::builder()
                        .refresh_token("test-token".to_string())
                        .build()
                )
                .await
                .err(),
            Some(IamError::RefreshTokenExpired)
        );
    }
}
