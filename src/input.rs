use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::api::ApiClient;
use crate::app::{spawn_match_detail, spawn_search, App, InputMode, Message};
use crate::config::{matches, ResolvedKeybinds};
use crate::i18n::I18n;

const STEAMID64_BASE: u64 = 76561197960265728;
const NAV_DEBOUNCE_MS: u64 = 80;

pub fn handle_event(
    event: Event,
    app: &mut App,
    tx: &mpsc::Sender<Message>,
    api: &ApiClient,
    keybinds: &ResolvedKeybinds,
    i18n: &I18n,
) {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => match app.input_mode {
            InputMode::Editing => handle_editing_key(key, app, tx, api, keybinds, i18n),
            InputMode::Normal => handle_normal_key(key, app, tx, api, keybinds, i18n),
        },
        _ => {}
    }
}

fn handle_editing_key(
    key: KeyEvent,
    app: &mut App,
    tx: &mpsc::Sender<Message>,
    api: &ApiClient,
    keybinds: &ResolvedKeybinds,
    i18n: &I18n,
) {
    if matches(keybinds.help, key.code, key.modifiers) {
        app.show_help = !app.show_help;
        app.image_reset = true;
        return;
    }
    if key.code == KeyCode::Tab {
        if let Some(entry) = autocomplete_recent(app) {
            app.input = entry.account_id.to_string();
        }
        return;
    }
    if key.code == KeyCode::Esc {
        app.input_mode = InputMode::Normal;
        app.set_status(i18n.status_search_cancelled());
        return;
    }

    if matches(keybinds.select, key.code, key.modifiers) {
        if start_search_from_input(app, tx, api, i18n).is_ok() {
            app.input_mode = InputMode::Normal;
        }
        return;
    }

    if matches(keybinds.clear_input, key.code, key.modifiers) {
        app.input.clear();
        return;
    }

    match key.code {
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Char(c) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                app.input.push(c);
            }
        }
        _ => {}
    }
}

fn handle_normal_key(
    key: KeyEvent,
    app: &mut App,
    tx: &mpsc::Sender<Message>,
    api: &ApiClient,
    keybinds: &ResolvedKeybinds,
    i18n: &I18n,
) {
    if app.show_help {
        if key.code == KeyCode::Esc || matches(keybinds.help, key.code, key.modifiers) {
            app.show_help = false;
            app.image_reset = true;
        }
        return;
    }
    if matches(keybinds.quit, key.code, key.modifiers) {
        app.should_quit = true;
        return;
    }
    if matches(keybinds.help, key.code, key.modifiers) {
        app.show_help = true;
        app.image_reset = true;
        return;
    }
    if matches(keybinds.search, key.code, key.modifiers) {
        app.input_mode = InputMode::Editing;
        app.set_status(i18n.status_need_id());
        return;
    }
    if matches(keybinds.down, key.code, key.modifiers) {
        if can_navigate(app) {
            select_next_match(app);
        }
        app.recent_active = false;
        return;
    }
    if matches(keybinds.up, key.code, key.modifiers) {
        if can_navigate(app) {
            select_prev_match(app);
        }
        app.recent_active = false;
        return;
    }
    if matches(keybinds.top, key.code, key.modifiers) {
        if can_navigate(app) && !app.matches.is_empty() {
            app.match_state.select(Some(0));
        }
        app.recent_active = false;
        return;
    }
    if matches(keybinds.bottom, key.code, key.modifiers) {
        if can_navigate(app) && !app.matches.is_empty() {
            app.match_state.select(Some(app.matches.len() - 1));
        }
        app.recent_active = false;
        return;
    }
    if key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::CONTROL) {
        select_next_recent(app);
        return;
    }
    if key.code == KeyCode::BackTab && key.modifiers.contains(KeyModifiers::CONTROL) {
        select_prev_recent(app);
        return;
    }
    if matches(keybinds.select, key.code, key.modifiers) {
        if app.recent_active {
            if let Some(idx) = app.recent_index {
                if let Some(account_id) = app
                    .recent_searches
                    .get(idx)
                    .map(|entry| entry.account_id)
                {
                    start_search_with_id(app, tx, api, i18n, account_id);
                    app.input = account_id.to_string();
                    app.recent_active = false;
                    return;
                }
            }
        }
        if let Some(match_id) = app.selected_match().map(|m| m.match_id) {
            app.detail_loading = true;
            app.net_total = 1;
            app.net_done = 0;
            app.net_inflight = 1;
            app.net_last_ms = None;
            app.set_status(i18n.status_loading_match(match_id));
            spawn_match_detail(tx.clone(), api.clone(), match_id);
        }
        return;
    }
    if matches(keybinds.tab_next, key.code, key.modifiers) {
        app.tab_index = (app.tab_index + 1) % 3;
        app.image_reset = true;
        return;
    }
    if matches(keybinds.tab_prev, key.code, key.modifiers) {
        if app.tab_index == 0 {
            app.tab_index = 2;
        } else {
            app.tab_index -= 1;
        }
        app.image_reset = true;
        return;
    }

    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.set_status(i18n.status_search_cancelled());
        }
        _ => {}
    }
}

