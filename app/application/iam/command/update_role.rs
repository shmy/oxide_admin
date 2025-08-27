use anyhow::Result;
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::value_object::permission_code::PermissionCode;
use domain::iam::value_object::role_id::RoleId;
use domain::iam::{entity::role::Role, event::IamEvent};
use domain::shared::domain_repository::DomainRepository;
use domain::shared::event_util::UpdatedEvent;
use infrastructure::repository::iam::role_repository::RoleRepositoryImpl;
use nject::injectable;
use serde::Deserialize;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Deserialize, Builder)]
pub struct UpdateRoleCommand {
    id: RoleId,
    name: Option<String>,
    permission_ids: Option<Vec<PermissionCode>>,
    enabled: Option<bool>,
}

#[injectable]
pub struct UpdateRoleCommandHandler {
    role_repo: RoleRepositoryImpl,
}

impl CommandHandler for UpdateRoleCommandHandler {
    type Command = UpdateRoleCommand;
    type Output = Role;
    type Event = IamEvent;
    type Error = IamError;

    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let id = cmd.id;
        let mut role = self.role_repo.by_id(&id).await?;
        let before = role.clone();
        if let Some(name) = cmd.name {
            role.update_name(name);
        }
        if let Some(permission_ids) = cmd.permission_ids {
            role.update_permission_ids(permission_ids);
        }
        if let Some(enabled) = cmd.enabled {
            role.update_enabled(enabled);
        }
        let role = self.role_repo.save(role).await?;
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
