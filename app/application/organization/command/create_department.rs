use bon::Builder;
use domain::organization::entity::department::Department;
use domain::organization::error::OrganizationError;
use domain::organization::event::OrganizationEvent;
use domain::organization::value_object::department_id::DepartmentId;
use domain::shared::port::domain_repository::DomainRepository;
use infrastructure::repository::organization::department_repository::DepartmentRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct CreateDepartmentCommand {
    name: String,
    code: String,
    parent: Option<CreateDepartmentParent>,
}

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct CreateDepartmentParent {
    value: String,
}

#[derive(Debug)]
#[injectable]
pub struct CreateDepartmentCommandHandler {
    department_repo: DepartmentRepositoryImpl,
}

impl CommandHandler for CreateDepartmentCommandHandler {
    type Command = CreateDepartmentCommand;
    type Output = Department;
    type Event = OrganizationEvent;
    type Error = OrganizationError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let department = Department::builder()
            .id(DepartmentId::generate())
            .name(cmd.name)
            .code(cmd.code)
            .maybe_parent_code(cmd.parent.map(|parent| parent.value))
            .build();
        let department = self.department_repo.save(department).await?;
        Ok(CommandResult::with_event(
            department.clone(),
            OrganizationEvent::DepartmentsCreated {
                items: vec![department],
            },
        ))
    }
}
