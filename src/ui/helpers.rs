use ratatui::prelude::*;

use crate::app::App;
use crate::i18n::I18n;
use crate::models::PlayerMatch;

pub fn build_profile_text(app: &App, i18n: &I18n) -> String {
    if app.loading {
        return i18n.loading_player().to_string();
    }
    let profile = match &app.profile {
        Some(profile) => profile,
        None => return i18n.no_player_loaded().to_string(),
    };
    let persona = profile
        .profile
        .as_ref()
        .and_then(|p| p.personaname.as_deref())
        .unwrap_or(i18n.unknown());
    let steamid = profile
        .profile
        .as_ref()
        .and_then(|p| p.steamid.as_deref())
        .unwrap_or(i18n.placeholder_dash());
    let mmr = profile
        .mmr_estimate
        .as_ref()
        .and_then(|m| m.estimate)
        .map(|v| v.to_string())
        .unwrap_or_else(|| i18n.placeholder_dash().to_string());

    format!(
        "{}: {persona}\n{}: {steamid}\n{}: {mmr}",
        i18n.label_name(),
        i18n.label_steamid(),
        i18n.label_mmr()
    )
}

pub fn build_quick_stats(app: &App, i18n: &I18n) -> String {
    let total = app.matches.len();
    if total == 0 {
        return i18n.status_no_matches().to_string();
    }
    let wins = app.matches.iter().filter(|m| is_win(m)).count();
    i18n.quick_stats_format(total, wins)
}

pub fn compute_winrate(app: &App) -> f64 {
    let total = app.matches.len();
    if total == 0 {
        return 0.0;
    }
    let wins = app.matches.iter().filter(|m| is_win(m)).count();
    wins as f64 / total as f64
}

pub fn build_sparkline(app: &App) -> Vec<u64> {
    let mut data = Vec::new();
    for m in app.matches.iter().take(20).rev() {
        data.push(if is_win(m) { 10 } else { 2 });
    }
    if data.is_empty() {
        data.push(0);
    }
    data
}

pub fn build_stats_text(app: &App, i18n: &I18n) -> String {
    let total = app.matches.len();
    let wins = app.matches.iter().filter(|m| is_win(m)).count();
    let winrate = compute_winrate(app) * 100.0;
    i18n.stats_summary_format(total, wins, winrate)
}

pub fn is_win(match_item: &PlayerMatch) -> bool {
    let is_radiant = match_item.player_slot < 128;
    is_radiant == match_item.radiant_win
}

pub fn format_duration(seconds: u32) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", mins, secs)
}

pub fn format_relative_time(start_time: Option<i64>, i18n: &I18n) -> String {
    let Some(start) = start_time else {
        return i18n.placeholder_dash().to_string();
    };
    let now = chrono::Utc::now().timestamp();
    let diff = (now - start).max(0);
    if diff < 60 {
        return i18n.time_now().to_string();
    }
    if diff < 3600 {
        return i18n.time_minutes(diff / 60);
    }
    if diff < 86400 {
        return i18n.time_hours(diff / 3600);
    }
    i18n.time_days(diff / 86400)
}

pub fn format_game_mode(game_mode: Option<i32>, i18n: &I18n) -> String {
    i18n.format_game_mode(game_mode)
}

pub fn truncate_text(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_string();
    }
    let mut out = value.chars().take(max.saturating_sub(1)).collect::<String>();
    out.push('…');
    out
}


pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
