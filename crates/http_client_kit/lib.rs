pub use reqwest::header;
use reqwest::Client;
pub use reqwest::Error as ReqwestError;
pub use reqwest_middleware::Error as ReqwestMiddlewareError;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::sync::LazyLock;

pub struct HttpClient;

pub static HTTP_CLIENT: LazyLock<reqwest_middleware::ClientWithMiddleware> = LazyLock::new(|| {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    reqwest_middleware::ClientBuilder::new(Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
});

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn it_text_works() {
        let result: String = HTTP_CLIENT
            .get("https://httpbin.org/robots.txt")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert!(&result.contains("Disallow"));
    }

    #[tokio::test]
    async fn it_html_works() {
        let result: String = HTTP_CLIENT
            .get("https://httpbin.org/html")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert!(&result.contains("Herman Melville"));
    }

    #[tokio::test]
    async fn it_json_works() {
        let result: serde_json::Value = HTTP_CLIENT
            .get("https://httpbin.org/json")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(&result.get("slideshow").unwrap().as_object().is_some());
    }

    #[tokio::test]
    #[cfg(feature = "deflate")]
    async fn it_deflate_works() {
        let result: serde_json::Value = HTTP_CLIENT
            .get("https://httpbin.org/deflate")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(&result.get("deflated").unwrap().as_bool().unwrap());
    }

    #[tokio::test]
    #[cfg(feature = "gzip")]
    async fn it_gzip_works() {
        let result: serde_json::Value = HTTP_CLIENT
            .get("https://httpbin.org/gzip")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(&result.get("gzipped").unwrap().as_bool().unwrap());
    }

    #[tokio::test]
    #[cfg(feature = "brotli")]
    async fn it_brotli_works() {
        let result: serde_json::Value = HTTP_CLIENT
            .get("https://httpbin.org/brotli")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(&result.get("brotli").unwrap().as_bool().unwrap());
    }
}
