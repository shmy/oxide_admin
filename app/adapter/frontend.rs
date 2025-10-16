use askama::Template;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_ENCODING, CONTENT_TYPE};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use i18n::LanguageIdentifier;
use include_dir::{Dir, File, include_dir};

use crate::shared::extractor::accept_language::AcceptLanguage;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    locale: &'a LanguageIdentifier,
}

#[derive(Template)]
#[template(path = "sign_in.html")]
struct SignInTemplate<'a> {
    locale: &'a LanguageIdentifier,
}

const CONTENT_ENCODING_EXTENSION: &str = include_str!("../../frontend/dist/.EXTENSION");
const CONTENT_ENCODING_VALUE: &str = include_str!("../../frontend/dist/.ENCODING");
const NO_CONTENT_ENCODING_VALUE: &str = "identity";
const CACHE_CONTROL_VALUE: &str = "public,max-age=3600";

static FRONTEND_STATIC_DIR: Dir<'_> = include_dir!("./frontend/dist/static");

async fn index(language: AcceptLanguage) -> impl IntoResponse {
    let template = IndexTemplate {
        locale: language.identifier(),
    };
    if let Ok(content) = template.render() {
        return Html(content).into_response();
    }

    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

async fn sign_in(language: AcceptLanguage) -> impl IntoResponse {
    let template = SignInTemplate {
        locale: language.identifier(),
    };
    if let Ok(content) = template.render() {
        return Html(content).into_response();
    }

    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

struct FileResult<'a> {
    file: &'a File<'a>,
    compressed: bool,
}

async fn asset(Path(path): Path<String>) -> impl IntoResponse {
    let compressed_path = format!("{}.{}", &path, CONTENT_ENCODING_EXTENSION);
    if let Some(result) = FRONTEND_STATIC_DIR
        .get_file(&compressed_path)
        .map(|file| FileResult {
            file,
            compressed: true,
        })
        .or_else(|| {
            FRONTEND_STATIC_DIR.get_file(&path).map(|file| FileResult {
                file,
                compressed: false,
            })
        })
    {
        let guess = mime_guess::from_path(&path).first_or_octet_stream();
        let content_encoding = if result.compressed {
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
