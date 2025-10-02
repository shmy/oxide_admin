use bon::Builder;
use domain::organization::error::OrganizationError;
use domain::organization::value_object::department_id::DepartmentId;
use domain::organization::{entity::department::Department, event::OrganizationEvent};
use domain::shared::event_util::UpdatedEvent;
use domain::shared::port::domain_repository::DomainRepository;
use infrastructure::repository::organization::department_repository::DepartmentRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct UpdateDepartmentCommand {
    id: DepartmentId,
    name: Option<String>,
    code: Option<String>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    parent_id: Option<Option<String>>,
    enabled: Option<bool>,
}

#[derive(Debug)]
#[injectable]
pub struct UpdateDepartmentCommandHandler {
    department_repo: DepartmentRepositoryImpl,
}

impl CommandHandler for UpdateDepartmentCommandHandler {
    type Command = UpdateDepartmentCommand;
    type Output = Department;
    type Event = OrganizationEvent;
    type Error = OrganizationError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let id = cmd.id;
        let mut department = self.department_repo.by_id(&id).await?;
        let before = department.clone();
        if let Some(name) = cmd.name {
            department.update_name(name);
        }
        if let Some(code) = cmd.code {
            department.update_code(code);
        }
        if let Some(parent_id) = cmd.parent_id {
            department.update_parent_id(parent_id);
        }
        if let Some(enabled) = cmd.enabled {
            department.update_enabled(enabled);
        }
        let department = self.department_repo.save(department).await?;
        Ok(CommandResult::with_event(
            department.clone(),
            OrganizationEvent::DepartmentsUpdated {
                items: vec![UpdatedEvent {
                    before,
                    after: department,
                }],
            },
        ))
    }
}
