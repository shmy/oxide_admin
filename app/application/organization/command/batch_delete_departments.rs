use bon::Builder;
use domain::{
    organization::{event::OrganizationEvent, value_object::department_id::DepartmentId},
    shared::port::domain_repository::DomainRepository,
};
use infrastructure::repository::organization::department_repository::DepartmentRepositoryImpl;
use nject::injectable;
use serde::Deserialize;

use crate::{
    error::ApplicationError,
    shared::command_handler::{CommandHandler, CommandResult},
};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct BatchDeleteDepartmentsCommand {
    ids: Vec<DepartmentId>,
}

#[derive(Debug)]
#[injectable]
pub struct BatchDeleteDepartmentsCommandHandler {
    department_repo: DepartmentRepositoryImpl,
}

impl CommandHandler for BatchDeleteDepartmentsCommandHandler {
    type Command = BatchDeleteDepartmentsCommand;
    type Output = ();
    type Event = OrganizationEvent;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, ApplicationError> {
        let items = self.department_repo.batch_delete(&cmd.ids).await?;
        Ok(CommandResult::with_event(
            (),
            OrganizationEvent::DepartmentsDeleted { items },
        ))
    }
}
