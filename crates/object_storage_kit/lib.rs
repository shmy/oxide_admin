use futures_util::{SinkExt as _, Stream, StreamExt as _, TryStreamExt as _};
use std::io::{Read, Seek};

use anyhow::Result;
use axum::http::Uri;
use opendal::{DeleteInput, IntoDeleteInput, Operator};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
#[cfg(feature = "fs")]
mod fs;
#[cfg(feature = "s3")]
mod s3;
#[cfg(feature = "fs")]
pub type ObjectStorage = fs::Fs;

#[cfg(feature = "fs")]
pub use fs::FsConfig;

#[cfg(feature = "s3")]
pub type ObjectStorage = s3::S3;

#[cfg(feature = "s3")]
pub use s3::S3Config;

pub trait ObjectStorageTrait {
    fn operator(&self) -> Operator;
}

pub trait ObjectStorageWriter: ObjectStorageTrait {
    fn write_stream(
        &self,
        path: impl AsRef<str>,
        mut stream: impl Stream<Item = Result<ReaderStream<File>>> + Unpin,
    ) -> impl Future<Output = Result<()>> {
        async move {
            let writer = self
                .operator()
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
    }

    fn write(
        &self,
        path: impl AsRef<str>,
        mut reader: impl Read + Seek,
    ) -> impl Future<Output = Result<()>> {
        async move {
            reader.rewind()?;
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf)?;
            let _ = self.operator().write(path.as_ref(), buf).await?;
            Ok(())
        }
    }

    fn delete_many(&self, paths: Vec<String>) -> impl Future<Output = Result<()>> {
        async move {
            let items: Vec<DeleteInput> = paths
                .into_iter()
                .map(IntoDeleteInput::into_delete_input)
                .collect();
            self.operator().delete_iter(items).await?;
            Ok(())
        }
    }
}

impl<T> ObjectStorageWriter for T where T: ObjectStorageTrait {}

pub trait ObjectStorageReader {
    fn presign_url(&self, path: impl AsRef<str>) -> impl Future<Output = Result<String>>;
    fn verify_url(&self, url: Uri) -> bool;
    fn purify_url(&self, signed: String) -> String;
    fn purify_url_opt(&self, signed: Option<String>) -> Option<String>;
}
