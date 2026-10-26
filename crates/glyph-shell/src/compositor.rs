//! Compositor adapter — GlyphOS's interface layer to the active Wayland
//! compositor. This is how the shell stays compositor-agnostic: today's
//! *development fallback* is Hyprland (a running, complete compositor);
//! `compositor/glyphwm` (Smithay) will speak through the same adapter once
//! implemented. All window/workspace state comes from real IPC — nothing is
//! simulated.
//!
//! Dispatch surface used by the shell:
//! - `workspaces()`  → real virtual desktops of the active backend
//! - `switch_workspace(i)`
//! - `windows()`     → mapped windows (class, title, workspace)
//! - `focus(addr)`   → raise + focus a specific window
//! - `close_focused()`

use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
pub struct HWindow {
    pub address: String,
    pub class: String,
    pub title: String,
    pub workspace: WorkspaceRef,
    pub mapped: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceRef {
    pub id: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HWorkspace {
    pub id: i32,
    pub name: Option<String>,
}

/// Cached workspace state so UI ticks never block on subprocess IPC.
pub fn ws_cache() -> &'static std::sync::RwLock<Vec<(i32, String, bool)>> {
    static CACHE: std::sync::OnceLock<std::sync::RwLock<Vec<(i32, String, bool)>>> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::RwLock::new(Vec::new()))
}

/// Detect the active backend in priority order.
pub fn backend_name() -> &'static str {
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        "hyprland"
    } else {
        "unknown"
    }
}

/// Cached mapped windows so UI ticks never block on subprocess IPC.
pub fn window_cache() -> &'static std::sync::RwLock<Vec<HWindow>> {
    static CACHE: std::sync::OnceLock<std::sync::RwLock<Vec<HWindow>>> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::RwLock::new(Vec::new()))
}

/// Launch the background refresh loop for cached compositor state.
pub fn start_polling() {
    if backend_name() != "hyprland" {
        return; // caches stay empty; future adapters (glyphwm) add support
    }
    std::thread::Builder::new().name("glyph-ws-poll".into()).spawn(|| loop {
        let active = active_workspace();
        let now_ws = hypr_json::<Vec<HWorkspace>>(&["-j", "workspaces"])
            .unwrap_or_default()
            .into_iter()
            .map(|w| {
                let id = w.id;
                (id, w.name.clone().unwrap_or_else(|| format!("{id}")), id == active)
            })
            .collect::<Vec<_>>();
        if let Ok(mut c) = ws_cache().write() { *c = now_ws; }
        let now_wins = windows_now();
        if let Ok(mut c) = window_cache().write() { *c = now_wins; }
        std::thread::sleep(std::time::Duration::from_millis(700));
    }).ok();
}

fn windows_now() -> Vec<HWindow> {
    hypr_json::<Vec<HWindow>>(&["-j", "clients"])
        .unwrap_or_default()
        .into_iter()
        .filter(|w| w.mapped && w.workspace.id >= 1)
        .collect()
}

fn hyprctl(args: &[&str]) -> Option<String> {
    Command::new("hyprctl").args(args).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
}

fn hypr_json<T: serde::de::DeserializeOwned>(args: &[&str]) -> Option<T> {
    hyprctl(args).and_then(|s| serde_json::from_str(&s).ok())
}

pub fn active_workspace() -> i32 {
    hypr_json::<HWorkspace>(&["-j", "activeworkspace"]).map(|w| w.id).unwrap_or(1)
}

pub fn switch_workspace(i: u32) {
    match backend_name() {
        "hyprland" => {
            let _ = hyprctl(&["dispatch", "workspace", &format!("{i}")]);
        }
        _ => eprintln!("glyph-shell: no compositor backend for workspace switching"),
    }
}

pub fn windows() -> Vec<HWindow> {
    window_cache().read().unwrap().clone()
}

pub fn focus(addr: &str) {
    match backend_name() {
        "hyprland" => {
            let _ = hyprctl(&["dispatch", "focuswindow", &format!("address:{addr}")]);
        }
        _ => eprintln!("glyph-shell: no compositor backend for window focus"),
    }
}

pub fn close_focused() {
    match backend_name() {
        "hyprland" => {
            let _ = hyprctl(&["dispatch", "killactive"]);
        }
        _ => eprintln!("glyph-shell: no compositor backend for window close"),
    }
}
