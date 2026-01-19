use std::collections::{HashMap, VecDeque};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::sync::{Mutex, Semaphore};

use crate::config::ApiConfig;
use crate::models::{HeroConstant, HeroStat, ItemConstant, MatchDetail, PlayerMatch, PlayerResponse};

#[derive(Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    cache: Arc<Mutex<Cache>>,
    inflight: Arc<Semaphore>,
    log_path: Option<PathBuf>,
}

impl ApiClient {
    pub fn new(config: ApiConfig) -> Self {
        let log_path = config.resolve_log_path().ok().flatten();
        Self {
            client: reqwest::Client::builder()
                .user_agent("dota2_tui")
                .timeout(Duration::from_secs(20))
                .connect_timeout(Duration::from_secs(8))
                .http1_only()
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            base_url: config.base_url,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(
                config.rate_limit_per_minute,
            ))),
            cache: Arc::new(Mutex::new(Cache::new(
                config.cache_max_entries,
                Duration::from_secs(config.cache_ttl_secs),
            ))),
            inflight: Arc::new(Semaphore::new(config.max_inflight.max(1))),
            log_path,
        }
    }

    pub async fn fetch_heroes(&self) -> Result<HashMap<i32, String>> {
        let url = format!("{}/heroStats", self.base_url);
        let heroes: Vec<HeroStat> = self.get_json(url, None).await?;
        Ok(heroes
            .into_iter()
            .map(|hero| (hero.id, hero.localized_name))
            .collect())
    }

    pub async fn fetch_profile(&self, account_id: u32) -> Result<PlayerResponse> {
        let url = format!("{}/players/{account_id}", self.base_url);
        self.get_json(url, None).await
    }

    pub async fn fetch_matches(&self, account_id: u32) -> Result<Vec<PlayerMatch>> {
        let url = format!("{}/players/{account_id}/recentMatches", self.base_url);
        let primary = self.get_json(url, None).await;
        if primary.is_ok() {
            return primary;
        }
        let fallback = format!("{}/players/{account_id}/matches", self.base_url);
        self.log_line(format!(
            "fallback matches for account_id={}",
            account_id
        ));
        self.get_json(
            fallback,
            Some(vec![("limit", "20".to_string()), ("significant", "0".to_string())]),
        )
        .await
    }

    pub async fn fetch_match_detail(&self, match_id: u64) -> Result<MatchDetail> {
        let url = format!("{}/matches/{match_id}", self.base_url);
        self.get_json(url, None).await
    }

    pub async fn fetch_hero_constants(&self) -> Result<HashMap<i32, HeroConstant>> {
        let url = format!("{}/constants/heroes", self.base_url);
        let raw: HashMap<String, HeroConstant> = self.get_json(url, None).await?;
        Ok(raw.into_values().map(|hero| (hero.id, hero)).collect())
    }

    pub async fn fetch_item_constants(&self) -> Result<HashMap<i32, ItemConstant>> {
        let url = format!("{}/constants/items", self.base_url);
        let raw: HashMap<String, ItemConstant> = self.get_json(url, None).await?;
        Ok(raw
            .into_values()
            .filter(|item| item.id != 0)
            .map(|item| (item.id, item))
            .collect())
    }

    pub async fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        Ok(response.to_vec())
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: String,
        query: Option<Vec<(&str, String)>>,
    ) -> Result<T> {
        let mut req = self.client.get(&url);
        if let Some(params) = &query {
            let owned: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            req = req.query(&owned);
        }

        let cache_key = build_cache_key(&url, query.as_ref());
        let cached = self.cache.lock().await.get(&cache_key);
        if let Some((cached, true)) = cached.as_ref() {
            return Ok(serde_json::from_slice(cached)?);
        }

        let mut attempt = 0;
        loop {
            attempt += 1;
            let _permit = self.inflight.acquire().await.ok();
            let wait = self.rate_limiter.lock().await.acquire().await;
            if let Some(waited) = wait {
                self.log_line(format!(
                    "rate_limit_wait_ms={}",
                    waited.as_millis()
                ));
            }

            let started = Instant::now();
            let response = req.try_clone().unwrap_or_else(|| self.client.get(&url)).send().await;
            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if !status.is_success() {
                        let elapsed = started.elapsed().as_millis();
                        self.log_line(format!("GET {} status={} elapsed_ms={}", cache_key, status, elapsed));
                        if attempt < 2 {
                            tokio::time::sleep(backoff(attempt)).await;
                            continue;
                        }
                        if let Some((stale, _)) = cached.as_ref() {
                            self.log_line(format!("using_stale_cache {}", cache_key));
                            return Ok(serde_json::from_slice(stale)?);
                        }
                        return Err(anyhow::anyhow!("HTTP {}", status));
                    }
                    let body = resp.bytes().await?;
                    let elapsed = started.elapsed().as_millis();
                    self.log_line(format!(
                        "GET {} status=200 elapsed_ms={}",
                        cache_key, elapsed
                    ));
                    let bytes = body.to_vec();
                    self.cache.lock().await.set(cache_key, bytes.clone());
                    return Ok(serde_json::from_slice(&bytes)?);
                }
                Err(err) => {
                    self.log_line(format!(
                        "GET {} error=\"{}\" attempt={}",
                        cache_key, err, attempt
                    ));
                    if attempt < 2 && (err.is_timeout() || err.is_connect() || err.is_request()) {
                        tokio::time::sleep(backoff(attempt)).await;
                        continue;
                    }
                    if let Some((stale, _)) = cached.as_ref() {
                        self.log_line(format!("using_stale_cache {}", cache_key));
                        return Ok(serde_json::from_slice(stale)?);
                    }
                    return Err(err.into());
                }
            }
        }
    }

    fn log_line(&self, line: String) {
        let Some(path) = &self.log_path else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{}", line);
        }
    }
}

