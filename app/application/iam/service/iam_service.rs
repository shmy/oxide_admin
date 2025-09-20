use crate::iam::dto::user::UserDto;
use crate::iam::service::page::{PAGES, Page, SHARED_PAGES};
use crate::system::service::upload_service::UploadService;
use anyhow::{Result, bail};
use bon::Builder;
use domain::iam::value_object::permission_code::{ALL_PERMISSIONS, NONE, PermissionCode};
use domain::iam::value_object::permission_group::{PermissionChecker, PermissionGroup};
use domain::iam::value_object::user_id::UserId;
use domain::shared::port::permission_resolver::PermissionResolver;
use domain::shared::port::token_issuer::{TokenIssuerTrait, UserClaims};
use domain::shared::port::token_store::TokenStoreTrait;
use futures_util::{StreamExt, stream};
use infrastructure::port::permission_resolver_impl::PermissionResolverImpl;
use infrastructure::port::token_issuer_impl::TokenIssuerImpl;
use infrastructure::port::token_store_impl::TokenStoreImpl;
use infrastructure::shared::config::ConfigRef;
use nject::injectable;

#[derive(Debug, Clone, Builder)]
#[injectable]
pub struct IamService {
    token_issuer: TokenIssuerImpl,
    token_store: TokenStoreImpl,
    permission_resolver: PermissionResolverImpl,
    config: ConfigRef,
    upload_service: UploadService,
}

impl IamService {
    #[tracing::instrument]
    pub async fn verify_token(&self, token: &str) -> Result<UserId> {
        let secret = &self.config.jwt.access_token_secret;
        let claims = self.token_issuer.verify::<UserClaims>(token, secret)?;
        let id = claims.sub;
        let Some(existing_token) = self.token_store.retrieve(id.clone()).await else {
            bail!("Token 非法");
        };
        if existing_token != token {
            bail!("Token 已被回收");
        }
        Ok(UserId::new_unchecked(id))
    }

    #[tracing::instrument]
    pub async fn check_permissions(&self, id: &UserId, checker: PermissionChecker) -> Result<()> {
        let existing_group = self.permission_resolver.resolve(id).await;
        if !existing_group.permits(checker) {
            anyhow::bail!("权限不足");
        }
        Ok(())
    }

