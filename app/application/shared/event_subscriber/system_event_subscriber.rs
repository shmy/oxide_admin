use crate::{
    shared::event::Event,
    system::query::{search_roles::SearchRolesQueryHandler, search_users::SearchUsersQueryHandler},
    system::service::file_service::FileService,
};
use bon::Builder;
use domain::shared::port::menu_resolver::MenuResolver;
use domain::{shared::port::permission_resolver::PermissionResolver, system::event::SystemEvent};
use infrastructure::{error::InfrastructureResult, port::menu_resolver_impl::MenuResolverImpl};
use infrastructure::{
    port::permission_resolver_impl::PermissionResolverImpl, shared::event_bus::EventSubscriber,
};
use nject::injectable;

#[derive(Clone, Builder)]
#[injectable]
pub struct SystemEventSubscriber {
    permission_resolver: PermissionResolverImpl,
    menu_resolver: MenuResolverImpl,
    search_user_query_handler: SearchUsersQueryHandler,
    search_role_query_handler: SearchRolesQueryHandler,
    file_service: FileService,
}

impl SystemEventSubscriber {
    fn is_users_changed(event: &SystemEvent) -> bool {
        matches!(
            event,
            SystemEvent::UsersCreated { .. }
                | SystemEvent::UsersUpdated { .. }
                | SystemEvent::UsersDeleted { .. }
        )
    }

    fn is_roles_changed(event: &SystemEvent) -> bool {
        matches!(
            event,
            SystemEvent::RolesCreated { .. }
                | SystemEvent::RolesUpdated { .. }
                | SystemEvent::RolesDeleted { .. }
        )
    }
    fn is_permission_changed(event: &SystemEvent) -> bool {
        matches!(
            event,
            SystemEvent::UsersUpdated { .. }
                | SystemEvent::UsersDeleted { .. }
                | SystemEvent::RolesUpdated { .. }
                | SystemEvent::RolesDeleted { .. }
        )
    }
}

impl EventSubscriber<Event> for SystemEventSubscriber {
    async fn on_received(&self, event: Event) -> InfrastructureResult<()> {
        #[allow(irrefutable_let_patterns)]
        if let Event::System(e) = event {
            if Self::is_permission_changed(&e) {
                if let Err(err) = self.permission_resolver.refresh().await {
                    tracing::error!(?e, error = %err, "Failed to refresh permissions");
                }
                if let Err(err) = self.menu_resolver.refresh().await {
                    tracing::error!(?e, error = %err, "Failed to refresh menus");
                }
            }
            if Self::is_users_changed(&e) {
                let _ = self.search_user_query_handler.clean_cache().await;
            }
            if Self::is_roles_changed(&e) {
                let _ = self.search_role_query_handler.clean_cache().await;
            }
            match e {
                SystemEvent::UsersCreated { items } => {
                    let paths = items
                        .into_iter()
                        .filter_map(|item| item.portrait.clone())
                        .collect::<Vec<_>>();
                    if let Err(err) = self.file_service.set_files_used(&paths).await {
                        tracing::error!(error = %err, "UsersCreated: failed to set files used");
                    }
                }
                SystemEvent::UsersUpdated { items } => {
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
                SystemEvent::UsersDeleted { items } => {
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
