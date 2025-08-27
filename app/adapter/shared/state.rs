use application::re_export::provider::Provider;

#[derive(Clone)]
pub struct WebState {
    provider: Provider,
}

impl WebState {
    pub fn new(provider: Provider) -> Self {
        Self { provider }
    }

    pub fn provider(&self) -> &Provider {
        &self.provider
    }

    pub fn provider_owned(&self) -> Provider {
        self.provider.clone()
    }
}
