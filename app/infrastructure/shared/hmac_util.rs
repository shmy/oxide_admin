use axum::http::Uri;
use chrono::Utc;
use const_format::concatcp;
use nject::injectable;
use serde::Deserialize;

use crate::shared::config::Config;

pub const UPLOAD_PATH: &str = "/uploads";

#[derive(Clone)]
#[injectable]
pub struct HmacUtil {
    config: Config,
}
impl HmacUtil {
    fn encode(&self, path: &str, expired_at: i64) -> String {
        let data = format!("{}:{}", path, expired_at);
        hex::encode(hmac_sha256::HMAC::mac(
            data.as_bytes(),
            self.config.upload.hmac_secret,
        ))
    }

    pub fn sign_path(&self, path: String) -> String {
        let expired_at = (Utc::now() + self.config.upload.link_period).timestamp();
        let sign = self.encode(&path, expired_at);
        let url = format!("{}/{}?sign={}&exp={}", UPLOAD_PATH, path, sign, expired_at);
        url
    }

    pub fn sign_path_opt(&self, path: Option<String>) -> Option<String> {
        let path = path?;
        if path.is_empty() {
            return None;
        }
        Some(self.sign_path(path))
    }

    pub fn strip_query<'a>(&self, signed: &'a str) -> &'a str {
        signed
            .split('?')
            .next()
            .and_then(|d| d.strip_prefix(concatcp!(UPLOAD_PATH, "/")))
            .unwrap_or(signed)
    }

    pub fn strip_query_opt(&self, signed: Option<String>) -> Option<String> {
        let signed = signed?;
        Some(self.strip_query(&signed).to_string())
    }

    pub fn verify_path(&self, url: Uri) -> bool {
        let path = url.path();
        let path = path
            .strip_prefix(concatcp!(UPLOAD_PATH, "/"))
            .unwrap_or(path);
        let Some(query) = url.query() else {
            return false;
        };
        let Ok(params) = serde_urlencoded::from_str::<FileParams>(query) else {
            return false;
        };
        if Utc::now() > params.exp {
            return false;
        }
        let expected = self.encode(path, params.exp.timestamp());
        expected == params.sign
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileParams {
    sign: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    exp: chrono::DateTime<Utc>,
}
