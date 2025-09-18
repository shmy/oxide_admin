use std::{fmt::Debug, ops::Deref, sync::Arc};

use super::config::Config;
use anyhow::Result;
use open_feature::{Client, OpenFeature};

#[derive(Clone)]
pub struct FeatureFlag {
    client: Arc<Client>,
}

impl Deref for FeatureFlag {
    type Target = Client;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl Debug for FeatureFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeatureFlag").finish()
    }
}

impl FeatureFlag {
    #[cfg(feature = "flag_flipt")]
    pub async fn try_new(config: &Config) -> Result<Self> {
        use flag_kit::{FliptProvider, FliptProviderConfig};
        let mut api = OpenFeature::singleton_mut().await;
        api.set_provider(FliptProvider::try_new(
            FliptProviderConfig::builder()
                .endpoint(config.flip.endpoint.to_string())
                .environment(config.flip.environment.to_string())
                .namespace(config.flip.namespace.to_string())
                .build(),
        )?)
        .await;
        let client = api.create_client();
        Ok(Self {
            client: Arc::new(client),
        })
    }

    #[cfg(not(feature = "flag_flipt"))]
    pub async fn try_new(_config: &Config) -> Result<Self> {
        use open_feature::provider::NoOpProvider;

        let mut api = OpenFeature::singleton_mut().await;
        api.set_provider(NoOpProvider::default()).await;
        let client = api.create_client();
        Ok(Self {
            client: Arc::new(client),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any as _, TypeId};

    use super::*;

    #[tokio::test]
    async fn test_try_new() {
        let result = FeatureFlag::try_new(&Config::default()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_debug() {
        let result = FeatureFlag::try_new(&Config::default()).await.unwrap();
        assert_eq!(format!("{:?}", &result), "FeatureFlag");
        assert_eq!(result.deref().type_id(), TypeId::of::<Client>());
    }
}
