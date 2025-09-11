use std::{
    io::{Read, Seek},
    path::Path,
    time::Duration,
};

use anyhow::Result;
use axum::http::Uri;
use chrono::Utc;
use futures_util::{SinkExt as _, Stream, StreamExt as _, TryStreamExt as _};
use opendal::{Operator, layers::LoggingLayer, services};
use serde::Deserialize;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::{ObjectStorageReader, ObjectStorageWriter};

#[derive(Clone)]
pub struct Fs {
    operator: Operator,
    basepath: String,
    hmac_secret: &'static [u8],
    link_period: Duration,
}

impl Fs {
    pub fn try_new(
        root: impl AsRef<Path>,
        basepath: &'static str,
        hmac_secret: &'static [u8],
        link_period: Duration,
    ) -> Result<Self> {
        let builder = services::Fs::default().root(&root.as_ref().to_string_lossy());
        let operator = Operator::new(builder)?
            .layer(LoggingLayer::default())
            .finish();
        let basepath = if basepath.ends_with("/") {
            basepath.to_string()
        } else {
            format!("{}/", basepath)
        };
        Ok(Self {
            operator,
            basepath,
            hmac_secret,
            link_period,
        })
    }
}

impl ObjectStorageWriter for Fs {
    async fn write_stream(
        &self,
        path: impl AsRef<str>,
        mut stream: impl Stream<Item = Result<ReaderStream<File>>> + Unpin,
    ) -> Result<()> {
        let writer = self
            .operator
            .writer_with(path.as_ref())
            .concurrent(8)
            .await?;
        let mut sink = writer.into_bytes_sink();

        while let Some(rs) = stream.next().await {
            let rs = rs?;
            sink.send_all(&mut rs.map_ok(|b| b)).await?;
        }

        sink.close().await?;
        Ok(())
    }

    async fn write(&self, path: impl AsRef<str>, mut reader: impl Read + Seek) -> Result<()> {
        reader.rewind()?;
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let _ = self.operator.write(path.as_ref(), buf).await?;
        Ok(())
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
}

impl Fs {
    fn encode_hmac(&self, path: &str, expired_at: u64) -> String {
        let data = format!("{}:{}", path, expired_at);
        hex::encode(hmac_sha256::HMAC::mac(data.as_bytes(), &self.hmac_secret))
    }

    fn sign_path(&self, path: &str) -> String {
        let expired_at = (Utc::now() + self.link_period).timestamp() as u64;
        let sign = self.encode_hmac(&path, expired_at);
        let url = format!("{}{}?sign={}&exp={}", self.basepath, path, sign, expired_at);
        url
    }

    pub fn purify_url<'a>(&self, signed: &'a str) -> &'a str {
        signed
            .split('?')
            .next()
            .and_then(|d| d.strip_prefix(&self.basepath))
            .unwrap_or(signed)
    }

    pub fn purify_url_opt(&self, signed: Option<String>) -> Option<String> {
        let signed = signed?;
        Some(self.purify_url(&signed).to_string())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileParams {
    sign: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    exp: chrono::DateTime<Utc>,
}