fn select_next_match(app: &mut App) {
    let total = app.matches.len();
    if total == 0 {
        return;
    }
    let next = match app.match_state.selected() {
        Some(idx) if idx + 1 < total => idx + 1,
        _ => 0,
    };
    app.match_state.select(Some(next));
}

fn select_prev_match(app: &mut App) {
    let total = app.matches.len();
    if total == 0 {
        return;
    }
    let prev = match app.match_state.selected() {
        Some(idx) if idx > 0 => idx - 1,
        _ => total - 1,
    };
    app.match_state.select(Some(prev));
}

fn parse_account_id(input: &str) -> Result<u32, String> {
    let trimmed = input.trim();
    let value: u64 = trimmed
        .parse()
        .map_err(|_| "SteamID must be numeric".to_string())?;
    if value >= STEAMID64_BASE {
        Ok((value - STEAMID64_BASE) as u32)
    } else if value <= u64::from(u32::MAX) {
        Ok(value as u32)
    } else {
        Err("SteamID too large".to_string())
    }
}

fn can_navigate(app: &mut App) -> bool {
    let now = Instant::now();
    if now.duration_since(app.last_nav) < Duration::from_millis(NAV_DEBOUNCE_MS) {
        return false;
    }
    app.last_nav = now;
    true
}

fn start_search_from_input(
    app: &mut App,
    tx: &mpsc::Sender<Message>,
    api: &ApiClient,
    i18n: &I18n,
) -> Result<(), ()> {
    if app.input.trim().is_empty() {
        app.set_status(i18n.status_need_id());
        return Err(());
    }
    match parse_account_id(&app.input) {
        Ok(account_id) => {
            start_search_with_id(app, tx, api, i18n, account_id);
            Ok(())
        }
        Err(_) => {
            app.set_status(i18n.status_invalid_id());
            Err(())
        }
    }
}

fn start_search_with_id(
    app: &mut App,
    tx: &mpsc::Sender<Message>,
    api: &ApiClient,
    i18n: &I18n,
    account_id: u32,
) {
    app.loading = true;
    app.detail_loading = false;
    app.account_id = Some(account_id);
    app.profile = None;
    app.clear_matches();
    app.avatar_url = None;
    app.avatar_loading = false;
    app.player_avatar_requests.clear();
    app.pending_player_avatar_ids.clear();
    app.image_reset = true;
    app.image_inflight.clear();
    app.net_total = 2;
    app.net_done = 0;
    app.net_inflight = 2;
    app.net_last_ms = None;
    app.set_status(i18n.status_loading_player(account_id));
    spawn_search(tx.clone(), api.clone(), account_id);
}

fn select_next_recent(app: &mut App) {
    if app.recent_searches.is_empty() {
        return;
    }
    app.recent_active = true;
    let idx = match app.recent_index {
        Some(current) if current + 1 < app.recent_searches.len() => current + 1,
        _ => 0,
    };
    app.recent_index = Some(idx);
}

fn select_prev_recent(app: &mut App) {
    if app.recent_searches.is_empty() {
        return;
    }
    app.recent_active = true;
    let idx = match app.recent_index {
        Some(current) if current > 0 => current - 1,
        _ => app.recent_searches.len() - 1,
    };
    app.recent_index = Some(idx);
}

fn autocomplete_recent(app: &App) -> Option<&crate::app::SearchEntry> {
    if app.recent_searches.is_empty() {
        return None;
    }
    let input = app.input.trim().to_ascii_lowercase();
    if input.is_empty() {
        return app.recent_searches.first();
    }
    app.recent_searches.iter().find(|entry| {
        entry
            .personaname
            .to_ascii_lowercase()
            .starts_with(&input)
            || entry.account_id.to_string().starts_with(&input)
    })
}
