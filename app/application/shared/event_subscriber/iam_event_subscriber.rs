use crate::{
    iam::query::{search_roles::SearchRolesQueryHandler, search_users::SearchUsersQueryHandler},
    shared::event::Event,
    system::service::file_service::FileService,
};
use bon::Builder;
use domain::{iam::event::IamEvent, shared::port::permission_resolver::PermissionResolver};
use infrastructure::{
    port::permission_resolver_impl::PermissionResolverImpl, shared::event_bus::EventSubscriber,
};
use nject::injectable;

#[derive(Clone, Builder)]
#[injectable]
pub struct IamEventSubscriber {
    permission_resolver: PermissionResolverImpl,
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
    async fn on_received(&self, event: Event) -> anyhow::Result<()> {
        #[allow(irrefutable_let_patterns)]
        if let Event::Iam(e) = event {
            if Self::is_permission_changed(&e)
                && let Err(err) = self.permission_resolver.refresh().await
            {
                tracing::error!(?e, error = %err, "权限刷新失败");
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::shared::cache_provider::CacheProvider;

    use super::*;
    use domain::{
        iam::{
            entity::user::User,
            event::IamEvent,
            value_object::{hashed_password::HashedPassword, user_id::UserId},
        },
        shared::event_util::UpdatedEvent,
    };
    use infrastructure::{
        shared::{chrono_tz::ChronoTz, pg_pool::PgPool},
        test_utils::{setup_database, setup_kvdb},
    };
    use sqlx::types::chrono::Utc;

    async fn build_subscriber(pool: PgPool) -> IamEventSubscriber {
        setup_database(pool.clone()).await;
        let kvdb = setup_kvdb().await;
        let permission_resolver = PermissionResolverImpl::builder()
            .pool(pool.clone())
            .kvdb(kvdb.clone())
            .build();
        let search_user_query_handler = {
            let cache_provider = CacheProvider::builder()
                .prefix("iam_search_users:")
                .ttl(Duration::from_secs(15 * 60))
                .kvdb(kvdb.clone())
                .build();
            SearchUsersQueryHandler::builder()
                .pool(pool.clone())
                .cache_provider(cache_provider)
                .build()
        };
        let search_role_query_handler = {
            let cache_provider = CacheProvider::builder()
                .prefix("iam_search_roles:")
                .ttl(Duration::from_secs(15 * 60))
                .kvdb(kvdb)
                .build();
            SearchRolesQueryHandler::builder()
                .pool(pool.clone())
                .cache_provider(cache_provider)
                .build()
        };
        let file_service = FileService::builder()
            .pool(pool)
            .ct(ChronoTz::default())
            .build();
        IamEventSubscriber::builder()
            .permission_resolver(permission_resolver)
            .search_user_query_handler(search_user_query_handler)
            .search_role_query_handler(search_role_query_handler)
            .file_service(file_service)
            .build()
    }

    #[sqlx::test]
    async fn test_on_received(pool: PgPool) {
        let subscriber = build_subscriber(pool).await;
        let user = User::builder()
            .id(UserId::generate())
            .account("test".to_string())
            .name("test".to_string())
            .password(HashedPassword::try_new("123123".to_string()).unwrap())
            .privileged(false)
            .role_ids(vec![])
            .refresh_token_expired_at(Utc::now().naive_utc() + Duration::from_secs(60))
            .refresh_token("test-token".to_string())
            .enabled(true)
            .build();
        assert!(
            subscriber
                .on_received(Event::Iam(IamEvent::UsersCreated {
                    items: vec![user.clone()]
                }))
                .await
                .is_ok()
        );
        assert!(
            subscriber
                .on_received(Event::Iam(IamEvent::UsersDeleted {
                    items: vec![user.clone()]
                }))
                .await
                .is_ok()
        );
        assert!(
            subscriber
                .on_received(Event::Iam(IamEvent::RolesCreated { items: vec![] }))
                .await
                .is_ok()
        );

        // user add portrait
        let mut after_user = user.clone();
        after_user.update_portrait(Some("test-portrait".to_string()));
        assert!(
            subscriber
                .on_received(Event::Iam(IamEvent::UsersUpdated {
                    items: vec![UpdatedEvent {
                        before: user.clone(),
                        after: after_user.clone(),
                    }]
                }))
                .await
                .is_ok()
        );

        // user changed portrait
        let portrait_user = after_user.clone();
        let mut after_user = portrait_user.clone();
        after_user.update_portrait(Some("test-portrait2".to_string()));
        assert!(
            subscriber
                .on_received(Event::Iam(IamEvent::UsersUpdated {
                    items: vec![UpdatedEvent {
                        before: portrait_user.clone(),
                        after: after_user.clone(),
                    }]
                }))
                .await
                .is_ok()
        );
        let portrait_user = after_user.clone();
        let after_user = portrait_user.clone();

        // user portrait not changed
        assert!(
            subscriber
                .on_received(Event::Iam(IamEvent::UsersUpdated {
                    items: vec![UpdatedEvent {
                        before: portrait_user,
                        after: after_user.clone(),
                    }]
                }))
                .await
                .is_ok()
        );

        // user delete portrait
        let portrait_user = after_user.clone();
        let mut after_user = portrait_user.clone();
        after_user.update_portrait(None);
        assert!(
            subscriber
                .on_received(Event::Iam(IamEvent::UsersUpdated {
                    items: vec![UpdatedEvent {
                        before: portrait_user,
                        after: after_user
                    }]
                }))
                .await
                .is_ok()
        );
    }

    #[test]
    fn test_is_users_changed() {
        assert!(IamEventSubscriber::is_users_changed(
            &IamEvent::UsersCreated { items: vec![] }
        ));
        assert!(IamEventSubscriber::is_users_changed(
            &IamEvent::UsersUpdated { items: vec![] }
        ));
        assert!(IamEventSubscriber::is_users_changed(
            &IamEvent::UsersDeleted { items: vec![] }
        ));
        assert!(!IamEventSubscriber::is_users_changed(
            &IamEvent::UserLoginSucceeded {
                id: UserId::generate()
            }
        ));
        assert!(!IamEventSubscriber::is_users_changed(
            &IamEvent::UserLogoutSucceeded {
                id: UserId::generate()
            }
        ));
        assert!(!IamEventSubscriber::is_users_changed(
            &IamEvent::UserRefreshTokenSucceeded {
                id: UserId::generate()
            }
        ));
    }

    #[test]
    fn test_is_roles_changed() {
        assert!(IamEventSubscriber::is_roles_changed(
            &IamEvent::RolesCreated { items: vec![] }
        ));
        assert!(IamEventSubscriber::is_roles_changed(
            &IamEvent::RolesUpdated { items: vec![] }
        ));
        assert!(IamEventSubscriber::is_roles_changed(
            &IamEvent::RolesDeleted { items: vec![] }
        ));
        assert!(!IamEventSubscriber::is_roles_changed(
            &IamEvent::UsersCreated { items: vec![] }
        ));
        assert!(!IamEventSubscriber::is_roles_changed(
            &IamEvent::UsersUpdated { items: vec![] }
        ));
        assert!(!IamEventSubscriber::is_roles_changed(
            &IamEvent::UsersDeleted { items: vec![] }
        ));
    }

    #[test]
    fn test_is_permission_changed() {
        assert!(!IamEventSubscriber::is_permission_changed(
            &IamEvent::RolesCreated { items: vec![] }
        ));
        assert!(IamEventSubscriber::is_permission_changed(
            &IamEvent::RolesUpdated { items: vec![] }
        ));
        assert!(IamEventSubscriber::is_permission_changed(
            &IamEvent::RolesDeleted { items: vec![] }
        ));
        assert!(!IamEventSubscriber::is_permission_changed(
            &IamEvent::UsersCreated { items: vec![] }
        ));
        assert!(IamEventSubscriber::is_permission_changed(
            &IamEvent::UsersUpdated { items: vec![] }
        ));
        assert!(IamEventSubscriber::is_permission_changed(
            &IamEvent::UsersDeleted { items: vec![] }
        ));
    }
}
