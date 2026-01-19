use std::env;
use std::io;

use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::config::{cache_dir, ImageConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageProtocol {
    Kitty,
    Iterm2,
    None,
}

#[derive(Debug, Clone)]
pub struct ImageSupport {
    protocol: ImageProtocol,
    enabled: bool,
}

impl ImageSupport {
    pub fn new(protocol: ImageProtocol, enabled: bool) -> Self {
        Self {
            protocol,
            enabled,
        }
    }

    pub fn detect(enabled: bool) -> Self {
        if !enabled {
            return Self::new(ImageProtocol::None, false);
        }
        let term = env::var("TERM").unwrap_or_default().to_ascii_lowercase();
        let term_program = env::var("TERM_PROGRAM").unwrap_or_default();
        if env::var("KITTY_WINDOW_ID").is_ok()
            || term.contains("kitty")
            || term_program.to_ascii_lowercase().contains("ghostty")
        {
            return Self::new(ImageProtocol::Kitty, true);
        }
        if env::var("ITERM_SESSION_ID").is_ok()
            || env::var("WEZTERM_EXECUTABLE").is_ok()
            || term_program == "WezTerm"
            || term_program == "iTerm.app"
        {
            return Self::new(ImageProtocol::Iterm2, true);
        }
        Self::new(ImageProtocol::None, true)
    }

    pub fn from_config(config: &ImageConfig) -> Self {
        let enabled = config.enabled;
        match config.protocol.to_ascii_lowercase().as_str() {
            "kitty" => Self::new(ImageProtocol::Kitty, enabled),
            "iterm2" | "wezterm" => Self::new(ImageProtocol::Iterm2, enabled),
            "none" => Self::new(ImageProtocol::None, false),
            _ => Self::detect(enabled),
        }
    }

    pub fn render_avatar(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        area: Option<ratatui::layout::Rect>,
        avatar: Option<&[u8]>,
    ) -> io::Result<()> {
        if !self.enabled || self.protocol == ImageProtocol::None {
            return Ok(());
        }
        let area = match area {
            Some(area) => area,
            None => return Ok(()),
        };
        let bytes = match avatar {
            Some(bytes) => bytes,
            None => return Ok(()),
        };

        let stdout = terminal.backend_mut();
        let (inner_x, inner_y, width, height) = if area.width <= 2 || area.height <= 2 {
            let w = area.width.max(1);
            let h = area.height.max(1);
            (area.x, area.y, w, h)
        } else {
            let inner_x = area.x + 1;
            let inner_y = area.y + 1;
            let width = area.width.saturating_sub(2).max(1);
            let height = area.height.saturating_sub(2).max(1);
            (inner_x, inner_y, width, height)
        };
        queue!(stdout, MoveTo(inner_x, inner_y))?;

        match self.protocol {
            ImageProtocol::Iterm2 => write_iterm2_image(stdout, bytes, width, height)?,
            ImageProtocol::Kitty => write_kitty_image(stdout, bytes, width, height)?,
            ImageProtocol::None => {}
        }

        Ok(())
    }

    pub fn reset(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let stdout = terminal.backend_mut();
        match self.protocol {
            ImageProtocol::Kitty => {
                let seq = "\u{1b}_Ga=d\u{1b}\\";
                queue!(stdout, crossterm::style::Print(seq))?;
            }
            ImageProtocol::Iterm2 => {}
            ImageProtocol::None => {}
        }
        Ok(())
    }
}

pub fn read_disk_cache(url: &str) -> io::Result<Option<Vec<u8>>> {
    let path = match cache_path(url) {
        Some(path) => path,
        None => return Ok(None),
    };
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path)?;
    Ok(Some(bytes))
}

pub fn write_disk_cache(url: &str, bytes: &[u8]) -> io::Result<()> {
    let path = match cache_path(url) {
        Some(path) => path,
        None => return Ok(()),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    Ok(())
}

pub fn ensure_png(bytes: &[u8]) -> io::Result<Vec<u8>> {
    if bytes.starts_with(b"\\x89PNG") {
        return Ok(bytes.to_vec());
    }
    let img = image::load_from_memory(bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageOutputFormat::Png)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(out)
}

fn cache_path(url: &str) -> Option<std::path::PathBuf> {
    let base = cache_dir().ok()?;
    let mut dir = base;
    dir.push("images");
    let name = URL_SAFE_NO_PAD.encode(url.as_bytes());
    dir.push(name);
    Some(dir)
}

fn write_iterm2_image(
    stdout: &mut CrosstermBackend<io::Stdout>,
    bytes: &[u8],
    width: u16,
    height: u16,
) -> io::Result<()> {
    let payload = STANDARD.encode(bytes);
    let escape = format!(
        "\x1b]1337;File=inline=1;width={width}c;height={height}c;preserveAspectRatio=1:{payload}\x07"
    );
    queue!(stdout, crossterm::style::Print(escape))?;
    Ok(())
}

fn write_kitty_image(
    stdout: &mut CrosstermBackend<io::Stdout>,
    bytes: &[u8],
    width: u16,
    height: u16,
) -> io::Result<()> {
    let payload = STANDARD.encode(bytes);
    let mut chunks = payload.as_bytes().chunks(4096).peekable();
    while let Some(chunk) = chunks.next() {
        let more = if chunks.peek().is_some() { 1 } else { 0 };
        let header = format!(
            "\x1b_Ga=T,f=100,c={width},r={height},m={more};"
        );
        let mut out = String::with_capacity(header.len() + chunk.len() + 2);
        out.push_str(&header);
        out.push_str(std::str::from_utf8(chunk).unwrap_or(""));
        out.push_str("\x1b\\");
        queue!(stdout, crossterm::style::Print(out))?;
    }
    Ok(())
}
