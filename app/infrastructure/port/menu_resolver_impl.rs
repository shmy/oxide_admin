use crate::error::{InfrastructureError, InfrastructureResult};
use bon::Builder;
use domain::iam::error::IamError;
use domain::iam::value_object::menu::{ALL_MENUS, Menu};
use domain::iam::value_object::menu_group::MenuGroup;
use domain::iam::value_object::role_id::RoleId;
use domain::iam::value_object::user_id::UserId;
use domain::shared::port::menu_resolver::MenuResolver;
use domain::shared::to_inner_vec::ToInnerVec;
use kvdb_kit::{Kvdb, KvdbTrait as _};
use nject::injectable;
use single_flight::single_flight;
use sqlx::prelude::FromRow;
use std::collections::HashSet;
use std::time::Duration;

use crate::shared::pg_pool::PgPool;

const KEY_PREFIX: &str = "menu:";

#[derive(Debug, Clone, Builder)]
#[injectable]
pub struct MenuResolverImpl {
    pool: PgPool,
    kvdb: Kvdb,
}

impl MenuResolverImpl {
    #[tracing::instrument]
    async fn solve(&self, id: &UserId) -> InfrastructureResult<MenuGroup> {
        let res = async move {
            if let Ok(menu_group) = self.find_from_db(id.clone()).await {
                return menu_group;
            }
            MenuGroup::default()
        }
        .await;
        Ok(res)
    }
}
impl MenuResolver for MenuResolverImpl {
    type Error = InfrastructureError;

    #[tracing::instrument]
    async fn resolve(&self, id: &UserId) -> MenuGroup {
        match self.kvdb.get(&self.full_key(id)).await {
            Some(cache) => cache,
            None => match self.solve(id).await {
                Ok(cache) => {
                    let _ = self
                        .kvdb
                        .set_with_ex(
                            &self.full_key(id),
                            cache.clone(),
                            Duration::from_secs(30 * 60),
                        )
                        .await;
                    cache
                }
                Err(_) => Default::default(),
            },
        }
    }

    #[tracing::instrument]
    async fn refresh(&self) -> Result<(), Self::Error> {
        self.kvdb.delete_prefix(KEY_PREFIX).await?;
        Ok(())
    }
}

impl MenuResolverImpl {
    #[single_flight]
    pub async fn find_from_db(&self, id: UserId) -> Result<MenuGroup, IamError> {
        let user_record = sqlx::query!(
            r#"SELECT privileged, role_ids as "role_ids: Vec<RoleId>" from _users WHERE id = $1"#,
            &id
        )
        .fetch_one(&self.pool)
        .await?;
        if user_record.privileged {
            return Ok(MenuGroup::new(ALL_MENUS.iter().cloned().collect()));
        }

        let mut menus = HashSet::new();
        let role_records = sqlx::query_as!(
            RoleRecord,
            r#"
            SELECT privileged, menus as "menus: Vec<Menu>" from _roles WHERE id = ANY($1) AND enabled = true
            "#,
            &user_record.role_ids.inner_vec()
        )
        .fetch_all(&self.pool)
        .await?;

        for role in role_records {
            if role.privileged {
                menus.extend(ALL_MENUS.to_vec());
            } else {
                menus.extend(role.menus);
            }
        }
        Ok(MenuGroup::new(menus))
    }

    fn full_key(&self, id: &UserId) -> String {
        format!("{}{}", KEY_PREFIX, &**id)
    }
}

#[derive(FromRow)]
struct RoleRecord {
    privileged: bool,
    menus: Vec<Menu>,
}
