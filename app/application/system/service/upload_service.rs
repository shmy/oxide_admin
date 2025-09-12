use anyhow::{Result, bail};
use axum::http::Uri;
use domain::shared::id_generator::IdGenerator;
use futures_util::{StreamExt, stream};
use image::{ImageFormat, ImageReader};
use imageformat::detect_image_format;
use infrastructure::shared::{
    chrono_tz::{ChronoTz, Datelike as _},
    path::TEMP_DIR,
};
use nject::injectable;
use object_storage::{ObjectStorage, ObjectStorageReader, ObjectStorageWriter};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};
use std::{
    io::Seek,
    path::Path,
    pin::{self},
};
use tempfile::NamedTempFile;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

#[derive(Debug, Clone)]
#[injectable]
pub struct UploadService {
    ct: ChronoTz,
    object_storage: ObjectStorage,
}

impl UploadService {
    #[tracing::instrument(skip(file))]
    pub async fn image(&self, mut file: NamedTempFile) -> Result<FinishResponse> {
        let Some(format) = SupportedFormat::validate_image_type(&mut file) else {
            bail!("不支持的图片格式");
        };
        let filename = IdGenerator::filename().to_lowercase();
        let relative_path = self.build_relative_path(format!("{}.{}", filename, "webp"));
        let reader = SupportedFormat::convert_to_webp(format, file).await?;
        self.object_storage.write(&relative_path, reader).await?;
        Ok(FinishResponse {
            url: self.object_storage.presign_url(&relative_path).await?,
            value: relative_path,
        })
    }

    #[tracing::instrument(skip(file))]
    pub async fn single(
        &self,
        filename: Option<String>,
        file: NamedTempFile,
    ) -> Result<FinishResponse> {
        let extension = Self::extract_extension(filename);
        let filename = IdGenerator::filename().to_lowercase();
        let relative_path = self.build_relative_path(format!("{filename}.{extension}"));
        self.object_storage.write(&relative_path, file).await?;
        Ok(FinishResponse {
            url: self.object_storage.presign_url(&relative_path).await?,
            value: relative_path,
        })
    }

    #[tracing::instrument]
    pub async fn start_chunk(&self, filename: String) -> Result<StartChunkResponse> {
        let extension = Self::extract_extension(Some(filename));
        let key = IdGenerator::filename().to_lowercase();
        let upload_id = format!("{key}{extension}").to_lowercase();
        let tmp_dir = TEMP_DIR.join(&key);
        tokio::fs::create_dir_all(tmp_dir).await?;
        Ok(StartChunkResponse { key, upload_id })
    }

    #[tracing::instrument(skip(file))]
    pub async fn chunk(
        &self,
        key: String,
        part_number: u32,
        file: NamedTempFile,
    ) -> Result<ChunkResponse> {
        let tmp_dir = TEMP_DIR.join(&key);
        let filepath = tmp_dir.join(part_number.to_string());
        persist_file(file, &filepath).await?;
        Ok(ChunkResponse {
            e_tag: part_number.to_string(),
        })
    }

    #[tracing::instrument]
    pub async fn finish_chunk(
        &self,
        key: String,
        upload_id: String,
        part_list: Vec<PartItem>,
    ) -> Result<FinishResponse> {
        let tmp_dir = TEMP_DIR.join(&key);
        let relative_path = self.build_relative_path(upload_id);
        let stream = stream::iter(part_list).then(|part| {
            let chunk_path = tmp_dir.join(part.part_number.to_string());
            async move {
                let file = File::open(chunk_path).await?;
                let reader = ReaderStream::new(file);
                Ok::<_, anyhow::Error>(reader)
            }
        });
        self.object_storage
            .write_stream(&relative_path, pin::pin!(stream))
            .await?;
        Ok(FinishResponse {
            url: self.object_storage.presign_url(&relative_path).await?,
            value: relative_path,
        })
    }

    #[tracing::instrument(skip(path))]
    pub async fn presign_url(&self, path: impl AsRef<str>) -> Result<String> {
        self.object_storage.presign_url(path).await
    }

    #[tracing::instrument]
    pub fn verify_url(&self, url: Uri) -> bool {
        self.object_storage.verify_url(url)
    }

    #[tracing::instrument]
    fn build_relative_path(&self, filename: String) -> String {
        let now = self.ct.now();
        let year = now.year().to_string();
        let month = format!("{:02}", now.month());
        let relative_path = format!("{year}/{month}/{filename}");
        relative_path
    }

    #[tracing::instrument]
    fn extract_extension(path: Option<String>) -> String {
        path.as_ref()
            .and_then(|f| Path::new(f).extension())
            .and_then(|ext| ext.to_str())
            .map(|ext| format!(".{}", ext))
            .unwrap_or_default()
    }
}

#[derive(Debug, Serialize)]
pub struct StartChunkResponse {
    pub key: String,
    #[serde(rename = "uploadId")]
    pub upload_id: String,
}

#[derive(Debug, Serialize)]
pub struct ChunkResponse {
    #[serde(rename = "eTag")]
    pub e_tag: String,
}

#[derive(Debug, Deserialize)]
pub struct PartItem {
    #[serde(rename = "partNumber")]
    pub part_number: u32,
}

#[derive(Debug, Serialize)]
pub struct FinishResponse {
    pub value: String,
    pub url: String,
}

enum SupportedFormat {
    Png,
    Jpg,
    Webp,
}

impl SupportedFormat {
    fn validate_image_type<R: std::io::Read>(contents: &mut R) -> Option<SupportedFormat> {
        let Ok(format) = detect_image_format(contents) else {
            return None;
        };
        let extension = match format {
            imageformat::ImageFormat::Jpeg | imageformat::ImageFormat::JpegXl => {
                SupportedFormat::Jpg
            }
            imageformat::ImageFormat::Png => SupportedFormat::Png,
            imageformat::ImageFormat::Webp => SupportedFormat::Webp,
            _ => {
                return None;
            }
        };
        Some(extension)
    }

    async fn convert_to_webp(
        format: SupportedFormat,
        mut file: NamedTempFile,
    ) -> Result<impl Read + Seek> {
        if let SupportedFormat::Webp = format {
            let data = tokio::fs::read(file.path()).await?;
            return Ok(Cursor::new(data)); // Cursor 实现了 Read + Seek
        }
        tokio::task::spawn_blocking(move || {
            file.rewind()?;
            let reader = std::io::BufReader::new(file.into_file());
            let img = ImageReader::with_format(
                reader,
                match format {
                    SupportedFormat::Png => ImageFormat::Png,
                    SupportedFormat::Jpg => ImageFormat::Jpeg,
                    SupportedFormat::Webp => unreachable!(),
                },
            )
            .decode()?;
            let mut data = Cursor::new(Vec::new());
            img.write_to(&mut data, ImageFormat::WebP)?;

            Ok(data)
        })
        .await?
    }
}

async fn persist_file(file: NamedTempFile, destination: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        file.persist(destination)?;
    }
    #[cfg(windows)]
    {
        match file.persist(destination) {
            Ok(_) => {}
            Err(e) => {
                let tmp = e.file;
                tokio::fs::copy(tmp.path(), destination).await?;
            }
        }
    }
    Ok(())
}
