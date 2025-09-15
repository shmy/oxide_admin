use anyhow::Result;
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::event::IamEvent;
use domain::iam::value_object::role_id::RoleId;
use domain::iam::{
    entity::user::User, value_object::hashed_password::HashedPassword,
    value_object::user_id::UserId,
};
use domain::shared::port::domain_repository::DomainRepository;
use infrastructure::repository::iam::user_repository::UserRepositoryImpl;
use nject::injectable;
use object_storage_kit::{ObjectStorage, ObjectStorageReader as _};
use serde::Deserialize;

use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Debug, Deserialize, Builder)]
pub struct CreateUserCommand {
    account: String,
    password: String,
    portrait: Option<String>,
    name: String,
    role_ids: Vec<RoleId>,
    enabled: bool,
}

#[derive(Debug)]
#[injectable]
pub struct CreateUserCommandHandler {
    user_repository: UserRepositoryImpl,
    object_storage: ObjectStorage,
}

impl CommandHandler for CreateUserCommandHandler {
    type Command = CreateUserCommand;
    type Output = User;
    type Event = IamEvent;
    type Error = IamError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let password = HashedPassword::try_new(cmd.password)?;
        let user = User::builder()
            .id(UserId::generate())
            .account(cmd.account)
            .maybe_portrait(self.object_storage.purify_url_opt(cmd.portrait))
            .name(cmd.name)
            .password(password)
            .privileged(false)
            .role_ids(cmd.role_ids)
            .enabled(cmd.enabled)
            .build();
        let user = self.user_repository.save(user).await?;
        Ok(CommandResult::with_event(
            user.clone(),
            IamEvent::UsersCreated { items: vec![user] },
        ))
    }
}
