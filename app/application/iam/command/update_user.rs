use crate::iam::command::sign_out::{SignOutCommand, SignOutCommandHandler};
use crate::shared::command_handler::{CommandHandler, CommandResult};
use bon::Builder;
use domain::system::error::IamError;
use domain::system::event::IamEvent;
use domain::system::value_object::role_id::RoleId;
use domain::system::{entity::user::User, value_object::user_id::UserId};
use domain::shared::event_util::UpdatedEvent;
use domain::shared::port::domain_repository::DomainRepository;
use infrastructure::repository::system::user_repository::UserRepositoryImpl;
use nject::injectable;
use object_storage_kit::{ObjectStorage, ObjectStorageReader as _};
use serde::Deserialize;
use serde_with::serde_as;
use utoipa::ToSchema;

#[serde_as]
#[derive(Debug, Deserialize, Builder, ToSchema)]
pub struct UpdateUserCommand {
    id: UserId,
    account: Option<String>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    portrait: Option<Option<String>>,
    name: Option<String>,
    role_ids: Option<Vec<RoleId>>,
    enabled: Option<bool>,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct UpdateUserCommandHandler {
    user_repository: UserRepositoryImpl,
    sign_out_command_handler: SignOutCommandHandler,
    object_storage: ObjectStorage,
}

impl CommandHandler for UpdateUserCommandHandler {
    type Command = UpdateUserCommand;
    type Output = User;
    type Event = IamEvent;
    type Error = IamError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let id = cmd.id;
        let mut user = self.user_repository.by_id(&id).await?;
        if user.privileged {
            return Err(IamError::UserPrivilegedImmutable);
        }
        let before = user.clone();
        if let Some(account) = cmd.account {
            user.update_account(account);
        }
        if let Some(portrait) = cmd.portrait {
            let portrait = self.object_storage.purify_url_opt(portrait);
            user.update_portrait(portrait);
        }
        if let Some(name) = cmd.name {
            user.update_name(name);
        }
        if let Some(role_ids) = cmd.role_ids {
            user.update_role_ids(role_ids);
        }
        if let Some(enabled) = cmd.enabled {
            user.update_enabled(enabled);
        }
        let user = self.user_repository.save(user).await?;
        if !user.enabled {
            let command = SignOutCommand::builder().id(id.clone()).build();
            if let Err(err) = self.sign_out_command_handler.handle(command).await {
                tracing::error!(%err, "注销用户失败");
            }
        }
        Ok(CommandResult::with_event(
            user.clone(),
            IamEvent::UsersUpdated {
                items: vec![UpdatedEvent {
                    before,
                    after: user,
                }],
            },
        ))
    }
}
