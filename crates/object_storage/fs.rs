use std::io::{Read, Seek};

use anyhow::Result;
use futures_util::{SinkExt as _, Stream, StreamExt as _, TryStreamExt as _};
use opendal::{Operator, layers::LoggingLayer, services};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::ObjectStorageTrait;

#[derive(Clone)]
pub struct Fs {
    operator: Operator,
}

impl Fs {
    pub fn try_new(root: impl AsRef<str>) -> Result<Self> {
        let builder = services::Fs::default().root(root.as_ref());
        let operator = Operator::new(builder)?
            .layer(LoggingLayer::default())
            .finish();
        Ok(Self { operator })
    }
}

impl ObjectStorageTrait for Fs {
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

    async fn sign_url(&self, _path: impl AsRef<str>) -> Result<String> {
        Ok("_".to_string())
    }
}