fn backoff(attempt: u32) -> Duration {
    match attempt {
        1 => Duration::from_millis(200),
        2 => Duration::from_millis(500),
        _ => Duration::from_millis(900),
    }
}

fn build_cache_key(url: &str, query: Option<&Vec<(&str, String)>>) -> String {
    if let Some(params) = query {
        let mut pairs: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        pairs.sort();
        format!("{url}?{}", pairs.join("&"))
    } else {
        url.to_string()
    }
}

struct CacheEntry {
    inserted_at: Instant,
    payload: Vec<u8>,
}

struct Cache {
    entries: HashMap<String, CacheEntry>,
    order: VecDeque<String>,
    ttl: Duration,
    max_entries: usize,
}

impl Cache {
    fn new(max_entries: usize, ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            ttl,
            max_entries,
        }
    }

    fn get(&mut self, key: &str) -> Option<(Vec<u8>, bool)> {
        if let Some(entry) = self.entries.get(key) {
            let fresh = entry.inserted_at.elapsed() <= self.ttl;
            return Some((entry.payload.clone(), fresh));
        }
        None
    }

    fn set(&mut self, key: String, payload: Vec<u8>) {
        if self.entries.contains_key(&key) {
            self.order.retain(|k| k != &key);
        }
        self.entries.insert(
            key.clone(),
            CacheEntry {
                inserted_at: Instant::now(),
                payload,
            },
        );
        self.order.push_front(key);
        while self.order.len() > self.max_entries {
            if let Some(old) = self.order.pop_back() {
                self.entries.remove(&old);
            }
        }
    }
}

struct RateLimiter {
    per_minute: u32,
    tokens: u32,
    last_refill: Instant,
}

impl RateLimiter {
    fn new(per_minute: u32) -> Self {
        Self {
            per_minute,
            tokens: per_minute,
            last_refill: Instant::now(),
        }
    }

    async fn acquire(&mut self) -> Option<Duration> {
        self.refill();
        if self.tokens == 0 {
            let wait = Duration::from_secs(60).saturating_sub(self.last_refill.elapsed());
            tokio::time::sleep(wait).await;
            self.refill();
            return Some(wait);
        }
        if self.tokens > 0 {
            self.tokens -= 1;
        }
        None
    }

    fn refill(&mut self) {
        if self.last_refill.elapsed() >= Duration::from_secs(60) {
            self.tokens = self.per_minute;
            self.last_refill = Instant::now();
        }
    }
}
