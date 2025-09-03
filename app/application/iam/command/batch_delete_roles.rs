use anyhow::Result;
use bon::Builder;
use domain::{
    iam::{error::IamError, event::IamEvent, value_object::role_id::RoleId},
    shared::port::domain_repository::DomainRepository,
};
use infrastructure::repository::iam::role_repository::RoleRepositoryImpl;
use nject::injectable;
use serde::Deserialize;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Deserialize, Builder)]
pub struct BatchDeleteRolesCommand {
    ids: Vec<RoleId>,
}

#[injectable]
pub struct BatchDeleteRolesCommandHandler {
    role_repo: RoleRepositoryImpl,
}

impl CommandHandler for BatchDeleteRolesCommandHandler {
    type Command = BatchDeleteRolesCommand;
    type Output = ();
    type Event = IamEvent;
    type Error = IamError;

    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let items = self.role_repo.batch_delete(&cmd.ids).await?;
        Ok(CommandResult::with_event(
            (),
            IamEvent::RolesDeleted { items },
        ))
    }
}
