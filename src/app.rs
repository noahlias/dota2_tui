use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::time::{Duration, Instant};

use anyhow::Result;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::api::ApiClient;
use crate::config::{cache_dir, recent_log_path};
use crate::i18n::I18n;
use crate::models::{MatchDetail, PlayerMatch, PlayerResponse};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SearchEntry {
    pub account_id: u32,
    pub personaname: String,
    pub avatar_url: Option<String>,
}

pub struct App {
    pub input: String,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub status: String,
    pub account_id: Option<u32>,
    pub profile: Option<PlayerResponse>,
    pub matches: Vec<PlayerMatch>,
    pub match_state: ListState,
    pub match_detail: Option<MatchDetail>,
    pub recent_searches: Vec<SearchEntry>,
    pub recent_index: Option<usize>,
    pub recent_active: bool,
    pub heroes: HashMap<i32, String>,
    pub hero_images: HashMap<i32, String>,
    pub item_images: HashMap<i32, String>,
    pub image_cache: HashMap<String, Vec<u8>>,
    pub image_cache_order: VecDeque<String>,
    pub image_cache_max: usize,
    pub player_avatars: HashMap<u32, String>,
    pub player_avatar_requests: HashSet<u32>,
    pub pending_player_avatar_ids: Vec<u32>,
    pub image_inflight: HashSet<String>,
    pub loading: bool,
    pub detail_loading: bool,
    pub avatar_url: Option<String>,
    pub avatar_loading: bool,
    pub tab_index: usize,
    pub tick: u64,
    pub show_help: bool,
    pub banner_shimmer: u8,
    pub requested_hero_images: bool,
    pub requested_item_images: bool,
    pub image_reset: bool,
    pub last_nav: Instant,
    pub net_total: usize,
    pub net_done: usize,
    pub net_inflight: usize,
    pub net_last_ms: Option<u128>,
}

impl App {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            should_quit: false,
            status: String::new(),
            account_id: None,
            profile: None,
            matches: Vec::new(),
            match_state: state,
            match_detail: None,
            recent_searches: Vec::new(),
            recent_index: None,
            recent_active: false,
            heroes: HashMap::new(),
            hero_images: HashMap::new(),
            item_images: HashMap::new(),
            image_cache: HashMap::new(),
            image_cache_order: VecDeque::new(),
            image_cache_max: 256,
            player_avatars: HashMap::new(),
            player_avatar_requests: HashSet::new(),
            pending_player_avatar_ids: Vec::new(),
            image_inflight: HashSet::new(),
            loading: false,
            detail_loading: false,
            avatar_url: None,
            avatar_loading: false,
            tab_index: 0,
            tick: 0,
            show_help: false,
            banner_shimmer: 24,
            requested_hero_images: false,
            requested_item_images: false,
            image_reset: false,
            last_nav: Instant::now().checked_sub(Duration::from_secs(1)).unwrap_or_else(Instant::now),
            net_total: 0,
            net_done: 0,
            net_inflight: 0,
            net_last_ms: None,
        }
    }

    pub fn selected_match(&self) -> Option<&PlayerMatch> {
        self.match_state
            .selected()
            .and_then(|idx| self.matches.get(idx))
    }

    pub fn hero_name(&self, hero_id: i32, i18n: &I18n) -> String {
        self.heroes
            .get(&hero_id)
            .map(|s| s.as_str())
            .unwrap_or(i18n.unknown())
            .to_string()
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status = msg.into();
    }

    pub fn advance_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        if self.banner_shimmer > 0 {
            self.banner_shimmer = self.banner_shimmer.saturating_sub(1);
        }
    }

    pub fn clear_matches(&mut self) {
        self.matches.clear();
        self.match_state.select(Some(0));
        self.match_detail = None;
    }

    pub fn cache_image(&mut self, url: String, bytes: Vec<u8>) {
        if self.image_cache.contains_key(&url) {
            self.image_cache_order.retain(|key| key != &url);
        }
        self.image_cache.insert(url.clone(), bytes);
        self.image_cache_order.push_front(url);
        while self.image_cache_order.len() > self.image_cache_max {
            if let Some(old) = self.image_cache_order.pop_back() {
                self.image_cache.remove(&old);
            }
        }
    }
}

