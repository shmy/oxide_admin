use clap::Parser;
use infrastructure::shared::config::ConfigRef;
use serde::Deserialize;
use server::cli::Cli;
use testcontainers::{ContainerAsync, ImageExt as _, runners::AsyncRunner as _};
use testcontainers_modules::postgres::{self, Postgres};
use tokio::task::JoinHandle;

#[tokio::test]
async fn api_integration_test() {
    let (handle, base_url, _container) = setup_server().await;
    let variables = get_access_token(&base_url).await;
    run_hurl("auth", &variables).await;
    run_hurl("option", &variables).await;
    run_hurl("iam/user", &variables).await;
    run_hurl("iam/role", &variables).await;
    run_hurl("system", &variables).await;
    run_hurl("upload", &variables).await;
    run_hurl("last", &variables).await;
    handle.abort();
}

async fn setup_server() -> (JoinHandle<()>, String, ContainerAsync<Postgres>) {
    dotenvy::dotenv().ok();
    let container = postgres::Postgres::default()
        .with_tag("17-alpine")
        .start()
        .await
        .unwrap();
    let connection_string = format!(
        "postgresql://postgres:postgres@127.0.0.1:{}/postgres",
        container.get_host_port_ipv4(5432).await.unwrap()
    );
    let cli = Cli::parse_from(&["", "--database-url", &connection_string, "serve"]);
    let config: ConfigRef = cli.try_into().unwrap();
    let base_url = format!("http://{}:{}", config.server.bind, config.server.port);
    let handle = tokio::spawn(async move {
        server::serve(config).await.unwrap();
    });
    wait_for_server_health(&format!("{}/health", &base_url), 3).await;
    (handle, base_url, container)
}

async fn run_hurl(filename: &str, variables: &TestVariables) {
    let output = tokio::process::Command::new("hurl")
        .arg(&format!("tests/hurl/{}.hurl", filename))
        .arg("--very-verbose")
        .arg("--variable")
        .arg(&format!("base_url={}", variables.base_url))
        .arg("--variable")
        .arg(&format!("access_token={}", variables.access_token))
        .arg("--variable")
        .arg(&format!("refresh_token={}", variables.refresh_token))
        .output()
        .await
        .expect("failed to run hurl");

    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        panic!("Hurl test failed: {}", stderr);
    }
}

async fn get_access_token(base_url: &str) -> TestVariables {
    let res = reqwest::get(format!("{}/api/auth/refresh_captcha", base_url))
        .await
        .unwrap();
    let captcha_id = res
        .headers()
        .get("X-Captcha-Id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let res = reqwest::Client::new()
        .post(format!("{}/api/auth/sign_in", base_url))
        .json(&serde_json::json!({
            "account": "admin",
            "password": "123456",
            "captcha_key": captcha_id,
            "captcha_value": "2"
        }))
        .send()
        .await
        .unwrap()
        .json::<SignInResponse>()
        .await
        .unwrap();
    TestVariables {
        base_url: base_url.to_string(),
        access_token: res.data.access_token,
        refresh_token: res.data.refresh_token,
    }
}

async fn wait_for_server_health(url: &str, retries: u32) {
    for _ in 0..retries {
        if let Ok(resp) = reqwest::get(url).await {
            if resp.status() == 200 {
                return;
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    panic!("Health check failed {}", url);
}

struct TestVariables {
    base_url: String,
    access_token: String,
    refresh_token: String,
}

#[derive(Deserialize)]
struct SignInResponse {
    data: SignInResponseData,
}

#[derive(Deserialize)]
struct SignInResponseData {
    access_token: String,
    refresh_token: String,
}
