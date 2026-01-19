use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Gauge, Paragraph, Row, Sparkline, Table, TableState, Tabs, Wrap};

use crate::app::App;
use crate::config::Keybinds;
use crate::i18n::I18n;

use super::helpers::{
    build_profile_text, build_quick_stats, build_sparkline, build_stats_text, compute_winrate,
    centered_rect,
};
use super::images::{push_loadout_images, push_recent_images};
use super::tables::{draw_match_detail_tables, draw_matches_table};
use super::ImageTarget;

pub fn draw_left_panel(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    base: Color,
    text: Color,
    accent: Color,
    warn: Color,
    success: Color,
    images: &mut Vec<ImageTarget>,
    i18n: &I18n,
) {
    let panel = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Min(8),
            Constraint::Length(5),
        ])
        .split(area);

    let input_title = i18n.input_search(matches!(app.input_mode, crate::app::InputMode::Editing));
    let input = Paragraph::new(app.input.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(input_title)
                .border_style(Style::default().fg(accent)),
        )
        .style(Style::default().bg(base).fg(text));
    frame.render_widget(input, panel[0]);

    if app.recent_searches.is_empty() {
        let recent = Paragraph::new(i18n.no_recent())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(i18n.title_recent())
                    .border_style(Style::default().fg(accent)),
            )
            .style(Style::default().bg(base).fg(text))
            .wrap(Wrap { trim: true });
        frame.render_widget(recent, panel[1]);
    } else {
        let rows: Vec<Row> = app
            .recent_searches
            .iter()
            .map(|entry| {
                Row::new(vec![
                    String::new(),
                    format!("{} ({})", entry.personaname, entry.account_id),
                ])
                .height(2)
            })
            .collect();
        let recent = Table::new(
            rows,
            [Constraint::Length(6), Constraint::Min(10)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(i18n.title_recent())
                .border_style(Style::default().fg(accent)),
        )
        .column_spacing(1)
        .style(Style::default().bg(base).fg(text))
        .highlight_style(Style::default().bg(Color::Rgb(49, 50, 68)))
        .highlight_symbol("▌ ");
        let mut state = TableState::default();
        if app.recent_active {
            state.select(app.recent_index);
        }
        frame.render_stateful_widget(recent, panel[1], &mut state);
        push_recent_images(app, panel[1], images);
    }

    draw_search_help(frame, panel[2], base, text, accent, i18n);

    let status_style = if app.loading || app.detail_loading || app.avatar_loading {
        Style::default().fg(warn)
    } else {
        Style::default().fg(success)
    };
    let status_text = if app.net_total > 0 {
        if let Some(ms) = app.net_last_ms {
            format!("{} | {}/{} | {}ms", app.status, app.net_done, app.net_total, ms)
        } else {
            format!("{} | {}/{}", app.status, app.net_done, app.net_total)
        }
    } else {
        app.status.clone()
    };
    let status = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(i18n.title_status())
                .border_style(Style::default().fg(accent)),
        )
        .style(Style::default().bg(base).fg(text).patch(status_style))
        .wrap(Wrap { trim: true });
    frame.render_widget(status, panel[3]);

    if app.net_total > 0 {
        let ratio = (app.net_done as f64 / app.net_total.max(1) as f64).min(1.0);
        let bar = Gauge::default()
            .gauge_style(Style::default().fg(accent).bg(base))
            .ratio(ratio);
        let bar_area = Rect::new(panel[3].x + 1, panel[3].y + panel[3].height.saturating_sub(2), panel[3].width.saturating_sub(2), 1);
        frame.render_widget(bar, bar_area);
    }
}

fn draw_search_help(
    frame: &mut Frame,
    area: Rect,
    base: Color,
    text: Color,
    accent: Color,
    i18n: &I18n,
) {
    let title = i18n.title_results();
    let content = i18n.search_hint();
    let panel = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(accent)),
        )
        .style(Style::default().bg(base).fg(text))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

pub fn draw_right_panel(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    base: Color,
    text: Color,
    accent: Color,
    warn: Color,
    success: Color,
    images: &mut Vec<ImageTarget>,
    i18n: &I18n,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    draw_tabs(frame, app, layout[0], base, text, accent, i18n);

    match app.tab_index {
        0 => draw_overview_tab(frame, app, layout[1], base, text, accent, warn, success, images, i18n),
        1 => draw_matches_tab(frame, app, layout[1], base, text, accent, images, i18n),
        _ => draw_stats_tab(frame, app, layout[1], base, text, accent, success, i18n),
    }
}

fn draw_tabs(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    base: Color,
    text: Color,
    accent: Color,
    i18n: &I18n,
) {
    let titles = [i18n.tab_overview(), i18n.tab_matches(), i18n.tab_stats()]
        .iter()
        .map(|t| Line::from(*t))
        .collect::<Vec<_>>();
    let tabs = Tabs::new(titles)
        .select(app.tab_index)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(i18n.title_views())
                .border_style(Style::default().fg(accent)),
        )
        .style(Style::default().bg(base).fg(text))
        .highlight_style(Style::default().fg(accent).add_modifier(Modifier::BOLD));
    frame.render_widget(tabs, area);
}