pub enum Message {
    HeroesLoaded(Result<HashMap<i32, String>>),
    HeroImagesLoaded(Result<HashMap<i32, String>>),
    ItemImagesLoaded(Result<HashMap<i32, String>>),
    SearchLoaded(Result<SearchPayload>),
    MatchDetailLoaded(Result<MatchDetail>),
    ImageLoaded { url: String, result: Result<Vec<u8>> },
    PlayerAvatarLoaded { account_id: u32, result: Result<Option<String>> },
    NetEvent { elapsed_ms: u128 },
}

pub struct SearchPayload {
    pub account_id: u32,
    pub profile: Option<PlayerResponse>,
    pub matches: Vec<PlayerMatch>,
    pub profile_error: Option<String>,
    pub match_error: Option<String>,
}

pub fn handle_message(msg: Message, app: &mut App, i18n: &I18n) -> Option<String> {
    let mut avatar_request = None;
    match msg {
        Message::HeroesLoaded(result) => match result {
            Ok(heroes) => {
                app.heroes = heroes;
                app.set_status(i18n.status_hero_loaded());
            }
            Err(err) => {
                app.set_status(i18n.status_hero_failed(&err.to_string()));
            }
        },
        Message::HeroImagesLoaded(result) => match result {
            Ok(images) => {
                app.hero_images = images;
            }
            Err(err) => {
                app.set_status(i18n.status_hero_failed(&err.to_string()));
            }
        },
        Message::ItemImagesLoaded(result) => match result {
            Ok(images) => {
                app.item_images = images;
            }
            Err(err) => {
                app.set_status(i18n.status_hero_failed(&err.to_string()));
            }
        },
        Message::SearchLoaded(result) => {
            app.loading = false;
            match result {
                Ok(payload) => {
                    app.account_id = Some(payload.account_id);
                    app.profile = payload.profile;
                    app.matches = payload.matches;
                    app.avatar_loading = false;
                    app.avatar_url = app
                        .profile
                        .as_ref()
                        .and_then(|profile| profile.profile.as_ref())
                        .and_then(|profile| {
                            profile
                                .avatarfull
                                .as_deref()
                                .or_else(|| profile.avatarmedium.as_deref())
                                .or_else(|| profile.avatar.as_deref())
                        })
                        .map(|url| url.to_string());
                    if app.avatar_url.is_none() {
                        if let Some(account_id) = app.account_id {
                            if let Some(url) = app.player_avatars.get(&account_id) {
                                app.avatar_url = Some(url.clone());
                            }
                        }
                    }
                    if let (Some(account_id), Some(url)) = (app.account_id, app.avatar_url.clone()) {
                        app.player_avatars.insert(account_id, url);
                        save_avatar_map(&app.player_avatars);
                    }
                    if let Some(url) = app.avatar_url.clone() {
                        app.avatar_loading = true;
                        avatar_request = Some(url);
                    }
                    if let Some(err) = payload.match_error {
                        app.set_status(i18n.status_matches_failed(&err));
                        app.match_state.select(None);
                    } else if app.matches.is_empty() {
                        app.match_state.select(None);
                        app.set_status(i18n.status_no_matches());
                    } else {
                        app.match_state.select(Some(0));
                        app.set_status(i18n.status_matches_loaded());
                    }
                    if let Some(err) = payload.profile_error {
                        app.set_status(i18n.status_profile_failed(&err));
                    }
                    if let Some(profile) = app.profile.as_ref().and_then(|p| p.profile.as_ref()) {
                        let name = profile
                            .personaname
                            .as_deref()
                            .unwrap_or(i18n.unknown())
                            .to_string();
                        let avatar = profile
                            .avatarfull
                            .clone()
                            .or_else(|| profile.avatarmedium.clone())
                            .or_else(|| profile.avatar.clone());
                        if let Some(account_id) = app.account_id {
                            push_recent_search(&mut app.recent_searches, account_id, name, avatar);
                        }
                    }
                }
                Err(err) => {
                    app.set_status(i18n.status_search_failed(&err.to_string()));
                }
            }
        }
        Message::MatchDetailLoaded(result) => {
            app.detail_loading = false;
            match result {
                Ok(detail) => {
                    app.match_detail = Some(detail);
                    app.set_status(i18n.status_match_loaded());
                    if let Some(detail) = app.match_detail.as_ref() {
                        let disk_map = load_avatar_map();
                        for (id, url) in disk_map {
                            app.player_avatars.entry(id).or_insert(url);
                        }
                        let ids: Vec<u32> = detail
                            .players
                            .iter()
                            .filter_map(|p| p.account_id)
                            .collect();
                        let missing: Vec<u32> = ids
                            .into_iter()
                            .filter(|id| {
                                !app.player_avatars.contains_key(id)
                                    && app.player_avatar_requests.insert(*id)
                            })
                            .collect();
                        if !missing.is_empty() {
                            app.pending_player_avatar_ids = missing;
                        }
                    }
                }
                Err(err) => {
                    app.set_status(i18n.status_match_failed(&err.to_string()));
                }
            }
        }
        Message::ImageLoaded { url, result } => {
            app.avatar_loading = false;
            app.image_inflight.remove(&url);
            match result {
                Ok(bytes) => {
                    app.cache_image(url, bytes);
                }
                Err(err) => {
                    app.set_status(i18n.status_image_failed(&err.to_string()));
                }
            }
        }
        Message::PlayerAvatarLoaded { account_id, result } => {
            app.player_avatar_requests.remove(&account_id);
            if let Ok(Some(url)) = result {
                app.player_avatars.insert(account_id, url);
                save_avatar_map(&app.player_avatars);
            }
        }
        Message::NetEvent { elapsed_ms } => {
            app.net_last_ms = Some(elapsed_ms);
            if app.net_inflight > 0 {
                app.net_inflight -= 1;
            }
            app.net_done = app.net_done.saturating_add(1);
        }
    }
    app.avatar_loading = !app.image_inflight.is_empty();
    avatar_request
}

