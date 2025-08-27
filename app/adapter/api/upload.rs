use application::system::service::upload_service::{
    ChunkResponse, FinishResponse, StartChunkResponse, UploadService,
};
use axum::{Json, Router, extract::DefaultBodyLimit, routing::post};
use axum_typed_multipart::TypedMultipart;

use crate::{
    WebState,
    shared::{
        extractor::inject::Inject,
        response::{JsonResponse, JsonResponseType},
    },
};

async fn single(
    Inject(service): Inject<UploadService>,
    TypedMultipart(request): TypedMultipart<request::UploadRequest>,
) -> JsonResponseType<FinishResponse> {
    let resp = service
        .single(request.file.metadata.file_name, request.file.contents)
        .await?;
    JsonResponse::ok(resp)
}

async fn image(
    Inject(service): Inject<UploadService>,
    TypedMultipart(request): TypedMultipart<request::UploadRequest>,
) -> JsonResponseType<FinishResponse> {
    let resp = service.image(request.file.contents).await?;
    JsonResponse::ok(resp)
}

async fn start_chunk(
    Inject(service): Inject<UploadService>,
    Json(request): Json<request::StartChunkRequest>,
) -> JsonResponseType<StartChunkResponse> {
    let resp = service.start_chunk(request.filename).await?;
    JsonResponse::ok(resp)
}

async fn chunk(
    Inject(service): Inject<UploadService>,
    TypedMultipart(request): TypedMultipart<request::ChunkRequest>,
) -> JsonResponseType<ChunkResponse> {
    let resp = service
        .chunk(request.key, request.part_number, request.file.contents)
        .await?;
    JsonResponse::ok(resp)
}

async fn finish_chunk(
    Inject(service): Inject<UploadService>,
    Json(request): Json<request::FinishChunkRequest>,
) -> JsonResponseType<FinishResponse> {
    let resp = service
        .finish_chunk(request.key, request.upload_id, request.part_list)
        .await?;
    JsonResponse::ok(resp)
}

mod request {
    use application::system::service::upload_service::PartItem;
    use axum_typed_multipart::{FieldData, TryFromMultipart};
    use serde::Deserialize;
    use tempfile::NamedTempFile;

    #[derive(TryFromMultipart, Debug)]
    pub(crate) struct UploadRequest {
        #[form_data(limit = "2MiB")]
        /// max size: 2MiB
        pub file: FieldData<NamedTempFile>,
    }

    #[derive(Debug, Deserialize)]
    pub(crate) struct StartChunkRequest {
        pub filename: String,
    }

    #[derive(TryFromMultipart, Debug)]
    pub(crate) struct ChunkRequest {
        pub key: String,
        #[form_data(field_name = "partNumber")]
        pub part_number: u32,
        #[form_data(limit = "2MiB")]
        /// max size: 2MiB
        pub file: FieldData<NamedTempFile>,
    }

    #[derive(Debug, Deserialize)]
    pub(crate) struct FinishChunkRequest {
        pub key: String,
        #[serde(rename = "uploadId")]
        pub upload_id: String,
        #[serde(rename = "partList")]
        pub part_list: Vec<PartItem>,
    }
}

pub fn routing() -> Router<WebState> {
    Router::new()
        .route(
            "/single",
            post(single).route_layer(DefaultBodyLimit::max(3 * 1024 * 1024)),
        )
        .route(
            "/image",
            post(image).route_layer(DefaultBodyLimit::max(3 * 1024 * 1024)),
        )
        .route("/start_chunk", post(start_chunk))
        .route(
            "/chunk",
            post(chunk).route_layer(DefaultBodyLimit::max(3 * 1024 * 1024)),
        )
        .route("/finish_chunk", post(finish_chunk))
}
