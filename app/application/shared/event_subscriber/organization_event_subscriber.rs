use crate::{
    organization::query::{
        search_roles::SearchRolesQueryHandler, search_users::SearchUsersQueryHandler,
    },
    shared::event::Event,
    system::service::file_service::FileService,
};
use bon::Builder;
use domain::auth::port::menu_resolver::MenuResolver;
use domain::{
    auth::port::permission_resolver::PermissionResolver, organization::event::OrganizationEvent,
};
use event_kit::{EventSubscriber, error::Result};
use infrastructure::port::menu_resolver_impl::MenuResolverImpl;
use infrastructure::port::permission_resolver_impl::PermissionResolverImpl;
use nject::injectable;

#[derive(Clone, Builder)]
#[injectable]
pub struct OrganizationEventSubscriber {
    permission_resolver: PermissionResolverImpl,
    menu_resolver: MenuResolverImpl,
    search_user_query_handler: SearchUsersQueryHandler,
    search_role_query_handler: SearchRolesQueryHandler,
    file_service: FileService,
}

impl OrganizationEventSubscriber {
    fn is_users_changed(event: &OrganizationEvent) -> bool {
        matches!(
            event,
            OrganizationEvent::UsersCreated { .. }
                | OrganizationEvent::UsersUpdated { .. }
                | OrganizationEvent::UsersDeleted { .. }
        )
    }
    fn is_roles_changed(event: &OrganizationEvent) -> bool {
        matches!(
            event,
            OrganizationEvent::RolesCreated { .. }
                | OrganizationEvent::RolesUpdated { .. }
                | OrganizationEvent::RolesDeleted { .. }
        )
    }
    fn is_permission_changed(event: &OrganizationEvent) -> bool {
        matches!(
            event,
            OrganizationEvent::UsersUpdated { .. }
                | OrganizationEvent::UsersDeleted { .. }
                | OrganizationEvent::RolesUpdated { .. }
                | OrganizationEvent::RolesDeleted { .. }
        )
    }
}

impl EventSubscriber<Event> for OrganizationEventSubscriber {
    async fn on_received(&self, event: Event) -> Result<()> {
        if let Event::Organization(e) = event {
            if Self::is_users_changed(&e) {
                let _ = self.search_user_query_handler.clean_cache().await;
            }
            if Self::is_roles_changed(&e) {
                let _ = self.search_role_query_handler.clean_cache().await;
            }
            if Self::is_permission_changed(&e) {
                if let Err(err) = self.permission_resolver.refresh().await {
                    tracing::error!(?e, error = %err, "Failed to refresh permissions");
                }
                if let Err(err) = self.menu_resolver.refresh().await {
                    tracing::error!(?e, error = %err, "Failed to refresh menus");
                }
            }
            match e {
                OrganizationEvent::UsersCreated { items } => {
                    let paths = items
                        .into_iter()
                        .filter_map(|item| item.portrait.clone())
                        .collect::<Vec<_>>();
                    if let Err(err) = self.file_service.set_files_used(&paths).await {
                        tracing::error!(error = %err, "UsersCreated: failed to set files used");
                    }
                }
                OrganizationEvent::UsersUpdated { items } => {
                    let (mut unused_paths, mut used_paths) = (Vec::new(), Vec::new());

                    for item in items {
                        match (&item.before.portrait, &item.after.portrait) {
                            (Some(before), Some(after)) if before != after => {
                                // User changed portrait
                                unused_paths.push(before.clone());
                                used_paths.push(after.clone());
                            }
                            (Some(before), None) => {
                                unused_paths.push(before.clone()); // User deleted portrait
                            }
                            (None, Some(after)) => {
                                used_paths.push(after.clone()); // User added portrait
                            }
                            _ => {} // No change
                        }
                    }
                    if let Err(err) = self.file_service.set_files_unused(&unused_paths).await {
                        tracing::error!(error = %err, "UsersUpdated: failed to set files unused");
                    }
                    if let Err(err) = self.file_service.set_files_used(&used_paths).await {
                        tracing::error!(error = %err, "UsersUpdated: failed to set files used");
                    }
                }
                OrganizationEvent::UsersDeleted { items } => {
                    let paths = items
                        .into_iter()
                        .filter_map(|item| item.portrait.clone())
                        .collect::<Vec<_>>();
                    if let Err(err) = self.file_service.set_files_unused(&paths).await {
                        tracing::error!(error = %err, "UsersDeleted: failed to set files unused");
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
