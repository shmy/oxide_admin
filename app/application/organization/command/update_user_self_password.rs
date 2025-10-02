use crate::shared::command_handler::{CommandHandler, CommandResult};
use bon::Builder;
use domain::organization::error::OrganizationError;
use domain::organization::event::OrganizationEvent;
use domain::organization::{entity::user::User, value_object::user_id::UserId};
use domain::shared::event_util::UpdatedEvent;
use domain::shared::port::domain_repository::DomainRepository;
use infrastructure::repository::organization::user_repository::UserRepositoryImpl;
use nject::injectable;
use serde::Deserialize;

#[derive(Debug, Deserialize, Builder)]
pub struct UpdateUserSelfPasswordCommand {
    id: UserId,
    password: String,
    new_password: String,
    confirm_new_password: String,
}

#[derive(Debug, Builder)]
#[injectable]
pub struct UpdateUserSelfPasswordCommandHandler {
    user_repository: UserRepositoryImpl,
}

impl CommandHandler for UpdateUserSelfPasswordCommandHandler {
    type Command = UpdateUserSelfPasswordCommand;
    type Output = User;
    type Event = OrganizationEvent;
    type Error = OrganizationError;

    #[tracing::instrument]
    async fn execute(
        &self,
        cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let password = cmd.password.trim();
        let new_password = cmd.new_password.trim();
        let confirm_new_password = cmd.confirm_new_password.trim();
        if new_password != confirm_new_password {
            return Err(OrganizationError::PasswordMismatch);
        }
        if new_password == password {
            return Err(OrganizationError::PasswordUnchanged);
        }
        let mut user = self.user_repository.by_id(&cmd.id).await?;
        let before = user.clone();
        user.assert_activated()?;
        user.password.verify(password)?;
        user.update_password(new_password.to_string())?;

        let user = self.user_repository.save(user).await?;
        Ok(CommandResult::with_event(
            user.clone(),
            OrganizationEvent::UsersUpdated {
                items: vec![UpdatedEvent {
                    before,
                    after: user,
                }],
            },
        ))
    }
}
