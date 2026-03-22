#![allow(unexpected_cfgs)]

use reqwest::Client;
use std::time::Duration;

const STEALTH_USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.3.1 Safari/605.1.15",
    "com.apple.trustd/3.0",
];

pub fn build_stealth_client(proxy_url: Option<&str>) -> Result<Client, String> {
    let ua = STEALTH_USER_AGENTS[0];

    let mut builder = Client::builder()
        .user_agent(ua)
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10));

    if let Some(proxy) = proxy_url {
        if !proxy.is_empty() {
            let proxy = reqwest::Proxy::all(proxy)
                .map_err(|e| format!("Invalid proxy URL: {e}"))?;
            builder = builder.proxy(proxy);
        }
    }

    builder.build().map_err(|e| format!("Failed to build HTTP client: {e}"))
}

pub async fn apply_jitter() {
    use rand::Rng;
    let delay_ms = {
        let mut rng = rand::thread_rng();
        rng.gen_range(50..500)
    };
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
}
