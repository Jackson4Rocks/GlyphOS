//! Signal dispatch + a small registry mapping overlays to windows.
//! Handles Show/Hide/Toggle semantics consistently for every shell surface.
//!
//! GTK is single-threaded here: registry + dispatch are main-thread only.

use crate::state::{Overlay, ShellSignal, ShellState, ZAction};
use gtk::prelude::*;
use gtk::Window;
use std::sync::Arc;

/// Main-thread-only overlay registry; UI builders insert windows here.
thread_local! {
    static REG: std::cell::RefCell<std::collections::HashMap<&'static str, Window>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

pub fn register(key: &'static str, win: Window) {
    REG.with(|m| m.borrow_mut().insert(key, win));
}

/// Get a clone of the registered window handle (used by the notification
/// surface to re-present itself when a daemon event arrives).
pub fn lookup(key: &str) -> Option<Window> {
    REG.with(|m| m.borrow().get(key).cloned())
}

/// Hide every registered overlay surface. Used by desktop click and, later,
/// by global shortcuts that want "popovers out of the way".
pub fn hide_overlays() {
    REG.with(|m| {
        for w in m.borrow().values() {
            if w.is_visible() { w.hide(); }
        }
    });
}

/// Central signal handler, always executed on the GTK main thread.
pub fn handle(st: &Arc<ShellState>, sig: ShellSignal) {
    match sig {
        ShellSignal::Window(o, a) => {
            let key = match o {
                Overlay::Command => "command",
                Overlay::Control => "control",
                Overlay::Notify => "notify",
                Overlay::Quick => {
                    // Handled inline: opening always refreshes the flyout so
                    // its clock/media/notification data is never stale.
                    if let Some(w) = lookup("quick") {
                        if w.is_visible() { w.hide(); } else { w.present(); crate::ui::quick::refresh(); }
                        return;
                    }
                    ""
                }
            };
            if let Some(w) = lookup(key) {
                let show = |w: &Window| w.present();
                match a {
                    ZAction::Toggle => {
                        if w.is_visible() { w.hide() } else { show(&w) }
                    }
                    ZAction::Show => show(&w),
                    ZAction::Hide => w.hide(),
                }
            } else {
                eprintln!("glyph-shell: overlay '{key}' not registered");
            }
        }
        ShellSignal::Launch(exec) => launch(&exec),
        ShellSignal::ReloadTheme => {
            // Re-read glyphos.toml from disk: `glyphctl reload-theme` must
            // reflect external edits, not just in-memory toggles.
            let cfg = glyph_core::Config::load_or_default(&st.paths);
            if let Ok(mut g) = st.config.write() { *g = cfg.clone(); }
            reload_theme(&cfg);
        }
        ShellSignal::Workspace(i) => crate::compositor::switch_workspace(i),
        ShellSignal::Notification(n) => crate::ui::notify::on_notification(st, &n),
        ShellSignal::NotificationClosed(id, reason) => {
            crate::ui::notify::on_closed(st, id);
            st.notif_close_tx.send((id, reason)).ok();
        }
    }
}

/// Launch an application via desktop shell semantics (Field Codes honored).
pub fn launch(exec: &str) {
    // Strip desktop-exec field codes the desktop-entry spec defines.
    let cleaned: String = exec
        .split_whitespace()
        .filter(|t| !t.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ");
    let _ = std::process::Command::new("sh").arg("-c").arg(cleaned).spawn();
}

fn reload_theme(cfg: &glyph_core::Config) {
    let css = glyph_core::tokens::stylesheet(&cfg.theme);
    eprintln!("glyph-shell: theme reloaded → {}", css.lines().next().unwrap_or(""));
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&css);
    if let Some(d) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(&d, &provider, gtk::STYLE_PROVIDER_PRIORITY_USER);
    }
}
