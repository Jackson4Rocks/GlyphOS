//! Application registry: parse freedesktop `.desktop` entries for real app
//! discovery (used by dock, Glyph Command, Control Center and Settings).

use glyph_core::config::AppEntry;
use std::path::{Path, PathBuf};

/// Small installed-app record from a desktop file.
#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub comment: Option<String>,
    pub no_display: bool,
}

/// All registered apps: config overrides first (pinned custom entries), then
/// system desktop files, deduplicated by stable lowercase name.
pub fn all_apps(config: &glyph_core::Config) -> Vec<AppEntry> {
    let mut out: Vec<AppEntry> = config.apps.clone();
    let mut seen: Vec<String> = out.iter().map(|a| a.name.to_lowercase()).collect();
    for info in desktop_apps() {
        if info.no_display { continue }
        if !seen.iter().any(|s| *s == info.name.to_lowercase()) {
            out.push(AppEntry {
                name: info.name.clone(),
                exec: info.exec.clone(),
                description: info.comment.unwrap_or("Installed application".into()),
                icon: info.icon.clone().or_else(|| Some(generic_icon(&info.name))),
            });
            seen.push(info.name.to_lowercase());
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

pub fn icon_for(config: &glyph_core::Config, app: &AppEntry) -> String {
    let _ = config;
    app.icon.clone().unwrap_or_else(|| generic_icon(&app.name))
}

fn generic_icon(name: &str) -> String {
    name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_else(|| "?".into())
}

/// Scan standard XDG directories for `*.desktop` files.
pub fn desktop_apps() -> Vec<AppInfo> {
    let mut dirs: Vec<PathBuf> = vec!["/usr/share/applications".into()];
    if let Ok(home) = std::env::var("HOME") {
        dirs.push(Path::new(&home).join(".local/share/applications"));
    }
    let mut out = vec![];
    for dir in dirs {
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for e in rd.flatten() {
                if e.path().extension().and_then(|s| s.to_str()) == Some("desktop") {
                    if let Some(info) = parse_desktop(&e.path()) {
                        out.push(info);
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

/// Small robust .desktop parser (enough for name/exec/icon/comment/NoDisplay).
pub fn parse_desktop(path: &Path) -> Option<AppInfo> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut name = None; let mut exec = None; let mut icon = None;
    let mut comment = None; let mut no_display = false;
    let mut in_desktop_section = false;
    for line in text.lines() {
        if line.starts_with('[') {
            in_desktop_section = line.starts_with("[Desktop Entry]");
            continue;
        }
        if !in_desktop_section { continue }
        let Some((key, value)) = line.split_once('=') else { continue };
        let v = value.trim();
        match key.trim() {
            "Name" if name.is_none() => name = Some(v.to_string()),
            "Exec" if exec.is_none() => exec = Some(v.to_string()),
            "Icon" if icon.is_none() => icon = Some(v.to_string()),
            "Comment" if comment.is_none() => comment = Some(v.to_string()),
            "NoDisplay" => no_display = v.eq_ignore_ascii_case("true"),
            _ => {}
        }
    }
    Some(AppInfo { name: name?, exec: exec?.to_string(), icon, comment, no_display })
}
