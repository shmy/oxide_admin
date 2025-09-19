use crate::iam::command::sign_out::{SignOutCommand, SignOutCommandHandler};
use crate::shared::command_handler::{CommandHandler, CommandResult};
use anyhow::Result;
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::event::IamEvent;
use domain::iam::value_object::role_id::RoleId;
use domain::iam::{entity::user::User, value_object::user_id::UserId};
use domain::shared::event_util::UpdatedEvent;
use domain::shared::port::domain_repository::DomainRepository;
use infrastructure::repository::iam::user_repository::UserRepositoryImpl;
use nject::injectable;
use object_storage_kit::{ObjectStorage, ObjectStorageReader as _};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct UpdateUserCommand {
    id: UserId,
    account: Option<String>,
    portrait: Option<Option<String>>,
    name: Option<String>,
    role_ids: Option<Vec<RoleId>>,
    enabled: Option<bool>,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct UpdateUserCommandHandler {
    user_repository: UserRepositoryImpl,
    sign_out_command_handler: SignOutCommandHandler,
    object_storage: ObjectStorage,
}

impl CommandHandler for UpdateUserCommandHandler {
    type Command = UpdateUserCommand;
    type Output = User;
    type Event = IamEvent;
    type Error = IamError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let id = cmd.id;
        let mut user = self.user_repository.by_id(&id).await?;
        if user.privileged {
            return Err(IamError::UserPrivilegedImmutable);
        }
        let before = user.clone();
        if let Some(account) = cmd.account {
            user.update_account(account);
        }
        if let Some(portrait) = cmd.portrait {
            let portrait = self.object_storage.purify_url_opt(portrait);
            user.update_portrait(portrait);
        }
        if let Some(name) = cmd.name {
            user.update_name(name);
        }
        if let Some(role_ids) = cmd.role_ids {
            user.update_role_ids(role_ids);
        }
        if let Some(enabled) = cmd.enabled {
            user.update_enabled(enabled);
        }
        let user = self.user_repository.save(user).await?;
        if !user.enabled {
            let command = SignOutCommand::builder().id(id.clone()).build();
            if let Err(err) = self.sign_out_command_handler.handle(command).await {
                tracing::error!(%err, "注销用户失败");
            }
        }
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
    use std::time::Duration;

    use domain::iam::value_object::hashed_password::HashedPassword;
    use infrastructure::{
        port::token_store_impl::TokenStoreImpl,
        shared::{chrono_tz::ChronoTz, pg_pool::PgPool},
        test_utils::{setup_database, setup_kvdb},
    };
    use object_storage_kit::FsConfig;

    use super::*;
    async fn build_command_handler(pool: PgPool) -> UpdateUserCommandHandler {
        setup_database(pool.clone()).await;
        let kvdb = setup_kvdb().await;
        let user_repository = UserRepositoryImpl::builder()
            .pool(pool.clone())
            .ct(ChronoTz::default())
            .build();
        let object_storage = {
            let dir = tempfile::tempdir().unwrap();
            ObjectStorage::try_new(
                FsConfig::builder()
                    .root(dir.path().to_string_lossy().to_string())
                    .basepath("/uploads".to_string())
                    .hmac_secret(b"secret")
                    .link_period(Duration::from_secs(60))
                    .build(),
            )
            .unwrap()
        };
        let sign_out_command_handler = {
            let user_repository = UserRepositoryImpl::builder()
                .pool(pool.clone())
                .ct(ChronoTz::default())
                .build();
            SignOutCommandHandler::builder()
                .user_repository(user_repository)
                .token_store(TokenStoreImpl::builder().kvdb(kvdb).build())
                .build()
        };
        UpdateUserCommandHandler::builder()
            .user_repository(user_repository)
            .object_storage(object_storage)
            .sign_out_command_handler(sign_out_command_handler)
            .build()
    }

    #[sqlx::test]
    async fn test_update_user_return_err_given_user_not_found(pool: PgPool) {
        let handler = build_command_handler(pool).await;
        let cmd = UpdateUserCommand::builder()
            .id(UserId::generate())
            .name("test".to_string())
            .enabled(true)
            .build();
        let result = handler.handle(cmd).await;
        assert_eq!(result.err(), Some(IamError::UserNotFound));
    }

    #[sqlx::test]
    async fn test_update_user_return_err_given_user_is_privated(pool: PgPool) {
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
        let cmd = UpdateUserCommand::builder()
            .id(user_id)
            .name("test".to_string())
            .enabled(true)
            .role_ids(vec![])
            .build();
        let result = handler.handle(cmd).await;
        assert_eq!(result.err(), Some(IamError::UserPrivilegedImmutable));
    }
}
