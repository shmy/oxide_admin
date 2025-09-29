use crate::{
    iam::query::{search_roles::SearchRolesQueryHandler, search_users::SearchUsersQueryHandler},
    shared::event::Event,
    system::service::file_service::FileService,
};
use bon::Builder;
use domain::shared::port::menu_resolver::MenuResolver;
use domain::{system::event::IamEvent, shared::port::permission_resolver::PermissionResolver};
use infrastructure::{error::InfrastructureResult, port::menu_resolver_impl::MenuResolverImpl};
use infrastructure::{
    port::permission_resolver_impl::PermissionResolverImpl, shared::event_bus::EventSubscriber,
};
use nject::injectable;

#[derive(Clone, Builder)]
#[injectable]
pub struct IamEventSubscriber {
    permission_resolver: PermissionResolverImpl,
    menu_resolver: MenuResolverImpl,
    search_user_query_handler: SearchUsersQueryHandler,
    search_role_query_handler: SearchRolesQueryHandler,
    file_service: FileService,
}

impl IamEventSubscriber {
    fn is_users_changed(event: &IamEvent) -> bool {
        matches!(
            event,
            IamEvent::UsersCreated { .. }
                | IamEvent::UsersUpdated { .. }
                | IamEvent::UsersDeleted { .. }
        )
    }

    fn is_roles_changed(event: &IamEvent) -> bool {
        matches!(
            event,
            IamEvent::RolesCreated { .. }
                | IamEvent::RolesUpdated { .. }
                | IamEvent::RolesDeleted { .. }
        )
    }
    fn is_permission_changed(event: &IamEvent) -> bool {
        matches!(
            event,
            IamEvent::UsersUpdated { .. }
                | IamEvent::UsersDeleted { .. }
                | IamEvent::RolesUpdated { .. }
                | IamEvent::RolesDeleted { .. }
        )
    }
}

impl EventSubscriber<Event> for IamEventSubscriber {
    async fn on_received(&self, event: Event) -> InfrastructureResult<()> {
        #[allow(irrefutable_let_patterns)]
        if let Event::Iam(e) = event {
            if Self::is_permission_changed(&e) {
                if let Err(err) = self.permission_resolver.refresh().await {
                    tracing::error!(?e, error = %err, "权限刷新失败");
                }
                if let Err(err) = self.menu_resolver.refresh().await {
                    tracing::error!(?e, error = %err, "菜单刷新失败");
                }
            }
            if Self::is_users_changed(&e) {
                let _ = self.search_user_query_handler.clean_cache().await;
            }
            if Self::is_roles_changed(&e) {
                let _ = self.search_role_query_handler.clean_cache().await;
            }
            match e {
                IamEvent::UsersCreated { items } => {
                    let paths = items
                        .into_iter()
                        .filter_map(|item| item.portrait.clone())
                        .collect::<Vec<_>>();
                    if let Err(err) = self.file_service.set_files_used(&paths).await {
                        tracing::error!(error = %err, "UsersCreated: failed to set files used");
                    }
                }
                IamEvent::UsersUpdated { items } => {
                    let (mut unused_paths, mut used_paths) = (Vec::new(), Vec::new());

                    for item in items {
                        match (&item.before.portrait, &item.after.portrait) {
                            (Some(before), Some(after)) if before != after => {
                                // 用户修改了头像
                                unused_paths.push(before.clone());
                                used_paths.push(after.clone());
                            }
                            (Some(before), None) => {
                                unused_paths.push(before.clone()); // 用户删除了头像
                            }
                            (None, Some(after)) => {
                                used_paths.push(after.clone()); // 用户新增了头像
                            }
                            _ => {} // 没有变化的情况，不操作
                        }
                    }
                    if let Err(err) = self.file_service.set_files_unused(&unused_paths).await {
                        tracing::error!(error = %err, "UsersUpdated: failed to set files unused");
                    }
                    if let Err(err) = self.file_service.set_files_used(&used_paths).await {
                        tracing::error!(error = %err, "UsersUpdated: failed to set files used");
                    }
                }
                IamEvent::UsersDeleted { items } => {
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
