use std::time::Duration;

use crate::shared::command_handler::{CommandHandler, CommandResult};
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::event::IamEvent;
use domain::shared::port::captcha_issuer::{Captcha, CaptchaIssuerTrait as _};
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
    type Error = IamError;
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

#[cfg(test)]
mod tests {
    use infrastructure::test_utils::setup_kvdb;

    use super::*;
    async fn build_command_handler() -> RefreshCaptchaCommandHandler {
        let kvdb = setup_kvdb().await;
        let captcha_issuer = CaptchaIssuerImpl::builder().kvdb(kvdb).build();
        RefreshCaptchaCommandHandler::builder()
            .captcha_issuer(captcha_issuer)
            .build()
    }

    #[tokio::test]
    async fn test_refresh_captcha() {
        let command_handler = build_command_handler().await;
        assert!(
            command_handler
                .handle(RefreshCaptchaCommand {})
                .await
                .is_ok()
        );
    }
}
