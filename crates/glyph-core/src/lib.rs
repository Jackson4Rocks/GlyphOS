//! GlyphOS core: version, config paths, persistence helpers.

pub mod config;
pub mod ipc;
pub mod tokens;

/// Re-exports so callers can use `glyph_core::Config` directly.
pub use config::{AppEntry, Config, DockPosition, DockSection};

/// Library version as reported by Cargo.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Product identity block used across apps and the shell.
pub const PRODUCT: &str = "GlyphOS";
pub const TAGLINE: &str = "Every detail. By design.";

use directories::{BaseDirs, ProjectDirs};
use std::path::PathBuf;

/// Well-known GlyphOS runtime locations.
#[derive(Debug, Clone)]
pub struct Paths {
    /// `~/.config/glyphos`
    pub config_dir: PathBuf,
    /// User data: wallpaper cache, state files.
    pub data_dir: PathBuf,
    /// Unix socket used by glyphctl and the shell.
    pub runtime_dir: PathBuf,
    /// Asset directory shipped with the OS (fonts, wallpapers, icons).
    pub assets_dir: PathBuf,
}

impl Paths {
    pub fn discover() -> Self {
        let base = BaseDirs::new().expect("glyphos requires a home directory");
        let proj = ProjectDirs::from("os", "glyph", "glyphos")
            .expect("glyphos requires an operating system");
        // Dist installs ship assets under /usr/share/glyphos; dev check repo-relative.
        let assets = share_dirs().unwrap_or_else(|| PathBuf::from("./assets"));
        Self {
            config_dir: proj.config_local_dir().to_path_buf(),
            data_dir: proj.data_dir().to_path_buf(),
            runtime_dir: base
                .runtime_dir()
                .map(|p| p.join("glyphos"))
                .unwrap_or_else(|| PathBuf::from("/tmp").join("glyphos-runtime")),
            assets_dir: assets,
        }
    }

    pub fn ensure(&self) {
        let _ = std::fs::create_dir_all(&self.config_dir);
        let _ = std::fs::create_dir_all(&self.data_dir);
        let _ = std::fs::create_dir_all(&self.runtime_dir);
    }
}

fn share_dirs() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("/usr/share/glyphos"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets"),
    ];
    candidates.into_iter().find(|p| p.exists())
}
