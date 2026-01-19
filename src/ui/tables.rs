use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap};

use crate::app::App;
use crate::i18n::I18n;
use crate::models::MatchDetail;

use super::helpers::{format_duration, format_game_mode, format_relative_time, is_win, truncate_text};
use super::images::{push_match_row_images, push_team_images};
use super::ImageTarget;

pub fn draw_matches_table(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    base: Color,
    text: Color,
    accent: Color,
    images: &mut Vec<ImageTarget>,
    i18n: &I18n,
) {
    let header = Row::new(vec![
        i18n.table_hero(),
        i18n.table_result(),
        i18n.table_mode(),
        i18n.table_duration(),
        i18n.table_time(),
        i18n.table_k(),
        i18n.table_d(),
        i18n.table_a(),
    ])
    .style(Style::default().fg(accent).add_modifier(Modifier::BOLD));

    let total = app.matches.len();
    let selected = app.match_state.selected().unwrap_or(0).min(total.saturating_sub(1));
    let available = area.height.saturating_sub(3);
    let max_rows = (available / 2).max(1) as usize;
    let mut start = selected.saturating_sub(max_rows / 2);
    if start + max_rows > total {
        start = total.saturating_sub(max_rows);
    }
    let end = (start + max_rows).min(total);
    let row_count = end.saturating_sub(start);
    let rows: Vec<Row> = app.matches[start..end]
        .iter()
        .map(|m| {
            let win = is_win(m);
            let result = if win { i18n.result_win() } else { i18n.result_loss() };
            let duration = format_duration(m.duration);
            let rel_time = format_relative_time(m.start_time, i18n);
            let hero = truncate_text(&app.hero_name(m.hero_id, i18n), 18);
            let mode = format_game_mode(m.game_mode, i18n);
            let k = m
                .kills
                .map(|v| v.to_string())
                .unwrap_or_else(|| i18n.placeholder_dash().to_string());
            let d = m
                .deaths
                .map(|v| v.to_string())
                .unwrap_or_else(|| i18n.placeholder_dash().to_string());
            let a = m
                .assists
                .map(|v| v.to_string())
                .unwrap_or_else(|| i18n.placeholder_dash().to_string());
            let row_style = if win {
                Style::default().fg(Color::Rgb(166, 227, 161))
            } else {
                Style::default().fg(Color::Rgb(243, 139, 168))
            };
            Row::new(vec![
                Cell::from(format!("       {hero}")),
                Cell::from(result),
                Cell::from(mode),
                Cell::from(duration),
                Cell::from(rel_time),
                Cell::from(k),
                Cell::from(d),
                Cell::from(a),
            ])
            .style(row_style)
            .height(2)
        })
        .collect();

    let title = i18n.title_matches();
    let table = Table::new(
        rows,
        [
            Constraint::Length(28),
            Constraint::Length(4),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(accent)),
    )
    .column_spacing(1)
    .style(Style::default().bg(base).fg(text))
    .highlight_style(Style::default().bg(Color::Rgb(49, 50, 68)))
    .highlight_symbol("▌ ");

    let mut state = TableState::default();
    if total > 0 {
        state.select(Some(selected.saturating_sub(start)));
    }
    frame.render_stateful_widget(table, area, &mut state);
    push_match_row_images(app, area, images, start, row_count);
}

pub fn draw_match_detail_tables(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    base: Color,
    text: Color,
    accent: Color,
    images: &mut Vec<ImageTarget>,
    i18n: &I18n,
) {
    if app.detail_loading {
        let detail = Paragraph::new(i18n.loading_detail())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(i18n.title_match_detail())
                    .border_style(Style::default().fg(accent)),
            )
            .style(Style::default().bg(base).fg(text))
            .wrap(Wrap { trim: true });
        frame.render_widget(detail, area);
        return;
    }
    let detail = match &app.match_detail {
        Some(detail) => detail,
        None => {
            let detail = Paragraph::new(i18n.match_wait())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(i18n.title_match_detail())
                        .border_style(Style::default().fg(accent)),
                )
                .style(Style::default().bg(base).fg(text))
                .wrap(Wrap { trim: true });
            frame.render_widget(detail, area);
            return;
        }
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_team_table(frame, app, detail, layout[0], base, text, accent, images, i18n, true);
    draw_team_table(frame, app, detail, layout[1], base, text, accent, images, i18n, false);
}