    #[tracing::instrument]
    pub fn get_all_permissions(&self) -> &'static [PermissionCode] {
        ALL_PERMISSIONS
    }

    #[tracing::instrument]
    pub fn get_all_pages(&self) -> &'static [Page] {
        PAGES.as_ref()
    }

    #[tracing::instrument]
    pub async fn get_available_pages(&self, user_id: UserId) -> [Page; 2] {
        let group = self.permission_resolver.resolve(&user_id).await;
        let mut pages = Self::get_available_pages_by_group(PAGES.as_ref(), &group);
        pages.extend_from_slice(&*SHARED_PAGES);
        [
            Page::builder()
                .key(NONE)
                .url("/")
                .maybe_redirect(Self::find_default_path(&pages))
                .build(),
            Page::builder().key(NONE).children(pages).build(),
        ]
    }

    #[tracing::instrument]
    fn find_default_path(pages: &[Page]) -> Option<&'static str> {
        if pages.is_empty() {
            return None;
        }
        pages.first().and_then(|page| {
            if let Some(children) = &page.children {
                return Self::find_default_path(children);
            }
            page.url
        })
    }

    #[tracing::instrument]
    fn get_available_pages_by_group(pages: &[Page], group: &PermissionGroup) -> Vec<Page> {
        pages
            .iter()
            .filter_map(|page| {
                let mut page = page.to_owned();
                let children = page
                    .children
                    .as_ref()
                    .map(|c| Self::get_available_pages_by_group(c, group));

                let has_access = group.permit(&page.key);
                let has_visible_children = children.as_ref().is_some_and(|c| !c.is_empty());
                if has_access || has_visible_children {
                    page.children = children.filter(|c| !c.is_empty());
                    Some(page)
                } else {
                    None
                }
            })
            .collect()
    }

    #[tracing::instrument]
    pub async fn replenish_user_portrait(&self, dtos: &mut [UserDto]) {
        stream::iter(dtos)
            .for_each_concurrent(5, |dto| async move {
                let upload_service = &self.upload_service;

                if let Some(portrait) = &dto.portrait {
                    if let Ok(url) = upload_service.presign_url(portrait).await {
                        dto.portrait = Some(url);
                    } else {
                        dto.portrait = None; // 失败设为空
                    }
                }
            })
            .await;
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use domain::iam::value_object::permission_code::SYSTEM;
    use infrastructure::{
        shared::{chrono_tz::ChronoTz, pg_pool::PgPool, workspace::WorkspaceRef},
        test_utils::{setup_database, setup_kvdb, setup_object_storage},
    };
    use sqlx::{prelude::FromRow, types::chrono::Utc};

    use crate::system::service::file_service::FileService;

    use super::*;

    #[derive(FromRow)]
    struct UserRow {
        id: UserId,
    }

    async fn build_service(pool: PgPool) -> IamService {
        setup_database(pool.clone()).await;
        let kvdb = setup_kvdb().await;
        let object_storage = setup_object_storage().await;
        let token_issuer = TokenIssuerImpl::builder()
            .config(ConfigRef::default())
            .ct(ChronoTz::default())
            .build();
        let token_store = TokenStoreImpl::builder().kvdb(kvdb.clone()).build();
        let permission_resolver = PermissionResolverImpl::builder()
            .pool(pool.clone())
            .kvdb(kvdb.clone())
            .build();
        let upload_service = {
            let file_service = FileService::builder()
                .pool(pool)
                .ct(ChronoTz::default())
                .build();
            UploadService::builder()
                .ct(ChronoTz::default())
                .object_storage(object_storage)
                .file_service(file_service)
                .workspace(WorkspaceRef::default())
                .build()
        };
        IamService::builder()
            .token_issuer(token_issuer)
            .token_store(token_store)
            .permission_resolver(permission_resolver)
            .config(ConfigRef::default())
            .upload_service(upload_service)
            .build()
    }

    #[sqlx::test]
    async fn test_get_all_permissions(pool: PgPool) {
        let service = build_service(pool).await;
        let pages = service.get_all_permissions();
        assert_eq!(pages.len(), ALL_PERMISSIONS.len());
    }

    #[sqlx::test]
    async fn test_get_all_pages(pool: PgPool) {
        let service = build_service(pool).await;
        let pages = service.get_all_pages();
        assert_eq!(pages.len(), PAGES.len());
    }

    #[sqlx::test]
    async fn test_get_available_pages(pool: PgPool) {
        let service = build_service(pool.clone()).await;
        let row: UserRow =
            sqlx::query_as(r#"SELECT id from _users WHERE privileged = true LIMIT 1"#)
                .fetch_one(&pool)
                .await
                .unwrap();
        let pages = service.get_available_pages(row.id).await;
        assert_eq!(pages.len(), 2);
    }

    #[sqlx::test]
    async fn test_replenish_user_portrait(pool: PgPool) {
        let service = build_service(pool.clone()).await;
        let user_dto = UserDto {
            id: UserId::generate().to_string(),
            account: "test".to_string(),
            portrait: None,
            name: "test".to_string(),
            role_ids: vec![],
            role_names: vec![],
            privileged: false,
            enabled: true,
            created_at: Utc::now().naive_local(),
            updated_at: Utc::now().naive_local(),
        };
        let mut dtos = vec![user_dto];
        service.replenish_user_portrait(&mut dtos).await;
        assert!(dtos[0].portrait.is_none());
        dtos[0].portrait = Some("test".to_string());
        service.replenish_user_portrait(&mut dtos).await;
        assert!(dtos[0].portrait.is_some());
    }

    #[sqlx::test]
    async fn test_check_permissions(pool: PgPool) {
        let service = build_service(pool.clone()).await;
        let row: UserRow =
            sqlx::query_as(r#"SELECT id from _users WHERE privileged = true LIMIT 1"#)
                .fetch_one(&pool)
                .await
                .unwrap();
        let mut hash_set = HashSet::new();
        hash_set.insert(SYSTEM);
        let result = service
            .check_permissions(
                &row.id,
                PermissionChecker::All(PermissionGroup::new(hash_set)),
            )
            .await;
        assert!(result.is_ok());

        let mut hash_set = HashSet::new();
        hash_set.insert(PermissionCode::new(-1));
        let result = service
            .check_permissions(
                &row.id,
                PermissionChecker::All(PermissionGroup::new(hash_set)),
            )
            .await;
        assert!(result.is_err_and(|err| err.to_string() == "权限不足"));
    }
}
