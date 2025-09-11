use std::io::{Read, Seek};

use anyhow::Result;
use futures_util::Stream;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
#[cfg(feature = "fs")]
mod fs;
#[cfg(feature = "fs")]
pub type ObjectStorage = fs::Fs;

pub trait ObjectStorageTrait {
    fn write_stream(
        &self,
        path: impl AsRef<str>,
        stream: impl Stream<Item = Result<ReaderStream<File>>> + Unpin,
    ) -> impl Future<Output = Result<()>>;
    fn write(
        &self,
        path: impl AsRef<str>,
        reader: impl Read + Seek,
    ) -> impl Future<Output = Result<()>>;
    fn sign_url(&self, path: impl AsRef<str>) -> impl Future<Output = Result<String>>;
}
