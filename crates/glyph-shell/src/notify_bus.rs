//! glyph-shell notification daemon (phase 5).
//!
//! Serves the freedesktop `org.freedesktop.Notifications` interface on the
//! session D-Bus and forwards incoming notifications to the GTK main loop
//! through the existing signal channel. Real protocol; the well-known name is
//! claimed with replace + allow-replacement (standard daemon behavior). If
//! the name cannot be taken, the reason is logged and nothing is faked.

use crate::state::{Notif, ShellSignal, SignalTx};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc;
use zbus::zvariant::OwnedValue;

pub const WELLKNOWN: &str = "org.freedesktop.Notifications";
pub const PATH: &str = "/org/freedesktop/Notifications";

pub type CloseTx = mpsc::Sender<(u32, u32)>;
pub type CloseRx = mpsc::Receiver<(u32, u32)>;

/// Server-side notifier object. Real state: an id counter only; per-app
/// bookkeeping is not needed for what we support today.
pub struct NotifyServer {
    /// Shell event channel (into the GTK main-loop pump).
    pub tx: SignalTx,
    /// Next notification id when the client sends `replaces_id = 0`.
    pub next: AtomicU32,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl NotifyServer {
    fn notify(
        &self,
        app_name: String,
        replaces_id: u32,
        _app_icon: String,
        summary: String,
        body: String,
        _actions: Vec<String>,
        hints: HashMap<String, OwnedValue>,
        _expire_timeout: i32,
    ) -> u32 {
        let id = if replaces_id != 0 {
            replaces_id
        } else {
            self.next.fetch_add(1, Ordering::Relaxed)
        };
        let urgency = hints.get("urgency").and_then(|v| v.downcast_ref::<u8>().ok()).unwrap_or(0) as u32;
        let app = if app_name.is_empty() { "unknown".to_string() } else { app_name };
        self.tx
            .send(ShellSignal::Notification(Notif { id, app, summary, body, urgency }))
            .ok();
        id
    }

    fn close_notification(&self, id: u32) {
        // Reason 3 per the spec ("closed by explicit CloseNotification call").
        // The UI removes the entry and the daemon bridge echoes the signal.
        self.tx.send(ShellSignal::NotificationClosed(id, 3)).ok();
    }

    fn get_capabilities(&self) -> Vec<&'static str> {
        // Honest: body text support only — no actions, markup or sounds yet.
        vec!["body"]
    }

    fn get_server_information(&self) -> (String, String, String, String) {
        (
            "GlyphOS".into(),
            "GlyphOS Project".into(),
            glyph_core::VERSION.into(),
            "1.2".into(),
        )
    }

    /// DBus signal `NotificationClosed(id, reason)`:
    /// 1 = expired · 2 = dismissed by user · 3 = CloseNotification call.
    #[zbus(signal)]
    async fn notification_closed(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        id: u32,
        reason: u32,
    ) -> zbus::Result<()>;

    /// DBus signal `ActionInvoked(id, action_key)`. Declared for protocol
    /// completeness; no action buttons ship yet, so it is never emitted.
    #[zbus(signal)]
    async fn action_invoked(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        id: u32,
        action_key: String,
    ) -> zbus::Result<()>;
}

/// Spawns the daemon thread. Takes over `org.freedesktop.Notifications` and
/// relays every UI-side close back onto the bus through `close_rx`.
pub fn spawn(tx: SignalTx, close_rx: CloseRx) -> Result<(), String> {
    std::thread::Builder::new()
        .name("glyph-notifd".into())
        .spawn(move || serve(tx, close_rx))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn serve(tx: SignalTx, close_rx: CloseRx) {
    let server = NotifyServer { tx, next: AtomicU32::new(1) };
    let built = zbus::blocking::connection::Builder::session()
        .expect("glyph-notifd: no session bus")
        .allow_name_replacements(true)
        .replace_existing_names(true)
        .name(WELLKNOWN)
        .and_then(|b| b.serve_at(PATH, server))
        .and_then(|b| b.build());
    let conn = match built {
        Ok(c) => c,
        Err(e) => {
            eprintln!("glyph-notifd: cannot own {WELLKNOWN}: {e} (name held by another daemon)");
            return;
        }
    };
    eprintln!("glyph-notifd: serving {WELLKNOWN} on {PATH}");

    // UI → bus bridge: the GTK side decides a notification closed (user,
    // timeout or CloseNotification) and hands us (id, reason) to announce.
    let emitter = conn
        .object_server()
        .interface::<_, NotifyServer>(PATH)
        .map(|ref_| ref_.signal_emitter().clone());
    match emitter {
        Ok(e) => {
            while let Ok((id, reason)) = close_rx.recv() {
                if let Err(err) = zbus::block_on(NotifyServer::notification_closed(&e, id, reason)) {
                    eprintln!("glyph-notifd: emit NotificationClosed: {err}");
                }
            }
        }
        Err(err) => {
            eprintln!("glyph-notifd: interface lookup failed: {err}");
            while close_rx.recv().is_ok() { /* keep draining */ }
        }
    }
}