pub fn draw_team_table(
    frame: &mut Frame,
    app: &App,
    detail: &MatchDetail,
    area: Rect,
    base: Color,
    text: Color,
    accent: Color,
    images: &mut Vec<ImageTarget>,
    i18n: &I18n,
    radiant: bool,
) {
    let debug = std::env::var("DOTA2_TUI_DEBUG").ok().as_deref() == Some("1");
    let players: Vec<&crate::models::MatchPlayer> = detail
        .players
        .iter()
        .filter(|p| p.player_slot.map(|s| s < 128).unwrap_or(false) == radiant)
        .collect();
    let available = area.height.saturating_sub(3);
    let max_rows = (available / 2).max(1) as usize;
    let visible: Vec<&crate::models::MatchPlayer> = players.into_iter().take(max_rows).collect();

    let header = Row::new(vec![
        i18n.table_player(),
        i18n.table_hero(),
        i18n.table_k(),
        i18n.table_d(),
        i18n.table_a(),
        i18n.table_gpm(),
        i18n.table_xpm(),
        i18n.table_net(),
        i18n.table_items(),
    ])
    .style(Style::default().fg(accent).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = visible
        .iter()
        .map(|p| {
            let hero = p
                .hero_id
                .map(|id| app.hero_name(id, i18n))
                .unwrap_or_else(|| i18n.unknown().to_string());
            let hero = truncate_text(&hero, 18);
            let mut name = p
                .personaname
                .as_deref()
                .map(|value| truncate_text(value, 16))
                .unwrap_or_else(|| i18n.anonymous().to_string());
            if debug {
                let marker = match p.account_id {
                    Some(id) => match app.player_avatars.get(&id) {
                        Some(url) => {
                            if app.image_cache.contains_key(url) {
                                "[C]"
                            } else {
                                "[U]"
                            }
                        }
                        None => "[?]",
                    },
                    None => "[N]",
                };
                name = format!("{name} {marker}");
            }
            let k = p
                .kills
                .map(|v| v.to_string())
                .unwrap_or_else(|| i18n.placeholder_dash().to_string());
            let d = p
                .deaths
                .map(|v| v.to_string())
                .unwrap_or_else(|| i18n.placeholder_dash().to_string());
            let a = p
                .assists
                .map(|v| v.to_string())
                .unwrap_or_else(|| i18n.placeholder_dash().to_string());
            let gpm = p
                .gold_per_min
                .map(|v| v.to_string())
                .unwrap_or_else(|| i18n.placeholder_dash().to_string());
            let xpm = p
                .xp_per_min
                .map(|v| v.to_string())
                .unwrap_or_else(|| i18n.placeholder_dash().to_string());
            let net = p
                .net_worth
                .map(|v| v.to_string())
                .unwrap_or_else(|| i18n.placeholder_dash().to_string());
            Row::new(vec![
                format!("       {name}"),
                format!("       {hero}"),
                k,
                d,
                a,
                gpm,
                xpm,
                net,
                String::new(),
            ])
            .height(2)
        })
        .collect();

    let title = if radiant { i18n.title_radiant() } else { i18n.title_dire() };
    let table = Table::new(
        rows,
        [
            Constraint::Length(22),
            Constraint::Length(20),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(5),
            Constraint::Min(12),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(accent)),
    )
    .column_spacing(1)
    .style(Style::default().bg(base).fg(text));

    frame.render_widget(table, area);
    push_team_images(app, &visible, area, images);
}
