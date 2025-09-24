use std::{fmt::Debug, time::Duration};

use crate::error::Result;
use axum::http::Uri;
use bon::Builder;
use opendal::{Operator, layers::LoggingLayer, services};

use crate::{ObjectStorageReader, ObjectStorageTrait};

#[derive(Builder)]
pub struct S3Config {
    endpoint: String,
    bucket: String,
    access_key_id: String,
    secret_access_key: String,
    region: String,
}

#[derive(Clone)]
pub struct S3 {
    operator: Operator,
    bucket: String,
}

impl Debug for S3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("S3").finish()
    }
}

impl S3 {
    pub async fn try_new(config: S3Config) -> Result<Self> {
        let bucket = config.bucket;
        let builder = services::S3::default()
            .endpoint(&config.endpoint)
            .bucket(&bucket)
            .access_key_id(&config.access_key_id)
            .secret_access_key(&config.secret_access_key)
            .region(&config.region);
        let operator = Operator::new(builder)?
            .layer(LoggingLayer::default())
            .finish();
        operator.check().await?;
        tracing::info!("S3 bucket: {} connected", bucket);
        Ok(Self { operator, bucket })
    }
}

impl ObjectStorageTrait for S3 {
    fn operator(&self) -> Operator {
        self.operator.clone()
    }
}

impl ObjectStorageReader for S3 {
    async fn presign_url(&self, path: impl AsRef<str>) -> Result<String> {
        let req = self
            .operator()
            .presign_read(path.as_ref(), Duration::from_secs(60))
            .await?;

        Ok(req.uri().to_string())
    }
    fn verify_url(&self, _url: Uri) -> bool {
        true
    }

    fn purify_url(&self, signed: String) -> String {
        if let Ok(uri) = signed.parse::<Uri>() {
            let path = uri.path();
            let path = path
                .strip_prefix(&format!("/{}/", self.bucket))
                .unwrap_or(path)
                .to_string();
            return path;
        }
        signed.to_string()
    }
    fn purify_url_opt(&self, signed: Option<String>) -> Option<String> {
        let signed = signed?;
        Some(self.purify_url(signed).to_string())
    }
}
