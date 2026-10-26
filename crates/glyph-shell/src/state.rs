//! Shared shell state: config, paths, signal channel, real system snapshots.

use glyph_core::{Config, Paths};
use glyph_system::events::SystemSnapshot;
use gtk::glib;
use std::sync::Arc;

/// Which overlay a signal refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    Command,
    Control,
    Notify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZAction {
    Toggle,
    Show,
    Hide,
}

/// Signals the IPC thread can inject into the GTK main loop.
#[derive(Debug, Clone)]
pub enum ShellSignal {
    Window(Overlay, ZAction),
    Launch(String),
    ReloadTheme,
    Workspace(u32),
    NotifyEvent(String),
}

/// Signal transport: std mpsc on purpose — glib 0.22 dropped the classic
/// channel; the main loop drains this via a tiny idle pump (main.rs).
pub type SignalTx = std::sync::mpsc::Sender<ShellSignal>;

/// Cross-window shell state. Window handles live in `dispatch::Registry`;
/// state here only carries configuration and live system data.
pub struct ShellState {
    pub paths: Paths,
    pub config: Arc<std::sync::RwLock<Config>>,
    pub tx: SignalTx,
    /// Latest real system snapshot (panel + Control Center read this).
    pub snap: std::sync::Mutex<SystemSnapshot>,
}
