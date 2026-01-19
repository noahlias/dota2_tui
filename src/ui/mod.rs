use ratatui::prelude::*;
use ratatui::widgets::Block;

use crate::app::App;
use crate::config::{Keybinds, Theme};
use crate::i18n::I18n;

mod banner;
mod helpers;
mod images;
mod panels;
mod tables;

#[derive(Clone)]
pub struct ImageTarget {
    pub area: Rect,
    pub url: String,
}

pub struct UiDrawResult {
    pub images: Vec<ImageTarget>,
}

pub fn draw_ui(
    frame: &mut Frame,
    app: &mut App,
    theme: Theme,
    keybinds: &Keybinds,
    i18n: &I18n,
) -> UiDrawResult {
    let Theme {
        base,
        text,
        accent,
        warn,
        success,
    } = theme;

    let size = frame.size();
    frame.render_widget(Block::default().style(Style::default().bg(base)), size);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(0)].as_ref())
        .split(size);

    banner::draw_banner(frame, app, layout[0], base, text, accent, i18n);

    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(32), Constraint::Percentage(68)])
        .split(layout[1]);

    let mut images = Vec::new();
    panels::draw_left_panel(
        frame,
        app,
        content[0],
        base,
        text,
        accent,
        warn,
        success,
        &mut images,
        i18n,
    );
    panels::draw_right_panel(
        frame,
        app,
        content[1],
        base,
        text,
        accent,
        warn,
        success,
        &mut images,
        i18n,
    );

    if app.show_help {
        panels::draw_help_popup(frame, keybinds, base, text, accent, i18n);
    }

    UiDrawResult { images }
}
