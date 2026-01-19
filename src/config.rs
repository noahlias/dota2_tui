use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: ThemeConfig,
    pub keybinds: Keybinds,
    pub api: ApiConfig,
    pub images: ImageConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Keybinds {
    pub search: String,
    pub quit: String,
    pub up: String,
    pub down: String,
    pub select: String,
    pub top: String,
    pub bottom: String,
    pub clear_input: String,
    pub tab_next: String,
    pub tab_prev: String,
    pub help: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiConfig {
    pub base_url: String,
    pub rate_limit_per_minute: u32,
    pub cache_ttl_secs: u64,
    pub cache_max_entries: usize,
    pub max_inflight: usize,
    pub log_requests: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ImageConfig {
    pub enabled: bool,
    pub protocol: String,
    pub cdn_base: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub language: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub base: Color,
    pub text: Color,
    pub accent: Color,
    pub warn: Color,
    pub success: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyCombo {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedKeybinds {
    pub search: KeyCombo,
    pub quit: KeyCombo,
    pub up: KeyCombo,
    pub down: KeyCombo,
    pub select: KeyCombo,
    pub top: KeyCombo,
    pub bottom: KeyCombo,
    pub clear_input: KeyCombo,
    pub tab_next: KeyCombo,
    pub tab_prev: KeyCombo,
    pub help: KeyCombo,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            keybinds: Keybinds::default(),
            api: ApiConfig::default(),
            images: ImageConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "catppuccin".to_string(),
        }
    }
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            search: "/".to_string(),
            quit: "q".to_string(),
            up: "k".to_string(),
            down: "j".to_string(),
            select: "Enter".to_string(),
            top: "g".to_string(),
            bottom: "G".to_string(),
            clear_input: "Ctrl+u".to_string(),
            tab_next: "Right".to_string(),
            tab_prev: "Left".to_string(),
            help: "?".to_string(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.opendota.com/api".to_string(),
            rate_limit_per_minute: 60,
            cache_ttl_secs: 300,
            cache_max_entries: 256,
            max_inflight: 6,
            log_requests: true,
        }
    }
}

impl ApiConfig {
    pub fn resolve_log_path(&self) -> Result<Option<std::path::PathBuf>> {
        if !self.log_requests {
            return Ok(None);
        }
        let mut path = config_path()?;
        path.set_file_name("tui.log");
        Ok(Some(path))
    }
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            protocol: "auto".to_string(),
            cdn_base: "https://cdn.cloudflare.steamstatic.com".to_string(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            language: "zh".to_string(),
        }
    }
}

impl Config {
    pub fn load_or_create() -> Result<(Self, PathBuf)> {
        let path = config_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed reading {}", path.display()))?;
            let parsed: Config = toml::from_str(&content)
                .with_context(|| format!("Invalid config at {}", path.display()))?;
            Ok((parsed, path))
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed creating {}", parent.display()))?;
            }
            let default = Config::default();
            let toml = toml::to_string_pretty(&default)?;
            fs::write(&path, toml).with_context(|| format!("Failed writing {}", path.display()))?;
            Ok((default, path))
        }
    }

    pub fn resolve_theme(&self) -> Theme {
        match self.theme.name.as_str() {
            "catppuccin" => Theme {
                base: Color::Rgb(30, 30, 46),
                text: Color::Rgb(205, 214, 244),
                accent: Color::Rgb(137, 180, 250),
                warn: Color::Rgb(243, 139, 168),
                success: Color::Rgb(166, 227, 161),
            },
            "gruvbox" => Theme {
                base: Color::Rgb(40, 40, 40),
                text: Color::Rgb(235, 219, 178),
                accent: Color::Rgb(250, 189, 47),
                warn: Color::Rgb(251, 73, 52),
                success: Color::Rgb(184, 187, 38),
            },
            _ => Theme {
                base: Color::Black,
                text: Color::White,
                accent: Color::Cyan,
                warn: Color::Red,
                success: Color::Green,
            },
        }
    }

    pub fn resolve_keybinds(&self) -> Result<ResolvedKeybinds> {
        Ok(ResolvedKeybinds {
            search: parse_keycombo(&self.keybinds.search)?,
            quit: parse_keycombo(&self.keybinds.quit)?,
            up: parse_keycombo(&self.keybinds.up)?,
            down: parse_keycombo(&self.keybinds.down)?,
            select: parse_keycombo(&self.keybinds.select)?,
            top: parse_keycombo(&self.keybinds.top)?,
            bottom: parse_keycombo(&self.keybinds.bottom)?,
            clear_input: parse_keycombo(&self.keybinds.clear_input)?,
            tab_next: parse_keycombo(&self.keybinds.tab_next)?,
            tab_prev: parse_keycombo(&self.keybinds.tab_prev)?,
            help: parse_keycombo(&self.keybinds.help)?,
        })
    }
}

pub fn config_path() -> Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow!("No config directory"))?;
    Ok(base.join("dota2_tui").join("config.toml"))
}

pub fn recent_log_path() -> Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow!("No config directory"))?;
    Ok(base.join("dota2_tui").join("recent.jsonl"))
}

pub fn cache_dir() -> Result<PathBuf> {
    let base = dirs::cache_dir().ok_or_else(|| anyhow!("No cache directory"))?;
    Ok(base.join("dota2_tui"))
}

fn parse_keycombo(input: &str) -> Result<KeyCombo> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Empty keybind"));
    }

    let parts: Vec<&str> = trimmed.split('+').collect();
    let mut modifiers = KeyModifiers::empty();
    let key_part = if parts.len() > 1 {
        for part in &parts[..parts.len() - 1] {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
                "alt" => modifiers |= KeyModifiers::ALT,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                other => return Err(anyhow!("Unknown modifier: {other}")),
            }
        }
        parts[parts.len() - 1]
    } else {
        parts[0]
    };

    let code = parse_keycode(key_part)?;

    if parts.len() == 1 {
        if let KeyCode::Char(c) = code {
            if c.is_ascii_uppercase() {
                modifiers |= KeyModifiers::SHIFT;
            }
        }
    }

    Ok(KeyCombo { code, modifiers })
}

fn parse_keycode(input: &str) -> Result<KeyCode> {
    let trimmed = input.trim();
    if trimmed.len() == 1 {
        return Ok(KeyCode::Char(trimmed.chars().next().unwrap()));
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "enter" => Ok(KeyCode::Enter),
        "tab" => Ok(KeyCode::Tab),
        "esc" | "escape" => Ok(KeyCode::Esc),
        "backspace" => Ok(KeyCode::Backspace),
        "up" => Ok(KeyCode::Up),
        "down" => Ok(KeyCode::Down),
        "left" => Ok(KeyCode::Left),
        "right" => Ok(KeyCode::Right),
        "home" => Ok(KeyCode::Home),
        "end" => Ok(KeyCode::End),
        "pageup" => Ok(KeyCode::PageUp),
        "pagedown" => Ok(KeyCode::PageDown),
        "insert" => Ok(KeyCode::Insert),
        "delete" => Ok(KeyCode::Delete),
        "space" => Ok(KeyCode::Char(' ')),
        other => Err(anyhow!("Unknown key: {other}")),
    }
}

pub fn matches(combo: KeyCombo, code: KeyCode, modifiers: KeyModifiers) -> bool {
    if combo.code != code {
        return false;
    }
    if combo.modifiers.is_empty() {
        modifiers.is_empty()
    } else {
        combo.modifiers == modifiers
    }
}
