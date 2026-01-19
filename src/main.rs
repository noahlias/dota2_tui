use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::EventStream;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::time::interval;

mod api;
mod app;
mod config;
mod image;
mod i18n;
mod input;
mod models;
mod ui;

use crate::api::ApiClient;
use crate::app::{
    handle_message, load_avatar_map, load_recent_searches, spawn_hero_images, spawn_hero_load,
    spawn_image_fetch, spawn_item_images, spawn_player_avatars, App, Message,
};
use crate::config::Config;
use crate::image::{ensure_png, read_disk_cache, write_disk_cache, ImageSupport};
use crate::i18n::I18n;
use crate::input::handle_event;
use crate::ui::draw_ui;

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let (config, _config_path) = Config::load_or_create()?;
    let theme = config.resolve_theme();
    let keybinds = config.resolve_keybinds()?;
    let mut images = ImageSupport::from_config(&config.images);
    let i18n = I18n::new(crate::i18n::I18n::language_from_config(&config.ui.language));
    let cdn_base = config.images.cdn_base.clone();

    let api = ApiClient::new(config.api.clone());
    let (tx, mut rx) = mpsc::channel::<Message>(16);

    let mut app = App::new();
    app.set_status(i18n.status_ready());
    app.recent_searches = load_recent_searches(5);
    app.player_avatars = load_avatar_map();
    app.net_total = app.net_total.saturating_add(1);
    app.net_inflight = app.net_inflight.saturating_add(1);
    spawn_hero_load(tx.clone(), api.clone());
    let cdn_base = cdn_base;

    let mut events = EventStream::new();
    let mut tick = interval(Duration::from_millis(200));

    loop {
        let mut image_targets = Vec::new();
        terminal.draw(|frame| {
            image_targets = draw_ui(frame, &mut app, theme, &config.keybinds, &i18n).images;
        })?;
        if app.image_reset {
            images.reset(&mut terminal)?;
            app.image_reset = false;
        }

        if !app.show_help {
            for target in image_targets {
                if let Some(bytes) = app.image_cache.get(&target.url) {
                    images.render_avatar(&mut terminal, Some(target.area), Some(bytes))?;
                } else if !app.image_inflight.contains(&target.url) {
                    if let Ok(Some(bytes)) = read_disk_cache(&target.url) {
                        let bytes = ensure_png(&bytes).unwrap_or(bytes);
                        app.cache_image(target.url.clone(), bytes.clone());
                        images.render_avatar(&mut terminal, Some(target.area), Some(&bytes))?;
                    } else {
                        app.image_inflight.insert(target.url.clone());
                        spawn_image_fetch(tx.clone(), api.clone(), target.url);
                    }
                }
            }
        }
        app.avatar_loading = !app.image_inflight.is_empty();

        if app.profile.is_some() {
            if app.hero_images.is_empty() && !app.requested_hero_images {
                app.requested_hero_images = true;
                app.net_total = app.net_total.saturating_add(1);
                app.net_inflight = app.net_inflight.saturating_add(1);
                spawn_hero_images(tx.clone(), api.clone(), cdn_base.clone());
            }
            if app.item_images.is_empty() && !app.requested_item_images {
                app.requested_item_images = true;
                app.net_total = app.net_total.saturating_add(1);
                app.net_inflight = app.net_inflight.saturating_add(1);
                spawn_item_images(tx.clone(), api.clone(), cdn_base.clone());
            }
        }

        tokio::select! {
            _ = tick.tick() => {
                app.advance_tick();
            },
            maybe_event = events.next() => {
                if let Some(Ok(event)) = maybe_event {
                    handle_event(event, &mut app, &tx, &api, &keybinds, &i18n);
                }
            }
            maybe_msg = rx.recv() => {
                if let Some(msg) = maybe_msg {
                    if let Message::ImageLoaded { url, result: Ok(bytes) } = &msg {
                        if let Ok(png) = ensure_png(bytes) {
                            let _ = write_disk_cache(url, &png);
                            app.cache_image(url.clone(), png);
                        }
                    }
                    if let Some(url) = handle_message(msg, &mut app, &i18n) {
                        if !app.image_inflight.contains(&url) {
                            app.image_inflight.insert(url.clone());
                            spawn_image_fetch(tx.clone(), api.clone(), url);
                        }
                    }
                    if !app.pending_player_avatar_ids.is_empty() {
                        let ids = std::mem::take(&mut app.pending_player_avatar_ids);
                        spawn_player_avatars(tx.clone(), api.clone(), ids);
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
