use crate::system::service::file_service::FileService;
use anyhow::{Result, bail};
use domain::shared::id_generator::IdGenerator;
use image::{ImageFormat, ImageReader};
use imageformat::detect_image_format;
use infrastructure::shared::{
    chrono_tz::{ChronoTz, Datelike as _},
    hmac_util::HmacUtil,
    path::{TEMP_DIR, UPLOAD_DIR},
};
use nject::injectable;
use serde::{Deserialize, Serialize};
use std::{
    io::Seek,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;
use tokio::{
    fs::File,
    io::{AsyncReadExt as _, AsyncWriteExt as _, BufReader, BufWriter},
};

#[injectable]
pub struct UploadService {
    file_service: FileService,
    hman_util: HmacUtil,
    ct: ChronoTz,
}

impl UploadService {
    pub async fn image(&self, mut file: NamedTempFile) -> Result<FinishResponse> {
        let Some(format) = SupportedFormat::validate_image_type(&mut file) else {
            bail!("不支持的图片格式");
        };
        let (path_buf, relative_path) = self.build_paths().await?;
        let filename = IdGenerator::filename().to_lowercase();
        let relative_path = format!("{}/{}.{}", relative_path, filename, "webp");
        let filepath = path_buf.join(format!("{}.{}", filename, "webp"));
        tokio::try_join!(
            SupportedFormat::convert_to_webp(format, file, filepath),
            self.file_service.create(&relative_path)
        )?;
        Ok(FinishResponse {
            value: self.hman_util.sign_path(relative_path),
        })
    }

    pub async fn single(
        &self,
        filename: Option<String>,
        file: NamedTempFile,
    ) -> Result<FinishResponse> {
        let extension = Self::extract_extension(filename);
        let (path_buf, relative_path) = self.build_paths().await?;
        let filename = IdGenerator::filename().to_lowercase();
        let relative_path = format!("{}/{}{}", relative_path, filename, extension);
        let filepath = path_buf.join(format!("{}{}", filename, extension));
        tokio::try_join!(
            async {
                let _ = file.persist(filepath);
                Ok(())
            },
            self.file_service.create(&relative_path)
        )?;
        Ok(FinishResponse {
            value: self.hman_util.sign_path(relative_path),
        })
    }

    pub async fn start_chunk(&self, filename: String) -> Result<StartChunkResponse> {
        let extension = Self::extract_extension(Some(filename));
        let key = IdGenerator::filename().to_lowercase();
        let upload_id = format!("{key}{extension}").to_lowercase();
        let tmp_dir = TEMP_DIR.join(&key);
        tokio::fs::create_dir_all(tmp_dir).await?;
        Ok(StartChunkResponse { key, upload_id })
    }

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

    pub async fn finish_chunk(
        &self,
        key: String,
        upload_id: String,
        part_list: Vec<PartItem>,
    ) -> Result<FinishResponse> {
        let tmp_dir = TEMP_DIR.join(&key);
        let chunks = part_list
            .into_iter()
            .map(|part| tmp_dir.join(part.part_number.to_string()));
        let (path_buf, relative_path) = self.build_paths().await?;
        let relative_path = format!("{}/{}", relative_path, upload_id);
        let filepath = path_buf.join(upload_id);

        let final_file = File::create(&filepath).await?;
        let mut writer = BufWriter::new(final_file);
        for chunk_path in chunks {
            let chunk_file = File::open(&chunk_path).await?;
            let mut reader = BufReader::new(chunk_file);
            let mut buffer = vec![0u8; 2 * 1024 * 1024]; // 2M 缓冲

            loop {
                let n = reader.read(&mut buffer).await?;
                if n == 0 {
                    break;
                }
                writer.write_all(&buffer[..n]).await?;
            }
        }
        writer.flush().await?;
        self.file_service.create(&relative_path).await?;
        Ok(FinishResponse {
            value: self.hman_util.sign_path(relative_path),
        })
    }

    async fn build_paths(&self) -> Result<(PathBuf, String)> {
        let now = self.ct.now();
        let year = now.year().to_string();
        let month = format!("{:02}", now.month());
        let path_buf = UPLOAD_DIR.join(&year).join(&month);
        tokio::fs::create_dir_all(&path_buf).await?;
        let relative_path = format!("{year}/{month}");
        Ok((path_buf, relative_path))
    }

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
        filepath: PathBuf,
    ) -> Result<()> {
        if let SupportedFormat::Webp = format {
            return persist_file(file, filepath.as_path()).await;
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
            let output_file = std::fs::File::create(filepath)?;
            let mut buf_writer = std::io::BufWriter::new(output_file);
            img.write_to(&mut buf_writer, ImageFormat::WebP)?;
            Ok(())
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
