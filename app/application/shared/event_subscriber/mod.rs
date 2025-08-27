use crate::{
    iam::service::{role_service::RoleService, user_service::UserService},
    shared::{
        event::EVENT_BUS,
        event_subscriber::{
            iam_event_subscriber::IamEventSubscriber, log_event_subscriber::LogEventSubscriber,
        },
    },
    system::service::file_service::FileService,
};
use anyhow::Result;
use infrastructure::{
    implementation::permission_resolver_impl::PermissionResolverImpl, shared::provider::Provider,
};

pub mod iam_event_subscriber;
pub mod log_event_subscriber;

pub async fn register_subscribers(provider: &Provider) -> Result<()> {
    let permission_resolver = provider.provide::<PermissionResolverImpl>();
    let user_service = provider.provide::<UserService>();
    let role_service = provider.provide::<RoleService>();
    let file_service = provider.provide::<FileService>();
    EVENT_BUS.subscribe(LogEventSubscriber);
    EVENT_BUS.subscribe(IamEventSubscriber::new(
        permission_resolver,
        user_service,
        role_service,
        file_service,
    ));

    Ok(())
}
