use std::{fmt::Debug, time::Duration};

use anyhow::Result;
use axum::http::Uri;
use bon::Builder;
use chrono::Utc;
use opendal::{Operator, layers::LoggingLayer, services};
use serde::Deserialize;

use crate::{ObjectStorageReader, ObjectStorageTrait};

#[derive(Builder)]
pub struct FsConfig {
    root: String,
    basepath: String,
    hmac_secret: &'static [u8],
    link_period: Duration,
}

#[derive(Clone)]
pub struct Fs {
    operator: Operator,
    basepath: String,
    hmac_secret: &'static [u8],
    link_period: Duration,
}
impl Debug for Fs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fs").finish()
    }
}

impl Fs {
    pub fn try_new(config: FsConfig) -> Result<Self> {
        let builder = services::Fs::default().root(&config.root);
        let operator = Operator::new(builder)?
            .layer(LoggingLayer::default())
            .finish();
        let basepath = if config.basepath.ends_with("/") {
            config.basepath.to_string()
        } else {
            format!("{}/", config.basepath)
        };
        Ok(Self {
            operator,
            basepath,
            hmac_secret: config.hmac_secret,
            link_period: config.link_period,
        })
    }
}

impl ObjectStorageTrait for Fs {
    fn operator(&self) -> Operator {
        self.operator.clone()
    }
}

impl ObjectStorageReader for Fs {
    async fn presign_url(&self, path: impl AsRef<str>) -> Result<String> {
        Ok(self.sign_path(path.as_ref()))
    }
    fn verify_url(&self, url: Uri) -> bool {
        let path = url.path();
        let path = path.strip_prefix(&self.basepath).unwrap_or(path);
        let Some(query) = url.query() else {
            return false;
        };
        let Ok(params) = serde_urlencoded::from_str::<FileParams>(query) else {
            return false;
        };
        if Utc::now() > params.exp {
            return false;
        }
        let expected = self.encode_hmac(path, params.exp.timestamp() as u64);

        expected == params.sign
    }

    fn purify_url(&self, signed: String) -> String {
        signed
            .split('?')
            .next()
            .and_then(|s| s.strip_prefix(&self.basepath))
            .unwrap_or(&signed)
            .to_string()
    }

    fn purify_url_opt(&self, signed: Option<String>) -> Option<String> {
        let signed = signed?;
        Some(self.purify_url(signed).to_string())
    }
}

impl Fs {
    fn encode_hmac(&self, path: &str, expired_at: u64) -> String {
        let data = format!("{}:{}", path, expired_at);
        hex::encode(hmac_sha256::HMAC::mac(data.as_bytes(), self.hmac_secret))
    }

    fn sign_path(&self, path: &str) -> String {
        let expired_at = (Utc::now() + self.link_period).timestamp() as u64;
        let sign = self.encode_hmac(path, expired_at);
        let url = format!("{}{}?sign={}&exp={}", self.basepath, path, sign, expired_at);
        url
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileParams {
    sign: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    exp: chrono::DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn build_fs() -> Fs {
        let dir = tempfile::tempdir().unwrap();
        Fs::try_new(
            FsConfig::builder()
                .root(dir.path().to_string_lossy().to_string())
                .basepath("/uploads".to_string())
                .hmac_secret(b"secret")
                .link_period(Duration::from_secs(60))
                .build(),
        )
        .unwrap()
    }

    #[test]
    fn test_try_new() {
        let fs = build_fs();
        let debug_str = format!("{:?}", fs);
        assert_eq!(debug_str, "Fs");
    }

    #[test]
    fn test_try_new_return_ok_given_basepath_has_trailing_slash() {
        let dir = tempfile::tempdir().unwrap();

        let fs = Fs::try_new(
            FsConfig::builder()
                .root(dir.path().to_string_lossy().to_string())
                .basepath("/uploads/".to_string())
                .hmac_secret(b"secret")
                .link_period(Duration::from_secs(60))
                .build(),
        )
        .unwrap();
        let debug_str = format!("{:?}", fs);
        assert_eq!(debug_str, "Fs");
    }

    #[test]
    fn test_verify_url_return_false_given_url_is_not_query() {
        let fs = build_fs();
        let uri = Uri::from_static("http://localhost:8080/uploads/test.txt");
        assert!(!fs.verify_url(uri));
    }

    #[test]
    fn test_verify_url_return_false_given_url_invalid() {
        let fs = build_fs();
        let uri = Uri::from_static("http://localhost:8080/uploads/test.txt?a=1&b=2");
        assert!(!fs.verify_url(uri));
    }

    #[test]
    fn test_verify_url_return_false_given_url_exp_expired() {
        let fs = build_fs();
        let now = (Utc::now() - Duration::from_secs(60)).timestamp();
        let uri = Uri::from_str(&format!(
            "http://localhost:8080/uploads/test.txt?sign=123456&exp={}",
            now
        ))
        .unwrap();
        assert!(!fs.verify_url(uri));
    }

    #[test]
    fn test_purify_url_opt() {
        let fs = build_fs();

        assert!(fs.purify_url_opt(None).is_none());
        assert!(
            fs.purify_url_opt(Some("2025/09/09/test.txt".to_string()))
                .is_some()
        );
        assert!(
            fs.purify_url_opt(Some("2025/09/09/test.txt?a=1&b=2".to_string()))
                .is_some()
        );
    }
}
