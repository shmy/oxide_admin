use crate::iam::command::sign_out::{SignOutCommand, SignOutCommandHandler};
use crate::shared::command_handler::{CommandHandler, CommandResult};
use anyhow::Result;
use bon::Builder;
use domain::iam::port::user_repository::UserRepository;
use domain::iam::{error::IamError, event::IamEvent, value_object::user_id::UserId};
use futures_util::StreamExt as _;
use infrastructure::repository::iam::user_repository::UserRepositoryImpl;
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
    type Event = IamEvent;
    type Error = IamError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let items = self.user_repository.toggle_enabled(&cmd.ids, false).await?;
        let ids = cmd.ids.clone();
        tokio_stream::iter(ids)
            .for_each_concurrent(5, |id| async move {
                let command = SignOutCommand::builder().id(id).build();
                if let Err(err) = self.sign_out_command_handler.handle(command).await {
                    tracing::error!(%err, "注销用户失败");
                }
            })
            .await;
        Ok(CommandResult::with_event(
            (),
            IamEvent::UsersUpdated { items },
        ))
    }
}

#[cfg(test)]
mod tests {
    use infrastructure::{
        port::token_store_impl::TokenStoreImpl,
        shared::{chrono_tz::ChronoTz, pg_pool::PgPool},
        test_utils::{setup_database, setup_kvdb},
    };

    use super::*;

    async fn build_command_handler(pool: PgPool) -> BatchDisableUsersCommandHandler {
        setup_database(pool.clone()).await;
        let kvdb = setup_kvdb().await;
        let user_repository1 = UserRepositoryImpl::builder()
            .pool(pool.clone())
            .ct(ChronoTz::default())
            .build();
        let user_repository2 = UserRepositoryImpl::builder()
            .pool(pool.clone())
            .ct(ChronoTz::default())
            .build();
        let sign_out_command_handler = SignOutCommandHandler::builder()
            .user_repository(user_repository1)
            .token_store(TokenStoreImpl::builder().kvdb(kvdb).build())
            .build();
        BatchDisableUsersCommandHandler::builder()
            .user_repository(user_repository2)
            .sign_out_command_handler(sign_out_command_handler)
            .build()
    }

    #[sqlx::test]
    async fn test_batch_delete(pool: PgPool) {
        let command_handler = build_command_handler(pool).await;
        let cmd = BatchDisableUsersCommand::builder()
            .ids(vec![UserId::generate()])
            .build();
        assert!(command_handler.handle(cmd).await.is_ok());
    }
}
