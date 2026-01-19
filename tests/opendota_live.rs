use dota2_tui::api::ApiClient;
use dota2_tui::config::ApiConfig;

fn live_enabled() -> bool {
    std::env::var("OPENDOTA_LIVE").ok().as_deref() == Some("1")
}

fn account_id() -> u32 {
    std::env::var("OPENDOTA_ACCOUNT_ID")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(135664392)
}

fn api_config() -> ApiConfig {
    let mut config = ApiConfig::default();
    if let Ok(base) = std::env::var("OPENDOTA_BASE_URL") {
        config.base_url = base;
    }
    config.log_requests = true;
    config
}

#[tokio::test]
async fn fetch_profile_live() {
    if !live_enabled() {
        return;
    }
    let client = ApiClient::new(api_config());
    let profile = client.fetch_profile(account_id()).await;
    assert!(profile.is_ok(), "profile request failed: {:?}", profile);
}

#[tokio::test]
async fn fetch_matches_live() {
    if !live_enabled() {
        return;
    }
    let client = ApiClient::new(api_config());
    let matches = client.fetch_matches(account_id()).await;
    assert!(matches.is_ok(), "matches request failed: {:?}", matches);
}
