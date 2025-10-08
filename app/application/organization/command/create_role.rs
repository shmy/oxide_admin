use bon::Builder;
use domain::auth::value_object::menu::Menu;
use domain::auth::value_object::permission::Permission;
use domain::organization::event::OrganizationEvent;
use domain::organization::{entity::role::Role, value_object::role_id::RoleId};
use domain::shared::port::domain_repository::DomainRepository;
use infrastructure::repository::organization::role_repository::RoleRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::error::ApplicationError;
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
    type Event = OrganizationEvent;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, ApplicationError> {
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
            OrganizationEvent::RolesCreated { items: vec![role] },
        ))
    }
}
