use ratatui::prelude::*;

use crate::app::App;

use super::ImageTarget;

pub fn push_recent_images(app: &App, area: Rect, images: &mut Vec<ImageTarget>) {
    let start_x = area.x + 1;
    let mut y = area.y + 1;
    let width = 6;
    let height = 2;
    for entry in &app.recent_searches {
        if let Some(url) = &entry.avatar_url {
            if y + height <= area.y + area.height.saturating_sub(1) {
                images.push(ImageTarget {
                    area: Rect::new(start_x, y, width, height),
                    url: url.clone(),
                });
            }
        }
        y = y.saturating_add(height);
    }
}

pub fn push_loadout_images(app: &App, area: Rect, images: &mut Vec<ImageTarget>) {
    let player = match find_player_detail(app) {
        Some(player) => player,
        None => return,
    };
    let hero_id = match player.hero_id {
        Some(hero_id) => hero_id,
        None => return,
    };
    if let Some(url) = app.hero_images.get(&hero_id) {
        images.push(ImageTarget {
            area: Rect::new(area.x + 1, area.y + 1, 6, 4),
            url: url.clone(),
        });
    }
    let items = [
        player.item_0,
        player.item_1,
        player.item_2,
        player.item_3,
        player.item_4,
        player.item_5,
    ];
    let mut x = area.x + 8;
    let y = area.y + 1;
    for item in items.iter().flatten() {
        if let Some(url) = app.item_images.get(item) {
            images.push(ImageTarget {
                area: Rect::new(x, y, 4, 4),
                url: url.clone(),
            });
        }
        x = x.saturating_add(5);
    }
}

pub fn push_match_row_images(
    app: &App,
    area: Rect,
    images: &mut Vec<ImageTarget>,
    start: usize,
    count: usize,
) {
    let start_x = area.x + 1;
    let mut y = area.y + 2;
    let width = 6;
    let height = 2;
    for match_item in app.matches.iter().skip(start).take(count) {
        if let Some(url) = app.hero_images.get(&match_item.hero_id) {
            if y + height <= area.y + area.height.saturating_sub(1) {
                images.push(ImageTarget {
                    area: Rect::new(start_x, y, width, height),
                    url: url.clone(),
                });
            }
        }
        y = y.saturating_add(height);
    }
}

pub fn push_team_images(
    app: &App,
    players: &[&crate::models::MatchPlayer],
    area: Rect,
    images: &mut Vec<ImageTarget>,
) {
    let player_col = 22;
    let hero_col = 20;
    let hero_x = area.x + 1 + player_col + 1;
    let player_x = area.x + 1;
    let items_x = area.x + 1
        + player_col
        + 1
        + hero_col
        + 1
        + 3
        + 1
        + 3
        + 1
        + 3
        + 1
        + 4
        + 1
        + 4
        + 1
        + 5
        + 1;
    let mut y = area.y + 2;
    let max_y = area.y + area.height.saturating_sub(1);
    for player in players {
        if y + 2 > max_y {
            break;
        }
        if let Some(account_id) = player.account_id {
            if let Some(url) = app.player_avatars.get(&account_id) {
                if y + 2 <= max_y {
                    images.push(ImageTarget {
                        area: Rect::new(player_x, y, 6, 2),
                        url: url.clone(),
                    });
                }
            }
        }
        if let Some(hero_id) = player.hero_id {
            if let Some(url) = app.hero_images.get(&hero_id) {
                if y + 2 <= max_y {
                    images.push(ImageTarget {
                        area: Rect::new(hero_x, y, 6, 2),
                        url: url.clone(),
                    });
                }
            }
        }
        let items = [
            player.item_0,
            player.item_1,
            player.item_2,
            player.item_3,
            player.item_4,
            player.item_5,
        ];
        let mut x = items_x;
        for item in items.iter().flatten() {
            if let Some(url) = app.item_images.get(item) {
                if y + 2 <= max_y {
                    images.push(ImageTarget {
                        area: Rect::new(x, y, 4, 2),
                        url: url.clone(),
                    });
                }
            }
            x = x.saturating_add(5);
        }
        y = y.saturating_add(2);
    }
}

fn find_player_detail(app: &App) -> Option<&crate::models::MatchPlayer> {
    let detail = app.match_detail.as_ref()?;
    let account_id = app.account_id?;
    detail
        .players
        .iter()
        .find(|p| p.account_id == Some(account_id))
}
