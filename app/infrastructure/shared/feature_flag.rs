use std::{fmt::Debug, ops::Deref, sync::Arc};

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
    pub async fn try_new() -> Result<Self> {
        use flag_kit::FliptProvider;
        let mut api = OpenFeature::singleton_mut().await;
        api.set_provider(FliptProvider::try_new("staging", "oxide-admin")?)
            .await;
        let client = api.create_client();
        Ok(Self {
            client: Arc::new(client),
        })
    }

    #[cfg(not(feature = "flag_flipt"))]
    pub async fn try_new() -> Result<Self> {
        use open_feature::provider::NoOpProvider;

        let mut api = OpenFeature::singleton_mut().await;
        api.set_provider(NoOpProvider::default()).await;
        let client = api.create_client();
        Ok(Self {
            client: Arc::new(client),
        })
    }
}
