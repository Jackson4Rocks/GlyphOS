//! Shared shell state: config, paths, signal channel, real system snapshots.

use glyph_core::{Config, Paths};
use std::collections::VecDeque;
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
    /// Real D-Bus notification (phase 5 daemon).
    Notification(Notif),
    /// Peer closed a notification (reason 1=user, 2=daemon).
    NotificationClosed(u32, u32),
}

/// One notification as delivered over `org.freedesktop.Notifications`.
#[derive(Debug, Clone)]
pub struct Notif {
    pub id: u32,
    pub app: String,
    pub summary: String,
    pub body: String,
    /// Raw hint-driven urgency (0 normal, 1 low, 2 critical).
    pub urgency: u32,
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
    /// Live notification history (popup strip + future history view).
    pub notifs: Arc<std::sync::Mutex<VecDeque<Notif>>>,
    /// (id, reason) → notification-daemon bridge (GTK → bus emitter thread).
    pub notif_close_tx: std::sync::mpsc::Sender<(u32, u32)>,
}

/// Local (non-D-Bus) notification ids: huge namespace so they can never
/// collide with real bus daemon ids.
pub fn next_local_notif_id() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(1_000_000_000);
    N.fetch_add(1, Ordering::Relaxed)
}
