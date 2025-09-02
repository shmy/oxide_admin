use std::time::Duration;

use crate::iam::service::page::{PAGES, Page, SHARED_PAGES};
use anyhow::{Result, bail};
use captcha_generator::CaptchaTrait as _;
use domain::iam::value_object::permission_code::{ALL_PERMISSIONS, NONE, PermissionCode};
use domain::iam::value_object::permission_group::PermissionGroup;
use domain::iam::value_object::user_id::UserId;
use domain::shared::id_generator::IdGenerator;
use domain::shared::permission_resolver::PermissionResolver;
use domain::shared::token_issuer::{TokenIssuerTrait, UserClaims};
use domain::shared::token_store::TokenStoreTrait;
use infrastructure::implementation::permission_resolver_impl::PermissionResolverImpl;
use infrastructure::implementation::token_issuer_impl::TokenIssuerImpl;
use infrastructure::implementation::token_store_impl::TokenStoreImpl;
use infrastructure::shared::config::Config;
use infrastructure::shared::kv::{Kv, KvTrait as _};
use nject::injectable;

#[derive(Clone)]
#[injectable]
pub struct IamService {
    token_issuer: TokenIssuerImpl,
    token_store: TokenStoreImpl,
    permission_resolver: PermissionResolverImpl,
    config: Config,
    kv: Kv,
}

impl IamService {
    fn fill_captcha_key(key: &str) -> String {
        format!("captcha:{key}")
    }
}

impl IamService {
    pub async fn generate_captcha_with_ttl(&self, ttl: Duration) -> Result<Captcha> {
        let math = captcha_generator::math::MathCaptcha::new(100, 140, 40);
        let captcha_data = math.generate()?;
        let key = IdGenerator::random();
        let full_key = Self::fill_captcha_key(&key);
        self.kv.set_with_ex(&full_key, captcha_data.value, ttl)?;
        Ok(Captcha {
            bytes: captcha_data.bytes,
            key,
        })
    }

    async fn verify_captcha(&self, key: &str, value: &str) -> Result<()> {
        let full_key = Self::fill_captcha_key(key);
        let existing_value = self.kv.get::<String>(&full_key)?;
        if existing_value != value {
            return Err(anyhow::anyhow!("验证码错误"));
        }

        let _ = self.kv.delete(&full_key);
        Ok(())
    }

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

pub struct Captcha {
    pub bytes: Vec<u8>, // 图形验证码的原始数据
    pub key: String,    // 与存储绑定，用于校验
}
