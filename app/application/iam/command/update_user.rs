use crate::iam::command::sign_out::{SignOutCommand, SignOutCommandHandler};
use crate::shared::command_handler::{CommandHandler, CommandResult};
use anyhow::Result;
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::event::IamEvent;
use domain::iam::value_object::role_id::RoleId;
use domain::iam::{entity::user::User, value_object::user_id::UserId};
use domain::shared::domain_repository::DomainRepository;
use domain::shared::event_util::UpdatedEvent;
use infrastructure::repository::iam::user_repository::UserRepositoryImpl;
use infrastructure::shared::hmac_util::HmacUtil;
use nject::injectable;
use serde::Deserialize;

#[derive(Deserialize, Builder)]
pub struct UpdateUserCommand {
    id: UserId,
    account: Option<String>,
    portrait: Option<Option<String>>,
    name: Option<String>,
    role_ids: Option<Vec<RoleId>>,
    enabled: Option<bool>,
}

#[injectable]
pub struct UpdateUserCommandHandler {
    user_repository: UserRepositoryImpl,
    sign_out_command_handler: SignOutCommandHandler,
    hmac_util: HmacUtil,
}

impl CommandHandler for UpdateUserCommandHandler {
    type Command = UpdateUserCommand;
    type Output = User;
    type Event = IamEvent;
    type Error = IamError;

    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let id = cmd.id;
        let mut user = self.user_repository.by_id(&id).await?;
        let before = user.clone();
        if let Some(account) = cmd.account {
            user.update_account(account);
        }
        if let Some(portrait) = cmd.portrait {
            let portrait = self.hmac_util.strip_query_opt(portrait);
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
                tracing::error!(?err, "注销用户失败");
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
