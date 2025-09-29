use crate::shared::command_handler::{CommandHandler, CommandResult};
use bon::Builder;
use domain::shared::event_util::UpdatedEvent;
use domain::shared::port::domain_repository::DomainRepository;
use domain::system::error::SystemError;
use domain::system::event::SystemEvent;
use domain::system::{entity::user::User, value_object::user_id::UserId};
use infrastructure::repository::system::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;

#[derive(Debug, Deserialize, Builder)]
pub struct UpdateUserPasswordCommand {
    id: UserId,
    new_password: String,
    confirm_new_password: String,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct UpdateUserPasswordCommandHandler {
    user_repository: UserRepositoryImpl,
}

impl CommandHandler for UpdateUserPasswordCommandHandler {
    type Command = UpdateUserPasswordCommand;
    type Output = User;
    type Event = SystemEvent;
    type Error = SystemError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let new_password = cmd.new_password.trim();
        let confirm_new_password = cmd.confirm_new_password.trim();
        if new_password != confirm_new_password {
            return Err(SystemError::PasswordMismatch);
        }
        let mut user = self.user_repository.by_id(&cmd.id).await?;
        if user.privileged {
            return Err(SystemError::UserPrivilegedImmutable);
        }
        let before = user.clone();
        user.assert_activated()?;
        if user.password.verify(new_password).is_ok() {
            return Err(SystemError::PasswordUnchanged);
        }
        user.update_password(new_password.to_string())?;

        let user = self.user_repository.save(user).await?;
        Ok(CommandResult::with_event(
            user.clone(),
            SystemEvent::UsersUpdated {
                items: vec![UpdatedEvent {
                    before,
                    after: user,
                }],
            },
        ))
    }
}
