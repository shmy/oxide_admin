use std::time::Duration;

use crate::iam::service::iam_service::{Captcha, IamService};
use crate::shared::command_handler::{CommandHandler, CommandResult};
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::event::IamEvent;
use nject::injectable;
use serde::Deserialize;

#[derive(Deserialize, Builder)]
pub struct RefreshCaptchaCommand {}

#[injectable]
pub struct RefreshCaptchaCommandHandler {
    service: IamService,
}

impl CommandHandler for RefreshCaptchaCommandHandler {
    type Command = RefreshCaptchaCommand;
    type Output = Captcha;
    type Event = IamEvent;
    type Error = IamError;
    async fn execute(
        &self,
        _cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let output = self
            .service
            .generate_captcha_with_ttl(Duration::from_secs(60))
            .await
            .map_err(|_| IamError::CaptchaFailedGenerate)?;
        Ok(CommandResult::without_events(output))
    }
}
