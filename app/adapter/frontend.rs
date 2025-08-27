use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_ENCODING, CONTENT_TYPE};
use axum::response::IntoResponse;
use axum::routing::get;
use include_dir::{Dir, File, include_dir};

const CONTENT_ENCODING_EXTENSION: &str = "gz";
const CONTENT_ENCODING_VALUE: &str = "gzip";
const NO_CONTENT_ENCODING_VALUE: &str = "identity";
const CACHE_CONTROL_VALUE: &str = "public,max-age=3600";

static FRONTEND_STATIC_DIR: Dir<'_> = include_dir!("./frontend/dist/static");

async fn index() -> impl IntoResponse {
    (
        StatusCode::OK,
        [
            (CONTENT_TYPE, "text/html"),
            (CONTENT_ENCODING, CONTENT_ENCODING_VALUE),
            (CACHE_CONTROL, CACHE_CONTROL_VALUE),
        ],
        include_bytes!("../../frontend/dist/index.html.gz"),
    )
        .into_response()
}

async fn sign_in() -> impl IntoResponse {
    (
        StatusCode::OK,
        [
            (CONTENT_TYPE, "text/html"),
            (CONTENT_ENCODING, CONTENT_ENCODING_VALUE),
            (CACHE_CONTROL, CACHE_CONTROL_VALUE),
        ],
        include_bytes!("../../frontend/dist/sign_in.html.gz"),
    )
        .into_response()
}

struct FileResult<'a> {
    file: &'a File<'a>,
    gzip: bool,
}

async fn asset(Path(path): Path<String>) -> impl IntoResponse {
    let gziped_path = format!("{}.{}", &path, CONTENT_ENCODING_EXTENSION);
    if let Some(result) = FRONTEND_STATIC_DIR
        .get_file(&gziped_path)
        .map(|file| FileResult { file, gzip: true })
        .or_else(|| {
            FRONTEND_STATIC_DIR
                .get_file(&path)
                .map(|file| FileResult { file, gzip: false })
        })
    {
        let guess = mime_guess::from_path(&path).first_or_octet_stream();
        let content_encoding = if result.gzip {
            CONTENT_ENCODING_VALUE
        } else {
            NO_CONTENT_ENCODING_VALUE
        };

        return (
            StatusCode::OK,
            [
                (CONTENT_TYPE, guess.as_ref()),
                (CONTENT_ENCODING, content_encoding),
                (CACHE_CONTROL, CACHE_CONTROL_VALUE),
            ],
            result.file.contents(),
        )
            .into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}

pub fn routing() -> Router {
    let router = Router::new()
        .route("/", get(index))
        .route("/sign_in", get(sign_in))
        .route("/static/{*path}", get(asset))
        .fallback(index);

    Router::new()
        .route(
            "/_/",
            axum::routing::get(|| async { axum::response::Redirect::permanent("/_") }),
        )
        .nest("/_", router)
}
