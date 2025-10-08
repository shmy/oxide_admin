use bon::Builder;
use domain::organization::event::OrganizationEvent;
use domain::organization::value_object::role_id::RoleId;
use domain::organization::{
    entity::user::User, value_object::hashed_password::HashedPassword,
    value_object::user_id::UserId,
};
use domain::shared::port::domain_repository::DomainRepository;
use infrastructure::repository::organization::user_repository::UserRepositoryImpl;
use nject::injectable;
use object_storage_kit::{ObjectStorage, ObjectStorageReader as _};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::error::ApplicationError;
use crate::shared::command_handler::{CommandHandler, CommandResult};

#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct CreateUserCommand {
    account: String,
    password: String,
    portrait: Option<String>,
    name: String,
    role_ids: Vec<RoleId>,
    enabled: bool,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct CreateUserCommandHandler {
    user_repository: UserRepositoryImpl,
    object_storage: ObjectStorage,
}

impl CommandHandler for CreateUserCommandHandler {
    type Command = CreateUserCommand;
    type Output = User;
    type Event = OrganizationEvent;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, ApplicationError> {
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
            OrganizationEvent::UsersCreated { items: vec![user] },
        ))
    }
}
