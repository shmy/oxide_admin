use crate::error::{ApplicationError, ApplicationResult};
use axum::http::Uri;
use bon::Builder;
use domain::shared::id_generator::IdGenerator;
use futures_util::{StreamExt, TryFutureExt, stream};
use image::{ImageFormat, ImageReader};
use imageformat::detect_image_format;
use infrastructure::shared::chrono_tz::{ChronoTz, Datelike as _};
use infrastructure::shared::workspace::WorkspaceRef;
use nject::injectable;
use object_storage_kit::error::ObjectStorageError;
use object_storage_kit::{ObjectStorage, ObjectStorageReader, ObjectStorageWriter};
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
use utoipa::ToSchema;

use crate::system::service::file_service::FileService;

#[derive(Debug, Clone, Builder)]
#[injectable]
pub struct UploadService {
    ct: ChronoTz,
    file_service: FileService,
    object_storage: ObjectStorage,
    workspace: WorkspaceRef,
}

impl UploadService {
    #[tracing::instrument(skip(file))]
    pub async fn image(&self, mut file: NamedTempFile) -> ApplicationResult<FinishResponse> {
        let Some(format) = SupportedFormat::validate_image_type(&mut file) else {
            return Err(ApplicationError::UnsupportedImageFormat);
        };
        let filename = IdGenerator::filename().to_lowercase();
        let relative_path = self.build_relative_path(format!("{}.{}", filename, "webp"));
        let reader = SupportedFormat::convert_to_webp(format, file).await?;
        tokio::try_join!(
            self.object_storage
                .write(&relative_path, reader)
                .map_err(Into::into),
            self.file_service.create(&relative_path)
        )?;
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
    ) -> ApplicationResult<FinishResponse> {
        let extension = Self::extract_extension(filename);
        let filename = IdGenerator::filename().to_lowercase();
        let relative_path = self.build_relative_path(format!("{filename}{extension}"));
        tokio::try_join!(
            self.object_storage
                .write(&relative_path, file)
                .map_err(Into::into),
            self.file_service.create(&relative_path)
        )?;
        Ok(FinishResponse {
            url: self.object_storage.presign_url(&relative_path).await?,
            value: relative_path,
        })
    }

    #[tracing::instrument]
    pub async fn start_chunk(&self, filename: String) -> ApplicationResult<StartChunkResponse> {
        let extension = Self::extract_extension(Some(filename));
        let key = IdGenerator::filename().to_lowercase();
        let upload_id = format!("{key}{extension}").to_lowercase();
        let tmp_dir = self.workspace.temp_dir().join(&key);
        tokio::fs::create_dir_all(tmp_dir).await?;
        Ok(StartChunkResponse { key, upload_id })
    }

    #[tracing::instrument(skip(file))]
    pub async fn chunk(
        &self,
        key: String,
        part_number: u32,
        file: NamedTempFile,
    ) -> ApplicationResult<ChunkResponse> {
        let tmp_dir = self.workspace.temp_dir().join(&key);
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
    ) -> ApplicationResult<FinishResponse> {
        let tmp_dir = self.workspace.temp_dir().join(&key);
        let relative_path = self.build_relative_path(upload_id);
        let stream = stream::iter(part_list).then(|part| {
            let chunk_path = tmp_dir.join(part.part_number.to_string());
            async move {
                let file = File::open(chunk_path).await?;
                let reader = ReaderStream::new(file);
                Ok::<_, ObjectStorageError>(reader)
            }
        });
        tokio::try_join!(
            async {
                self.object_storage
                    .write_stream(&relative_path, pin::pin!(stream))
                    .await
                    .map_err(Into::into)
            },
            self.file_service.create(&relative_path)
        )?;
        Ok(FinishResponse {
            url: self.object_storage.presign_url(&relative_path).await?,
            value: relative_path,
        })
    }

    #[tracing::instrument(skip(path))]
    pub async fn presign_url(&self, path: impl AsRef<str>) -> ApplicationResult<String> {
        let url = self.object_storage.presign_url(path).await?;
        Ok(url)
    }

