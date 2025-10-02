use application::system::service::upload_service::{
    ChunkResponse, FinishResponse, StartChunkResponse, UploadService,
};
use axum::{Json, extract::DefaultBodyLimit};
use axum_typed_multipart::TypedMultipart;
use domain::auth::value_object::permission::SYSTEM_FILE_UPLOAD;
use utoipa_axum::{
    router::{OpenApiRouter, UtoipaMethodRouterExt},
    routes,
};

use crate::{
    WebState, perms,
    shared::{
        extractor::inject::Inject,
        middleware::perm_router_ext::PermissonRouteExt,
        response::{JsonResponse, JsonResponseType},
    },
};

#[utoipa::path(
    post,
    path = "/single",
    request_body(
        content = inline(request::UploadRequest),
        content_type = "multipart/form-data"
    ),
    summary = "Upload a file",
    tag = "Upload",
    responses(
        (status = 200, body = inline(JsonResponse<FinishResponse>))
    )
)]
#[tracing::instrument]
async fn single(
    Inject(service): Inject<UploadService>,
    TypedMultipart(request): TypedMultipart<request::UploadRequest>,
) -> JsonResponseType<FinishResponse> {
    let resp = service
        .single(request.file.metadata.file_name, request.file.contents)
        .await?;
    JsonResponse::ok(resp)
}

#[utoipa::path(
    post,
    path = "/image",
    request_body(
        content = inline(request::UploadRequest),
        content_type = "multipart/form-data"
    ),
    summary = "Upload an image file",
    tag = "Upload",
    responses(
        (status = 200, body = inline(JsonResponse<FinishResponse>))
    )
)]
#[tracing::instrument]
async fn image(
    Inject(service): Inject<UploadService>,
    TypedMultipart(request): TypedMultipart<request::UploadRequest>,
) -> JsonResponseType<FinishResponse> {
    let resp = service
        .image(request.file.metadata.file_name, request.file.contents)
        .await?;
    JsonResponse::ok(resp)
}

#[utoipa::path(
    post,
    path = "/start_chunk",
    summary = "Start upload chunk",
    tag = "Upload",
    responses(
        (status = 200, body = inline(JsonResponse<StartChunkResponse>))
    )
)]
#[tracing::instrument]
async fn start_chunk(
    Inject(service): Inject<UploadService>,
    Json(request): Json<request::StartChunkRequest>,
) -> JsonResponseType<StartChunkResponse> {
    let resp = service.start_chunk(request.filename).await?;
    JsonResponse::ok(resp)
}

#[utoipa::path(
    post,
    path = "/chunk",
    request_body(
        content = inline(request::ChunkRequest),
        content_type = "multipart/form-data"
    ),
    summary = "Chunk upload file",
    tag = "Upload",
    responses(
        (status = 200, body = inline(JsonResponse<ChunkResponse>))
    )
)]
#[tracing::instrument]
async fn chunk(
    Inject(service): Inject<UploadService>,
    TypedMultipart(request): TypedMultipart<request::ChunkRequest>,
) -> JsonResponseType<ChunkResponse> {
    let resp = service
        .chunk(request.key, request.part_number, request.file.contents)
        .await?;
    JsonResponse::ok(resp)
}

#[utoipa::path(
    post,
    path = "/finish_chunk",
    summary = "Finish upload chunk",
    tag = "Upload",
    responses(
        (status = 200, body = inline(JsonResponse<FinishResponse>))
    )
)]
#[tracing::instrument]
async fn finish_chunk(
    Inject(service): Inject<UploadService>,
    Json(request): Json<request::FinishChunkRequest>,
) -> JsonResponseType<FinishResponse> {
    let resp = service
        .finish_chunk(
            request.filename,
            request.key,
            request.upload_id,
            request.part_list,
        )
        .await?;
    JsonResponse::ok(resp)
}

mod request {
    use application::system::service::upload_service::PartItem;
    use axum_typed_multipart::{FieldData, TryFromMultipart};
    use serde::Deserialize;
    use tempfile::NamedTempFile;
    use utoipa::ToSchema;

    #[derive(TryFromMultipart, Debug, ToSchema)]
    pub(crate) struct UploadRequest {
        #[form_data(limit = "2MiB")]
        #[schema(value_type = Vec<u8>)]
        /// max size: 2MiB
        pub file: FieldData<NamedTempFile>,
    }

    #[derive(Debug, Deserialize, ToSchema)]
    pub(crate) struct StartChunkRequest {
        pub filename: String,
    }

    #[derive(TryFromMultipart, Debug, ToSchema)]
    pub(crate) struct ChunkRequest {
        pub key: String,
        #[form_data(field_name = "partNumber")]
        #[schema(rename = "partNumber")]
        pub part_number: u32,
        #[form_data(limit = "2MiB")]
        #[schema(value_type = Vec<u8>)]
        /// max size: 2MiB
        pub file: FieldData<NamedTempFile>,
    }

    #[derive(Debug, Deserialize, ToSchema)]
    pub(crate) struct FinishChunkRequest {
        pub filename: String,
        pub key: String,
        #[serde(rename = "uploadId")]
        pub upload_id: String,
        #[serde(rename = "partList")]
        pub part_list: Vec<PartItem>,
    }
}

pub fn routing() -> OpenApiRouter<WebState> {
    OpenApiRouter::new()
        .routes(routes!(single).layer(DefaultBodyLimit::max(3 * 1024 * 1024)))
        .routes(routes!(image).layer(DefaultBodyLimit::max(3 * 1024 * 1024)))
        .routes(routes!(chunk).layer(DefaultBodyLimit::max(3 * 1024 * 1024)))
        .routes(routes!(start_chunk))
        .routes(routes!(finish_chunk))
        .permit_all(perms!(SYSTEM_FILE_UPLOAD))
}
