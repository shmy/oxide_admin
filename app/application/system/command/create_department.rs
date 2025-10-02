use bon::Builder;
use domain::shared::port::domain_repository::DomainRepository;
use domain::system::entity::department::Department;
use domain::system::error::SystemError;
use domain::system::event::SystemEvent;
use domain::system::value_object::department_id::DepartmentId;
use infrastructure::repository::system::department_repository::DepartmentRepositoryImpl;
use nject::injectable;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct CreateDepartmentCommand {
    name: String,
    code: String,
    parent_id: Option<String>,
    enabled: bool,
}

#[derive(Debug)]
#[injectable]
pub struct CreateDepartmentCommandHandler {
    department_repo: DepartmentRepositoryImpl,
}

impl CommandHandler for CreateDepartmentCommandHandler {
    type Command = CreateDepartmentCommand;
    type Output = Department;
    type Event = SystemEvent;
    type Error = SystemError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let department = Department::builder()
            .id(DepartmentId::generate())
            .name(cmd.name)
            .code(cmd.code)
            .maybe_parent_id(cmd.parent_id)
            .enabled(cmd.enabled)
            .build();
        let department = self.department_repo.save(department).await?;
        Ok(CommandResult::with_event(
            department.clone(),
            SystemEvent::DepartmentsCreated {
                items: vec![department],
            },
        ))
    }
}
