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
