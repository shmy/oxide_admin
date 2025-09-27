use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::event::IamEvent;
use domain::iam::value_object::menu::Menu;
use domain::iam::value_object::permission::Permission;
use domain::iam::{entity::role::Role, value_object::role_id::RoleId};
use domain::shared::port::domain_repository::DomainRepository;
use infrastructure::repository::iam::role_repository::RoleRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct CreateRoleCommand {
    name: String,
    menus: Vec<Menu>,
    permissions: Vec<Permission>,
    enabled: bool,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct CreateRoleCommandHandler {
    role_repository: RoleRepositoryImpl,
}

impl CommandHandler for CreateRoleCommandHandler {
    type Command = CreateRoleCommand;
    type Output = Role;
    type Event = IamEvent;
    type Error = IamError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let role = Role::builder()
            .id(RoleId::generate())
            .name(cmd.name)
            .privileged(false)
            .menus(cmd.menus)
            .permissions(cmd.permissions)
            .enabled(cmd.enabled)
            .build();
        let role = self.role_repository.save(role).await?;
        Ok(CommandResult::with_event(
            role.clone(),
            IamEvent::RolesCreated { items: vec![role] },
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

    async fn build_command_handler(pool: PgPool) -> CreateRoleCommandHandler {
        setup_database(pool.clone()).await;
        let role_repository = RoleRepositoryImpl::builder()
            .pool(pool)
            .ct(ChronoTz::default())
            .build();
        CreateRoleCommandHandler::builder()
            .role_repository(role_repository)
            .build()
    }

    #[sqlx::test]
    async fn test_create(pool: PgPool) {
        let command_handler = build_command_handler(pool).await;
        let cmd = CreateRoleCommand::builder()
            .name("test".to_string())
            .menus(vec![])
            .permissions(vec![])
            .enabled(true)
            .build();
        assert!(command_handler.handle(cmd).await.is_ok());
    }

    #[sqlx::test]
    async fn test_create_return_err_given_duplicated_name(pool: PgPool) {
        let command_handler = build_command_handler(pool).await;
        let cmd = CreateRoleCommand::builder()
            .name("test".to_string())
            .menus(vec![])
            .permissions(vec![])
            .enabled(true)
            .build();
        assert!(command_handler.handle(cmd).await.is_ok());
        let cmd = CreateRoleCommand::builder()
            .name("test".to_string())
            .menus(vec![])
            .permissions(vec![])
            .enabled(true)
            .build();
        assert_eq!(
            command_handler.handle(cmd).await.err(),
            Some(IamError::RoleDuplicated)
        );
    }
}