pub fn spawn_hero_load(tx: mpsc::Sender<Message>, api: ApiClient) {
    tokio::spawn(async move {
        let started = Instant::now();
        let result = api.fetch_heroes().await;
        let _ = tx.send(Message::HeroesLoaded(result)).await;
        let _ = tx
            .send(Message::NetEvent {
                elapsed_ms: started.elapsed().as_millis(),
            })
            .await;
    });
}

pub fn spawn_hero_images(tx: mpsc::Sender<Message>, api: ApiClient, cdn_base: String) {
    tokio::spawn(async move {
        let started = Instant::now();
        let result = api
            .fetch_hero_constants()
            .await
            .map(|heroes| build_asset_map(heroes, &cdn_base, |hero| hero.img.clone()));
        let _ = tx.send(Message::HeroImagesLoaded(result)).await;
        let _ = tx
            .send(Message::NetEvent {
                elapsed_ms: started.elapsed().as_millis(),
            })
            .await;
    });
}

pub fn spawn_item_images(tx: mpsc::Sender<Message>, api: ApiClient, cdn_base: String) {
    tokio::spawn(async move {
        let started = Instant::now();
        let result = api
            .fetch_item_constants()
            .await
            .map(|items| build_asset_map(items, &cdn_base, |item| item.img.clone()));
        let _ = tx.send(Message::ItemImagesLoaded(result)).await;
        let _ = tx
            .send(Message::NetEvent {
                elapsed_ms: started.elapsed().as_millis(),
            })
            .await;
    });
}

pub fn spawn_search(tx: mpsc::Sender<Message>, api: ApiClient, account_id: u32) {
    tokio::spawn(async move {
        let started = Instant::now();
        let profile_task = api.fetch_profile(account_id);
        let matches_task = api.fetch_matches(account_id);
        let (profile_result, matches_result) = tokio::join!(profile_task, matches_task);
        let (profile, profile_error) = match profile_result {
            Ok(profile) => (Some(profile), None),
            Err(err) => (None, Some(err.to_string())),
        };
        let (matches, match_error) = match matches_result {
            Ok(matches) => (matches, None),
            Err(err) => (Vec::new(), Some(err.to_string())),
        };
        let payload = Ok(SearchPayload {
            account_id,
            profile,
            matches,
            profile_error,
            match_error,
        });
        let _ = tx.send(Message::SearchLoaded(payload)).await;
        let _ = tx
            .send(Message::NetEvent {
                elapsed_ms: started.elapsed().as_millis(),
            })
            .await;
    });
}

