//! GlyphOS configuration: load / save `glyphos.toml`, expose typed sections.

use crate::Paths;
use crate::tokens::Tokens;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Dock configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockSection {
    pub pinned: Vec<String>,
    pub position: DockPosition,
    pub icon_size: u16,
    pub autohide: bool,
}

impl Default for DockSection {
    fn default() -> Self {
        Self {
            pinned: vec!["glyphos-terminal".into(), "glyphos-files".into(), "glyphos-settings".into(), "glyphos-store".into(), "glyphos-monitor".into()],
            position: DockPosition::Bottom,
            icon_size: 44,
            autohide: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DockPosition { Bottom, Left, Right }

/// Third-party apps the shell launches via Glyph Command / dock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppEntry {
    /// Human name.
    pub name: String,
    /// Executable as launched.
    pub exec: String,
    /// Short TUI-ish description for the command bar.
    pub description: String,
    /// Optional icon (desktop-file name or path).
    pub icon: Option<String>,
}

/// Root config schema. Serialized at `~/.config/glyphos/glyphos.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Aesthetic tokens (theme, glass, scale, fonts).
    pub theme: Tokens,
    pub dock: DockSection,
    /// Extra applications configured by the user.
    #[serde(rename = "apps")]
    pub apps: Vec<AppEntry>,
    /// Wallpaper path (absolute or `assets-wallpapers:<file>`).
    pub wallpaper: Option<String>,
}

impl Config {
    /// Locate the config file on disk without creating it.
    pub fn file_path(paths: &Paths) -> PathBuf {
        paths.config_dir.join("glyphos.toml")
    }

    pub fn load(paths: &Paths) -> Result<Self, String> {
        let f = Self::file_path(paths);
        let text = std::fs::read_to_string(&f)
            .map_err(|e| format!("read {}: {e}", f.display()))?;
        let cfg: Config = toml::from_str(&text)
            .map_err(|e| format!("parse glyphos.toml: {e}"))?;
        Ok(cfg)
    }

    pub fn save(&self, paths: &Paths) -> Result<(), String> {
        let f = Self::file_path(paths);
        if let Some(dir) = f.parent() {
            std::fs::create_dir_all(dir).ok();
        }
        let s = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&f, s).map_err(|e| format!("write {}: {e}", f.display()))
    }

    /// Load-once-and-create-if-missing helper for the shell.
    pub fn load_or_default(paths: &Paths) -> Self {
        Self::load(paths).unwrap_or_else(|_| {
            let c = Self::default();
            let _ = c.save(paths);
            c
        })
    }
}
