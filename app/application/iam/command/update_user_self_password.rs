use crate::shared::command_handler::{CommandHandler, CommandResult};
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::event::IamEvent;
use domain::iam::{entity::user::User, value_object::user_id::UserId};
use domain::shared::event_util::UpdatedEvent;
use domain::shared::port::domain_repository::DomainRepository;
use infrastructure::repository::iam::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;

#[derive(Debug, Deserialize, Builder)]
pub struct UpdateUserSelfPasswordCommand {
    id: UserId,
    password: String,
    new_password: String,
    confirm_new_password: String,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct UpdateUserSelfPasswordCommandHandler {
    user_repository: UserRepositoryImpl,
}

impl CommandHandler for UpdateUserSelfPasswordCommandHandler {
    type Command = UpdateUserSelfPasswordCommand;
    type Output = User;
    type Event = IamEvent;
    type Error = IamError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let password = cmd.password.trim();
        let new_password = cmd.new_password.trim();
        let confirm_new_password = cmd.confirm_new_password.trim();
        if new_password != confirm_new_password {
            return Err(IamError::PasswordMismatch);
        }
        if new_password == password {
            return Err(IamError::PasswordUnchanged);
        }
        let mut user = self.user_repository.by_id(&cmd.id).await?;
        let before = user.clone();
        user.assert_activated()?;
        user.password.verify(password)?;
        user.update_password(new_password.to_string())?;

        let user = self.user_repository.save(user).await?;
        Ok(CommandResult::with_event(
            user.clone(),
            IamEvent::UsersUpdated {
                items: vec![UpdatedEvent {
                    before,
                    after: user,
                }],
            },
        ))
    }
}

#[cfg(test)]
mod tests {

    use domain::iam::value_object::hashed_password::HashedPassword;
    use infrastructure::{
        shared::{chrono_tz::ChronoTz, pg_pool::PgPool},
        test_utils::setup_database,
    };

    use super::*;
    async fn build_command_handler(pool: PgPool) -> UpdateUserSelfPasswordCommandHandler {
        setup_database(pool.clone()).await;
        let user_repository = UserRepositoryImpl::builder()
            .pool(pool.clone())
            .ct(ChronoTz::default())
            .build();

        UpdateUserSelfPasswordCommandHandler::builder()
            .user_repository(user_repository)
            .build()
    }

    #[sqlx::test]
    async fn test_update_user_password_return_err_given_user_not_found(pool: PgPool) {
        let handler = build_command_handler(pool).await;
        let cmd = UpdateUserSelfPasswordCommand::builder()
            .id(UserId::generate())
            .password("abc123".to_string())
            .new_password("abc1234".to_string())
            .confirm_new_password("abc1234".to_string())
            .build();
        let result = handler.handle(cmd).await;
        assert_eq!(result.err(), Some(IamError::UserNotFound));
    }

    #[sqlx::test]
    async fn test_update_user_password_return_err_given_passwords_mismatch(pool: PgPool) {
        let handler = build_command_handler(pool).await;
        let user_id = UserId::generate();
        let user = User::builder()
            .id(user_id.clone())
            .account("test".to_string())
            .name("Test".to_string())
            .password(HashedPassword::try_new("123123".to_string()).unwrap())
            .privileged(true)
            .role_ids(vec![])
            .enabled(true)
            .build();
        assert!(handler.user_repository.save(user).await.is_ok());
        let cmd = UpdateUserSelfPasswordCommand::builder()
            .id(user_id)
            .password("abc123".to_string())
            .new_password("abc123".to_string())
            .confirm_new_password("abc1234".to_string())
            .build();
        let result = handler.handle(cmd).await;
        assert_eq!(result.err(), Some(IamError::PasswordMismatch));
    }

    #[sqlx::test]
    async fn test_update_user_password_return_err_given_password_unchanged(pool: PgPool) {
        let handler = build_command_handler(pool).await;
        let user_id = UserId::generate();
        let user = User::builder()
            .id(user_id.clone())
            .account("test".to_string())
            .name("Test".to_string())
            .password(HashedPassword::try_new("123123".to_string()).unwrap())
            .privileged(true)
            .role_ids(vec![])
            .enabled(true)
            .build();
        assert!(handler.user_repository.save(user).await.is_ok());
        let cmd = UpdateUserSelfPasswordCommand::builder()
            .id(user_id)
            .password("abc123".to_string())
            .new_password("abc123".to_string())
            .confirm_new_password("abc123".to_string())
            .build();
        let result = handler.handle(cmd).await;
        assert_eq!(result.err(), Some(IamError::PasswordUnchanged));
    }

    #[sqlx::test]
    async fn test_update_user_password_return_err_given_user_disabled(pool: PgPool) {
        let handler = build_command_handler(pool).await;
        let user_id = UserId::generate();
        let user = User::builder()
            .id(user_id.clone())
            .account("test".to_string())
            .name("Test".to_string())
            .password(HashedPassword::try_new("123123".to_string()).unwrap())
            .privileged(true)
            .role_ids(vec![])
            .enabled(false)
            .build();
        assert!(handler.user_repository.save(user).await.is_ok());
        let cmd = UpdateUserSelfPasswordCommand::builder()
            .id(user_id)
            .password("abc123".to_string())
            .new_password("abc1234".to_string())
            .confirm_new_password("abc1234".to_string())
            .build();
        let result = handler.handle(cmd).await;
        assert_eq!(result.err(), Some(IamError::UserDisabled));
    }
}
