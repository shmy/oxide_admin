use bon::Builder;

use nject::provider;

use crate::{shared::config::Config, shared::kv::Kv, shared::pool::Pool};

#[derive(Clone, Builder)]
#[provider]
pub struct Provider {
    #[provide(Pool, |dep| dep.clone())]
    pool: Pool,
    #[provide(Kv, |dep| dep.clone())]
    kv: Kv,
    #[provide(Config, |dep| dep.clone())]
    config: Config,
}
