use std::time::Duration;

use anyhow::Result;
use axum::http::Uri;
use opendal::{Operator, layers::LoggingLayer, services};

use crate::{ObjectStorageReader, ObjectStorageTrait};

#[derive(Clone)]
pub struct S3 {
    operator: Operator,
}

impl S3 {
    pub async fn try_new() -> Result<Self> {
        let builder = services::S3::default()
            .endpoint("http://localhost:9000")
            .bucket("oxide-admin")
            .access_key_id("nhdfHo5zV4C36G8sJWgy")
            .secret_access_key("YsZwjmy0BxUkcDitv4lf8gA9rXGu7hRFHozJO2nN")
            .region("region");
        let operator = Operator::new(builder)?
            .layer(LoggingLayer::default())
            .finish();
        operator.check().await?;
        Ok(Self { operator })
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
    fn purify_url<'a>(&self, signed: &'a str) -> &'a str {
        signed
    }
    fn purify_url_opt(&self, signed: Option<String>) -> Option<String> {
        signed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_s3() -> Result<()> {
        let s3 = S3::try_new().await?;
        Ok(())
    }
}
