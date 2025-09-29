use bon::Builder;
use domain::shared::port::domain_repository::DomainRepository;
use domain::system::error::SystemError;
use domain::system::event::SystemEvent;
use domain::system::value_object::menu::Menu;
use domain::system::value_object::permission::Permission;
use domain::system::{entity::role::Role, value_object::role_id::RoleId};
use infrastructure::repository::system::role_repository::RoleRepositoryImpl;
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
    type Event = SystemEvent;
    type Error = SystemError;

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
            SystemEvent::RolesCreated { items: vec![role] },
        ))
    }
}
