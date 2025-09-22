use clap::Parser;
use infrastructure::shared::config::ConfigRef;
use server::cli::Cli;
use tokio::task::JoinHandle;

async fn wait_for_health(url: &str, retries: u32) {
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

#[tokio::test]
async fn main() {
    let (handle, base_url) = setup_server().await;
    run_hurl("health", &base_url).await;
    handle.abort();
}

async fn setup_server() -> (JoinHandle<()>, String) {
    dotenvy::dotenv().ok();
    let cli = Cli::parse_from(&["", "serve"]);
    let config: ConfigRef = cli.try_into().unwrap();
    let base_url = format!("http://{}:{}", config.server.bind, config.server.port);
    let handle = tokio::spawn(async move {
        server::serve(config).await.unwrap();
    });
    wait_for_health(&format!("{}/health", &base_url), 3).await;
    (handle, base_url)
}

async fn run_hurl(filename: &str, base_url: &str) {
    let output = tokio::process::Command::new("hurl")
        .arg(&format!("tests/hurl/{}.hurl", filename))
        .arg("--variable")
        .arg(&format!("base_url={}", base_url))
        .output()
        .await
        .expect("failed to run hurl");

    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        panic!("Hurl test failed: {}", stderr);
    }
}
