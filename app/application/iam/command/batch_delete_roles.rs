use anyhow::Result;
use bon::Builder;
use domain::{
    iam::{error::IamError, event::IamEvent, value_object::role_id::RoleId},
    shared::port::domain_repository::DomainRepository,
};
use infrastructure::repository::iam::role_repository::RoleRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct BatchDeleteRolesCommand {
    ids: Vec<RoleId>,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct BatchDeleteRolesCommandHandler {
    role_repository: RoleRepositoryImpl,
}

impl CommandHandler for BatchDeleteRolesCommandHandler {
    type Command = BatchDeleteRolesCommand;
    type Output = ();
    type Event = IamEvent;
    type Error = IamError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let items = self.role_repository.batch_delete(&cmd.ids).await?;
        Ok(CommandResult::with_event(
            (),
            IamEvent::RolesDeleted { items },
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

    async fn build_command_handler(pool: PgPool) -> BatchDeleteRolesCommandHandler {
        setup_database(pool.clone()).await;
        let role_repository = RoleRepositoryImpl::builder()
            .pool(pool.clone())
            .ct(ChronoTz::default())
            .build();
        let command_handler = BatchDeleteRolesCommandHandler::builder()
            .role_repository(role_repository)
            .build();
        command_handler
    }

    #[sqlx::test]
    async fn test_batch_delete(pool: PgPool) {
        let command_handler = build_command_handler(pool).await;
        let cmd = BatchDeleteRolesCommand::builder()
            .ids(vec![RoleId::generate()])
            .build();
        assert!(command_handler.handle(cmd).await.is_ok());
    }
}