fn draw_overview_tab(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    base: Color,
    text: Color,
    accent: Color,
    warn: Color,
    success: Color,
    images: &mut Vec<ImageTarget>,
    i18n: &I18n,
) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let profile_panel = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Length(5), Constraint::Min(0)])
        .split(layout[0]);

    let profile_text = build_profile_text(app, i18n);
    let status_style = if app.loading || app.detail_loading || app.avatar_loading {
        Style::default().fg(warn)
    } else {
        Style::default().fg(success)
    };
    let profile = Paragraph::new(profile_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(i18n.title_profile())
                .border_style(Style::default().fg(accent)),
        )
        .style(Style::default().bg(base).fg(text).patch(status_style))
        .wrap(Wrap { trim: true });
    frame.render_widget(profile, profile_panel[0]);

    let quick = Paragraph::new(build_quick_stats(app, i18n))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(i18n.title_quick_stats())
                .border_style(Style::default().fg(accent)),
        )
        .style(Style::default().bg(base).fg(text))
        .wrap(Wrap { trim: true });
    frame.render_widget(quick, profile_panel[1]);

    draw_match_detail_tables(frame, app, profile_panel[2], base, text, accent, images, i18n);

    let right_panel = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Length(6), Constraint::Min(0)])
        .split(layout[1]);

    let avatar_block = Block::default()
        .borders(Borders::ALL)
        .title(i18n.title_avatar())
        .border_style(Style::default().fg(accent))
        .style(Style::default().bg(base).fg(text));
    frame.render_widget(avatar_block, right_panel[0]);
    if let Some(url) = &app.avatar_url {
        let area = right_panel[0];
        let target = Rect::new(area.x + 1, area.y + 1, 8, 6);
        images.push(super::ImageTarget {
            area: target,
            url: url.clone(),
        });
    }

    let loadout_block = Block::default()
        .borders(Borders::ALL)
        .title(i18n.title_loadout())
        .border_style(Style::default().fg(accent))
        .style(Style::default().bg(base).fg(text));
    frame.render_widget(loadout_block, right_panel[1]);
    push_loadout_images(app, right_panel[1], images);

    let hint = Paragraph::new(i18n.hint_tabs())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(i18n.title_hints())
                .border_style(Style::default().fg(accent)),
        )
        .style(Style::default().bg(base).fg(text));
    frame.render_widget(hint, right_panel[2]);
}

fn draw_matches_tab(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    base: Color,
    text: Color,
    accent: Color,
    images: &mut Vec<ImageTarget>,
    i18n: &I18n,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    draw_matches_table(frame, app, layout[0], base, text, accent, images, i18n);
    draw_match_detail_tables(frame, app, layout[1], base, text, accent, images, i18n);
}

fn draw_stats_tab(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    base: Color,
    text: Color,
    accent: Color,
    success: Color,
    i18n: &I18n,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Length(6), Constraint::Min(0)])
        .split(area);

    let winrate = compute_winrate(app);
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(i18n.title_winrate())
                .border_style(Style::default().fg(accent)),
        )
        .gauge_style(Style::default().fg(success).bg(base))
        .ratio(winrate)
        .label(format!("{:.0}%", winrate * 100.0));
    frame.render_widget(gauge, layout[0]);

    let spark_data = build_sparkline(app);
    let spark = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(i18n.title_recent_results())
                .border_style(Style::default().fg(accent)),
        )
        .style(Style::default().fg(accent).bg(base))
        .data(&spark_data);
    frame.render_widget(spark, layout[1]);

    let info = Paragraph::new(build_stats_text(app, i18n))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(i18n.title_summary())
                .border_style(Style::default().fg(accent)),
        )
        .style(Style::default().bg(base).fg(text))
        .wrap(Wrap { trim: true });
    frame.render_widget(info, layout[2]);
}

pub fn draw_help_popup(
    frame: &mut Frame,
    keybinds: &Keybinds,
    base: Color,
    text: Color,
    accent: Color,
    i18n: &I18n,
) {
    let overlay = Block::default().style(Style::default().bg(Color::Rgb(12, 12, 18)));
    frame.render_widget(overlay, frame.size());

    let area = centered_rect(70, 70, frame.size());
    let labels = i18n.help_labels();
    let help_text = format!(
        "{}\n  {}  {}\n  {}  {}\n\n{}\n  {}  {}/{}\n  {}  {}\n  {}  {}/{}\n\n{}\n  {}  {}/{}\n\n{}\n  {}  {}\n  {}  {}",
        i18n.help_group_search(),
        keybinds.search,
        labels[0],
        keybinds.clear_input,
        labels[5],
        i18n.help_group_navigation(),
        keybinds.up,
        labels[2],
        keybinds.down,
        keybinds.select,
        labels[3],
        keybinds.top,
        labels[4],
        keybinds.bottom,
        i18n.help_group_views(),
        keybinds.tab_prev,
        labels[6],
        keybinds.tab_next,
        i18n.help_group_misc(),
        keybinds.help,
        labels[7],
        keybinds.quit,
        labels[1],
    );
    frame.render_widget(Clear, area);
    let popup = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(i18n.keybind_title())
                .border_style(Style::default().fg(accent)),
        )
        .style(Style::default().bg(base).fg(text))
        .wrap(Wrap { trim: true });
    frame.render_widget(popup, area);
}