    #[tracing::instrument]
    pub fn verify_url(&self, url: Uri) -> bool {
        self.object_storage.verify_url(url)
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete(&self, path: impl AsRef<str>) -> ApplicationResult<()> {
        self.delete_many(Vec::from([path.as_ref().to_string()]))
            .await
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_many(&self, paths: Vec<String>) -> ApplicationResult<()> {
        self.object_storage.delete_many(paths).await?;
        Ok(())
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

#[derive(Debug, Serialize, ToSchema)]
pub struct StartChunkResponse {
    pub key: String,
    #[serde(rename = "uploadId")]
    pub upload_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChunkResponse {
    #[serde(rename = "eTag")]
    pub e_tag: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PartItem {
    #[serde(rename = "partNumber")]
    pub part_number: u32,
}

#[derive(Debug, Serialize, ToSchema)]
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
    ) -> ApplicationResult<impl Read + Seek> {
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

async fn persist_file(file: NamedTempFile, destination: &Path) -> ApplicationResult<()> {
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

#[cfg(test)]
mod tests {
    use std::{io::Write, str::FromStr as _};

    use infrastructure::{
        shared::pg_pool::PgPool,
        test_utils::{setup_database, setup_object_storage},
    };

    use crate::system::service::file_service;

    use super::*;

    async fn build_service(pool: PgPool) -> UploadService {
        setup_database(pool.clone()).await;
        let object_storage = setup_object_storage().await;
        let file_service = {
            file_service::FileService::builder()
                .pool(pool.clone())
                .ct(ChronoTz::default())
                .build()
        };
        UploadService::builder()
            .ct(ChronoTz::default())
            .object_storage(object_storage)
            .file_service(file_service)
            .workspace(WorkspaceRef::default())
            .build()
    }

    fn build_text_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Brian was here. Briefly.").unwrap();
        file.rewind().unwrap();
        file
    }

    fn build_png_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(include_bytes!("../../tests/fixtures/rust-logo.png"))
            .unwrap();
        file.rewind().unwrap();
        file
    }

    fn build_jpg_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(include_bytes!("../../tests/fixtures/rust-logo.jpg"))
            .unwrap();
        file.rewind().unwrap();
        file
    }

    fn build_jpeg_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(include_bytes!("../../tests/fixtures/rust-logo.jpeg"))
            .unwrap();
        file.rewind().unwrap();
        file
    }

    fn build_webp_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(include_bytes!("../../tests/fixtures/rust-logo.webp"))
            .unwrap();
        file.rewind().unwrap();
        file
    }

    fn build_bmp_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(include_bytes!("../../tests/fixtures/rust-logo.bmp"))
            .unwrap();
        file.rewind().unwrap();
        file
    }

    #[tokio::test]
    async fn test_persist() {
        let file = build_text_file();
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("test.txt");
        persist_file(file, &destination).await.unwrap();
        assert!(destination.exists());
    }

    #[sqlx::test]
    async fn test_image_upload_return_ok_given_png_format(pool: PgPool) {
        let service = build_service(pool).await;
        let result = service.image(build_png_file()).await;
        assert!(result.is_ok());
        let path = result.unwrap().value;
        assert!(path.ends_with(".webp"));
        let presigned_url = service.presign_url(path).await.unwrap();
        let uri = Uri::from_str(&presigned_url).unwrap();
        assert!(service.verify_url(uri));
    }

    #[sqlx::test]
    async fn test_image_upload_return_ok_given_jpg_format(pool: PgPool) {
        let service = build_service(pool).await;
        let result = service.image(build_jpg_file()).await;
        assert!(result.is_ok());
        let path = result.unwrap().value;
        assert!(path.ends_with(".webp"));
        let presigned_url = service.presign_url(path).await.unwrap();
        let uri = Uri::from_str(&presigned_url).unwrap();
        assert!(service.verify_url(uri));
    }

    #[sqlx::test]
    async fn test_image_upload_return_ok_given_jpeg_format(pool: PgPool) {
        let service = build_service(pool).await;
        let result = service.image(build_jpeg_file()).await;
        assert!(result.is_ok());
        let path = result.unwrap().value;
        assert!(path.ends_with(".webp"));
        let presigned_url = service.presign_url(path).await.unwrap();
        let uri = Uri::from_str(&presigned_url).unwrap();
        assert!(service.verify_url(uri));
    }

    #[sqlx::test]
    async fn test_image_upload_return_ok_given_webp_format(pool: PgPool) {
        let service = build_service(pool).await;
        let result = service.image(build_webp_file()).await;
        assert!(result.is_ok());
        let path = result.unwrap().value;
        assert!(path.ends_with(".webp"));
        let presigned_url = service.presign_url(path).await.unwrap();
        let uri = Uri::from_str(&presigned_url).unwrap();
        assert!(service.verify_url(uri));
    }

    #[sqlx::test]
    async fn test_image_upload_return_err_given_bmp_format(pool: PgPool) {
        let service = build_service(pool).await;
        let result = service.image(build_bmp_file()).await;
        assert!(result.is_err_and(|err| err.to_string() == "不支持的图片格式"));
    }

    #[sqlx::test]
    async fn test_image_upload_return_err_given_text_format(pool: PgPool) {
        let service = build_service(pool).await;
        let result = service.image(build_text_file()).await;
        assert!(result.is_err_and(|err| err.to_string() == "不支持的图片格式"));
    }

    #[sqlx::test]
    async fn test_single_upload(pool: PgPool) {
        let service = build_service(pool).await;
        assert!(service.single(None, build_text_file()).await.is_ok());
        let result = service
            .single(Some("test.txt".to_string()), build_text_file())
            .await;
        assert!(result.is_ok());
        let path = result.unwrap().value;
        assert!(path.ends_with(".txt"));
        let presigned_url = service.presign_url(&path).await.unwrap();
        let uri = Uri::from_str(&presigned_url).unwrap();
        assert!(service.verify_url(uri));
        assert!(service.delete(path).await.is_ok());
    }

    #[sqlx::test]
    async fn test_chunk_upload(pool: PgPool) {
        let service = build_service(pool).await;
        let start_response = service.start_chunk("test.txt".to_string()).await.unwrap();
        let mut part_list = Vec::new();
        assert!(
            service
                .chunk(start_response.key.to_string(), 1, build_text_file())
                .await
                .is_ok()
        );
        part_list.push(PartItem { part_number: 1 });
        assert!(
            service
                .chunk(start_response.key.to_string(), 2, build_text_file())
                .await
                .is_ok()
        );
        part_list.push(PartItem { part_number: 2 });
        assert!(
            service
                .finish_chunk(
                    start_response.key.to_string(),
                    start_response.upload_id,
                    part_list,
                )
                .await
                .is_ok()
        );
    }
}
