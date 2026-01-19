use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct HeroStat {
    pub id: i32,
    pub localized_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HeroConstant {
    pub id: i32,
    pub img: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct PlayerResponse {
    pub profile: Option<PlayerProfile>,
    pub mmr_estimate: Option<MmrEstimate>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct PlayerProfile {
    pub personaname: Option<String>,
    pub steamid: Option<String>,
    pub avatar: Option<String>,
    pub avatarmedium: Option<String>,
    pub avatarfull: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct MmrEstimate {
    pub estimate: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlayerMatch {
    pub match_id: u64,
    pub player_slot: u16,
    pub radiant_win: bool,
    pub duration: u32,
    pub start_time: Option<i64>,
    pub hero_id: i32,
    pub game_mode: Option<i32>,
    pub kills: Option<i32>,
    pub deaths: Option<i32>,
    pub assists: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MatchDetail {
    pub players: Vec<MatchPlayer>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MatchPlayer {
    pub account_id: Option<u32>,
    pub personaname: Option<String>,
    pub hero_id: Option<i32>,
    pub player_slot: Option<u16>,
    pub item_0: Option<i32>,
    pub item_1: Option<i32>,
    pub item_2: Option<i32>,
    pub item_3: Option<i32>,
    pub item_4: Option<i32>,
    pub item_5: Option<i32>,
    pub kills: Option<i32>,
    pub deaths: Option<i32>,
    pub assists: Option<i32>,
    pub gold_per_min: Option<i32>,
    pub xp_per_min: Option<i32>,
    pub net_worth: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ItemConstant {
    pub id: i32,
    pub img: Option<String>,
}
