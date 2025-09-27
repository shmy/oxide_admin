use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::value_object::menu::Menu;
use domain::iam::value_object::permission::Permission;
use domain::iam::value_object::role_id::RoleId;
use domain::iam::{entity::role::Role, event::IamEvent};
use domain::shared::event_util::UpdatedEvent;
use domain::shared::port::domain_repository::DomainRepository;
use infrastructure::repository::iam::role_repository::RoleRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct UpdateRoleCommand {
    id: RoleId,
    name: Option<String>,
    menus: Option<Vec<Menu>>,
    permissions: Option<Vec<Permission>>,
    enabled: Option<bool>,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct UpdateRoleCommandHandler {
    role_repository: RoleRepositoryImpl,
}

impl CommandHandler for UpdateRoleCommandHandler {
    type Command = UpdateRoleCommand;
    type Output = Role;
    type Event = IamEvent;
    type Error = IamError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let id = cmd.id;
        let mut role = self.role_repository.by_id(&id).await?;
        if role.privileged {
            return Err(IamError::RolePrivilegedImmutable);
        }
        let before = role.clone();
        if let Some(name) = cmd.name {
            role.update_name(name);
        }
        if let Some(menus) = cmd.menus {
            role.update_menus(menus);
        }
        if let Some(permissions) = cmd.permissions {
            role.update_permissions(permissions);
        }
        if let Some(enabled) = cmd.enabled {
            role.update_enabled(enabled);
        }
        let role = self.role_repository.save(role).await?;
        Ok(CommandResult::with_event(
            role.clone(),
            IamEvent::RolesUpdated {
                items: vec![UpdatedEvent {
                    before,
                    after: role,
                }],
            },
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
    async fn build_command_handler(pool: PgPool) -> UpdateRoleCommandHandler {
        setup_database(pool.clone()).await;
        let role_repository = RoleRepositoryImpl::builder()
            .pool(pool)
            .ct(ChronoTz::default())
            .build();
        UpdateRoleCommandHandler::builder()
            .role_repository(role_repository)
            .build()
    }

    #[sqlx::test]
    async fn test_update_role_return_err_given_role_not_found(pool: PgPool) {
        let handler = build_command_handler(pool).await;
        let cmd = UpdateRoleCommand::builder()
            .id(RoleId::generate())
            .name("test".to_string())
            .enabled(true)
            .build();
        let result = handler.handle(cmd).await;
        assert_eq!(result.err(), Some(IamError::RoleNotFound));
    }

    #[sqlx::test]
    async fn test_update_role_return_err_given_role_is_privated(pool: PgPool) {
        let handler = build_command_handler(pool).await;
        let role_id = RoleId::generate();
        let role = Role::builder()
            .id(role_id.clone())
            .name("test".to_string())
            .privileged(true)
            .menus(vec![])
            .permissions(vec![])
            .enabled(true)
            .build();
        assert!(handler.role_repository.save(role).await.is_ok());
        let cmd = UpdateRoleCommand::builder()
            .id(role_id)
            .name("test".to_string())
            .enabled(true)
            .menus(vec![])
            .permissions(vec![])
            .build();
        let result = handler.handle(cmd).await;
        assert_eq!(result.err(), Some(IamError::RolePrivilegedImmutable));
    }
}
