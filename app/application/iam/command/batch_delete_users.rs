use bon::Builder;
use domain::{
    iam::{error::IamError, event::IamEvent, value_object::user_id::UserId},
    shared::port::domain_repository::DomainRepository,
};
use infrastructure::repository::iam::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct BatchDeleteUsersCommand {
    ids: Vec<UserId>,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct BatchDeleteUsersCommandHandler {
    user_repository: UserRepositoryImpl,
}

impl CommandHandler for BatchDeleteUsersCommandHandler {
    type Command = BatchDeleteUsersCommand;
    type Output = ();
    type Event = IamEvent;
    type Error = IamError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let items = self.user_repository.batch_delete(&cmd.ids).await?;
        Ok(CommandResult::with_event(
            (),
            IamEvent::UsersDeleted { items },
        ))
    }
}

#[cfg(test)]
mod tests {
    use infrastructure::{
        shared::{chrono_tz::ChronoTz, pg_pool::PgPool},
        test_utils::setup_database,
    };

    use super::*;

    async fn build_command_handler(pool: PgPool) -> BatchDeleteUsersCommandHandler {
        setup_database(pool.clone()).await;
        let user_repository = UserRepositoryImpl::builder()
            .pool(pool.clone())
            .ct(ChronoTz::default())
            .build();
        BatchDeleteUsersCommandHandler::builder()
            .user_repository(user_repository)
            .build()
    }

    #[sqlx::test]
    async fn test_batch_delete(pool: PgPool) {
        let command_handler = build_command_handler(pool).await;
        let cmd = BatchDeleteUsersCommand::builder()
            .ids(vec![UserId::generate()])
            .build();
        assert!(command_handler.handle(cmd).await.is_ok());
    }
}