pub fn spawn_match_detail(tx: mpsc::Sender<Message>, api: ApiClient, match_id: u64) {
    tokio::spawn(async move {
        let started = Instant::now();
        let result = api.fetch_match_detail(match_id).await;
        let _ = tx.send(Message::MatchDetailLoaded(result)).await;
        let _ = tx
            .send(Message::NetEvent {
                elapsed_ms: started.elapsed().as_millis(),
            })
            .await;
    });
}

pub fn spawn_image_fetch(tx: mpsc::Sender<Message>, api: ApiClient, url: String) {
    tokio::spawn(async move {
        let result = api.fetch_bytes(&url).await;
        let _ = tx
            .send(Message::ImageLoaded { url, result })
            .await;
    });
}

pub fn spawn_player_avatars(tx: mpsc::Sender<Message>, api: ApiClient, account_ids: Vec<u32>) {
    tokio::spawn(async move {
        for account_id in account_ids {
            let result = match api.fetch_profile(account_id).await {
                Ok(profile) => {
                    let url = profile
                        .profile
                        .as_ref()
                        .and_then(|p| {
                            p.avatarfull
                                .as_deref()
                                .or_else(|| p.avatarmedium.as_deref())
                                .or_else(|| p.avatar.as_deref())
                        })
                        .map(|value| value.to_string());
                    Ok(url)
                }
                Err(err) => Err(err),
            };
            let _ = tx
                .send(Message::PlayerAvatarLoaded { account_id, result })
                .await;
        }
    });
}

pub fn load_recent_searches(max_entries: usize) -> Vec<SearchEntry> {
    let path = match recent_log_path() {
        Ok(path) => path,
        Err(_) => return Vec::new(),
    };
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return Vec::new(),
    };
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    for line in reader.lines().flatten() {
        if let Ok(entry) = serde_json::from_str::<SearchEntry>(&line) {
            entries.push(entry);
        }
    }
    let mut seen = HashSet::new();
    let mut recent = Vec::new();
    for entry in entries.into_iter().rev() {
        if seen.insert(entry.account_id) {
            recent.push(entry);
            if recent.len() >= max_entries {
                break;
            }
        }
    }
    recent
}

pub fn load_avatar_map() -> HashMap<u32, String> {
    let path = match avatar_map_path() {
        Ok(path) => path,
        Err(_) => return HashMap::new(),
    };
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => return HashMap::new(),
    };
    let raw: HashMap<String, String> = serde_json::from_slice(&bytes).unwrap_or_default();
    let mut map = HashMap::new();
    for (key, value) in raw {
        if let Ok(id) = key.parse::<u32>() {
            map.insert(id, value);
        }
    }
    map
}

pub fn save_avatar_map(map: &HashMap<u32, String>) {
    let path = match avatar_map_path() {
        Ok(path) => path,
        Err(_) => return,
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(bytes) = serde_json::to_vec(map) {
        let _ = std::fs::write(path, bytes);
    }
}

pub fn append_recent_search(entry: &SearchEntry) {
    let path = match recent_log_path() {
        Ok(path) => path,
        Err(_) => return,
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut file = match OpenOptions::new().create(true).append(true).open(path) {
        Ok(file) => file,
        Err(_) => return,
    };
    if let Ok(line) = serde_json::to_string(entry) {
        let _ = writeln!(file, "{}", line);
    }
}

fn avatar_map_path() -> Result<std::path::PathBuf> {
    let mut base = cache_dir()?;
    base.push("avatar_map.json");
    Ok(base)
}

fn build_asset_map<T: Clone>(
    raw: HashMap<i32, T>,
    cdn_base: &str,
    image: impl Fn(&T) -> Option<String>,
) -> HashMap<i32, String> {
    raw.into_iter()
        .filter_map(|(id, entry)| {
            let img = image(&entry)?;
            let url = if img.starts_with("http") {
                img
            } else {
                format!("{}{}", cdn_base.trim_end_matches('/'), img)
            };
            Some((id, url))
        })
        .collect()
}

fn push_recent_search(
    recent: &mut Vec<SearchEntry>,
    account_id: u32,
    name: String,
    avatar_url: Option<String>,
) {
    let entry = SearchEntry {
        account_id,
        personaname: name,
        avatar_url,
    };
    append_recent_search(&entry);
    if let Some(pos) = recent.iter().position(|item| item.account_id == account_id) {
        recent.remove(pos);
    }
    recent.insert(0, entry);
    recent.truncate(5);
}
