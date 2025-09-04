use crate::shared::command_handler::{CommandHandler, CommandResult};
use anyhow::Result;
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::event::IamEvent;
use domain::iam::{entity::user::User, value_object::user_id::UserId};
use domain::shared::event_util::UpdatedEvent;
use domain::shared::port::domain_repository::DomainRepository;
use infrastructure::repository::iam::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;

#[derive(Deserialize, Builder)]
pub struct UpdateUserSelfPasswordCommand {
    id: UserId,
    password: String,
    new_password: String,
    confirm_new_password: String,
}

#[injectable]
pub struct UpdateUserSelfPasswordCommandHandler {
    user_repository: UserRepositoryImpl,
}

impl CommandHandler for UpdateUserSelfPasswordCommandHandler {
    type Command = UpdateUserSelfPasswordCommand;
    type Output = User;
    type Event = IamEvent;
    type Error = IamError;

    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let password = cmd.password.trim();
        let new_password = cmd.new_password.trim();
        let confirm_new_password = cmd.confirm_new_password.trim();
        if new_password != confirm_new_password {
            return Err(IamError::PasswordMismatch);
        }
        if new_password == password {
            return Err(IamError::PasswordUnchanged);
        }
        let mut user = self.user_repository.by_id(&cmd.id).await?;
        let before = user.clone();
        user.assert_activated()?;
        user.password.verify(password)?;
        user.update_password(new_password.to_string())?;

        let user = self.user_repository.save(user).await?;
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
