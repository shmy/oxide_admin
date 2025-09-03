use crate::iam::service::page::{PAGES, Page, SHARED_PAGES};
use anyhow::{Result, bail};
use domain::iam::value_object::permission_code::{ALL_PERMISSIONS, NONE, PermissionCode};
use domain::iam::value_object::permission_group::PermissionGroup;
use domain::iam::value_object::user_id::UserId;
use domain::shared::port::permission_resolver::PermissionResolver;
use domain::shared::port::token_issuer::{TokenIssuerTrait, UserClaims};
use domain::shared::port::token_store::TokenStoreTrait;
use infrastructure::port::permission_resolver_impl::PermissionResolverImpl;
use infrastructure::port::token_issuer_impl::TokenIssuerImpl;
use infrastructure::port::token_store_impl::TokenStoreImpl;
use infrastructure::shared::config::Config;
use nject::injectable;

#[derive(Clone)]
#[injectable]
pub struct IamService {
    token_issuer: TokenIssuerImpl,
    token_store: TokenStoreImpl,
    permission_resolver: PermissionResolverImpl,
    config: Config,
}

impl IamService {
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

    pub async fn check_permission(&self, id: &UserId, group: &PermissionGroup) -> Result<()> {
        let existing_group = self.permission_resolver.resolve(id).await;
        if !existing_group.permits_all(group) {
            anyhow::bail!("权限不足");
        }
        Ok(())
    }

    pub fn get_all_permissions(&self) -> &'static [PermissionCode] {
        ALL_PERMISSIONS
    }

    pub fn get_all_pages(&self) -> &'static [Page] {
        PAGES.as_ref()
    }

    pub async fn get_available_pages(&self, user_id: UserId) -> Result<[Page; 2]> {
        let group = self.permission_resolver.resolve(&user_id).await;
        let mut pages = Self::get_available_pages_by_group(PAGES.as_ref(), &group);
        pages.extend_from_slice(&*SHARED_PAGES);
        Ok([
            Page::builder()
                .key(NONE)
                .url("/")
                .maybe_redirect(Self::find_default_path(&pages))
                .build(),
            Page::builder().key(NONE).children(pages).build(),
        ])
    }

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
}
