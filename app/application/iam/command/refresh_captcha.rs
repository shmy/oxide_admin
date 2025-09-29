use std::time::Duration;

use crate::shared::command_handler::{CommandHandler, CommandResult};
use bon::Builder;
use domain::shared::port::captcha_issuer::{Captcha, CaptchaIssuerTrait as _};
use domain::system::error::SystemError;
use domain::system::event::IamEvent;
use infrastructure::port::captcha_issuer_impl::CaptchaIssuerImpl;
use nject::injectable;
use serde::Deserialize;
#[derive(Debug, Deserialize, Builder)]
pub struct RefreshCaptchaCommand {}

#[derive(Debug, Builder)]
#[injectable]
pub struct RefreshCaptchaCommandHandler {
    captcha_issuer: CaptchaIssuerImpl,
}

impl CommandHandler for RefreshCaptchaCommandHandler {
    type Command = RefreshCaptchaCommand;
    type Output = Captcha;
    type Event = IamEvent;
    type Error = SystemError;
    #[tracing::instrument]
    async fn execute(
        &self,
        _cmd: Self::Command,
    ) -> Result<CommandResult<Self::Output, Self::Event>, Self::Error> {
        let output = self
            .captcha_issuer
            .generate_with_ttl(Duration::from_secs(60))
            .await?;
        Ok(CommandResult::without_events(output))
    }
}
