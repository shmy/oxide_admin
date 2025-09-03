use anyhow::Result;
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::event::IamEvent;
use domain::iam::value_object::permission_code::PermissionCode;
use domain::iam::{entity::role::Role, value_object::role_id::RoleId};
use domain::shared::port::domain_repository::DomainRepository;
use infrastructure::repository::iam::role_repository::RoleRepositoryImpl;
use nject::injectable;
use serde::Deserialize;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Deserialize, Builder)]
pub struct CreateRoleCommand {
    name: String,
    permission_ids: Vec<PermissionCode>,
    enabled: bool,
}

#[injectable]
pub struct CreateRoleCommandHandler {
    role_repo: RoleRepositoryImpl,
}

impl CommandHandler for CreateRoleCommandHandler {
    type Command = CreateRoleCommand;
    type Output = Role;
    type Event = IamEvent;
    type Error = IamError;

    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let role = Role::builder()
            .id(RoleId::generate())
            .name(cmd.name)
            .privileged(false)
            .permission_ids(cmd.permission_ids)
            .enabled(cmd.enabled)
            .build();
        let role = self.role_repo.save(role).await?;
        Ok(CommandResult::with_event(
            role.clone(),
            IamEvent::RolesCreated { items: vec![role] },
        ))
    }
}
