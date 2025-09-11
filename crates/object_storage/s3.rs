use std::time::Duration;

use anyhow::Result;
use axum::http::Uri;
use opendal::{Operator, layers::LoggingLayer, services};

use crate::{ObjectStorageReader, ObjectStorageTrait};

#[derive(Clone)]
pub struct S3 {
    operator: Operator,
    bucket: String,
}

impl S3 {
    pub async fn try_new(
        endpoint: &str,
        bucket: &str,
        access_key_id: &str,
        secret_access_key: &str,
        region: &str,
    ) -> Result<Self> {
        let bucket = bucket.to_string();
        let builder = services::S3::default()
            .endpoint(endpoint)
            .bucket(&bucket)
            .access_key_id(access_key_id)
            .secret_access_key(secret_access_key)
            .region(region);
        let operator = Operator::new(builder)?
            .layer(LoggingLayer::default())
            .finish();
        operator.check().await?;
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
