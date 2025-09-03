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
pub struct UpdateUserPasswordCommand {
    id: UserId,
    new_password: String,
    confirm_new_password: String,
}

#[injectable]
pub struct UpdateUserPasswordCommandHandler {
    user_repository: UserRepositoryImpl,
}

impl CommandHandler for UpdateUserPasswordCommandHandler {
    type Command = UpdateUserPasswordCommand;
    type Output = User;
    type Event = IamEvent;
    type Error = IamError;

    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let new_password = cmd.new_password.trim();
        let confirm_new_password = cmd.confirm_new_password.trim();
        if new_password != confirm_new_password {
            return Err(IamError::TwoPasswordsInconsistent);
        }
        let mut user = self.user_repository.by_id(&cmd.id).await?;
        let before = user.clone();
        if user.privileged {
            return Err(IamError::CannotPrivilegedUserPassword);
        }
        user.assert_activated()?;
        if user.password.verify(new_password).is_ok() {
            return Err(IamError::CannotSameOriginalPassword);
        }
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
