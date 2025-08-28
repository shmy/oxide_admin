use crate::{
    iam::query::{search_roles::SearchRolesQueryHandler, search_users::SearchUsersQueryHandler},
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
    let search_user_query_handler = provider.provide::<SearchUsersQueryHandler>();
    let search_role_query_handler = provider.provide::<SearchRolesQueryHandler>();
    let file_service = provider.provide::<FileService>();
    EVENT_BUS.subscribe(LogEventSubscriber);
    EVENT_BUS.subscribe(IamEventSubscriber::new(
        permission_resolver,
        search_user_query_handler,
        search_role_query_handler,
        file_service,
    ));

    Ok(())
}
