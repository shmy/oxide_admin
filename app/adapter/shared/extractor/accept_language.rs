use std::str::FromStr;

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
    response::Response,
};
use axum_extra::extract::CookieJar;

pub const LANGUAGE_COOKIE_NAME: &str = "lang";

const DEFAULT_LANG_ID: i18n::LanguageIdentifier = i18n::langid!("en-US");

#[derive(Clone, Debug)]
pub struct AcceptLanguage(i18n::LanguageIdentifier);

impl<S> FromRequestParts<S> for AcceptLanguage
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookie_lang = CookieJar::from_request_parts(parts, state)
            .await
            .ok()
            .and_then(|jar| {
                jar.get(LANGUAGE_COOKIE_NAME)
                    .map(|cookie| cookie.value().to_owned())
            });

        let header_lang = parts
            .headers
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| accept_language::parse(s).first().cloned());

        let lang_id = cookie_lang.or(header_lang).and_then(|lang| {
            let lang_str = match lang.as_str() {
                "zh" => "zh-CN",
                "en" => "en-US",
                other => other,
            };
            i18n::LanguageIdentifier::from_str(lang_str).ok()
        });

        Ok(Self(lang_id.unwrap_or(DEFAULT_LANG_ID)))
    }
}

impl AcceptLanguage {
    pub fn identifier(&self) -> &i18n::LanguageIdentifier {
        &self.0
    }
}
