//! Notification strip fed by the real D-Bus daemon (`notify_bus`). Shows the
//! newest entries stacked, auto-hides when quiescent, honors user dismissal
//! and reports every close back through the daemon bridge.

use crate::state::{Notif, ShellState};
use crate::ui::widget_helpers::glass_layer;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};
use gtk::glib;
use gtk::prelude::*;
use gtk::Window;
use std::sync::Arc;

/// Maximum entries rendered in one popup pass (newest first).
const MAX_SHOWN: usize = 5;
/// Quiescence window after the newest notification, before auto-hide.
const HIDE_AFTER_MS: u64 = 6000;
/// Close reason: dismissed by user (freedesktop spec §close reason codes).
const REASON_USER: u32 = 2;

thread_local! {
    /// Highest epoch wins: stale auto-hide timers abort on mismatch.
    static EPOCH: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// Popup content container, rebuilt on every store change.
    static CONTENT: std::cell::RefCell<Option<gtk::Box>> = const { std::cell::RefCell::new(None) };
}

pub fn build(_st: &Arc<ShellState>, app: &gtk::Application) {
    let win = Window::builder().application(app).title("Glyph Notifications").build();
    glass_layer(
        &win,
        Layer::Top,
        KeyboardMode::None,
        &[Edge::Top, Edge::Right],
        &[(Edge::Top, 92), (Edge::Right, 8)],
        0,
        "glyphos-notify",
    );
    win.set_default_size(320, 90);
    win.add_css_class("glyph-root");
    win.add_css_class("glass");

    let inner = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(6).build();
    CONTENT.with(|c| *c.borrow_mut() = Some(inner.clone()));
    win.set_child(Some(&inner));

    render(_st);
    crate::dispatch::register("notify", win.clone());
}

/// Daemon event: append + redraw + popup.
pub fn on_notification(st: &Arc<ShellState>, n: &Notif) {
    {
        let mut d = st.notifs.lock().unwrap();
        d.push_back(n.clone());
        while d.len() > 50 { d.pop_front(); }
    }
    render(st);
    present();
    schedule_hide();
}

/// Daemon close event (user dismiss fallback + CloseNotification calls).
/// Removal only; the bus-side echo happens via `notif_close_tx` in dispatch.
pub fn on_closed(st: &Arc<ShellState>, id: u32) {
    let removed = {
        let mut d = st.notifs.lock().unwrap();
        let before = d.len();
        d.retain(|n| n.id != id);
        d.len() != before
    };
    if removed {
        render(st);
        if store_is_empty(st) {
            // Nothing left to show; stop the popup unless the user forced it.
            hide();
        }
    }
}

/// Click-to-dismiss from the popup row. The dispatch arm removes the entry
/// and echoes `NotificationClosed(id, 2)` on the bus via the daemon bridge.
fn dismiss(st: &Arc<ShellState>, id: u32) {
    st.tx.send(crate::state::ShellSignal::NotificationClosed(id, REASON_USER)).ok();
}

// ---------------------------------------------------------------- internals

fn store_is_empty(st: &Arc<ShellState>) -> bool {
    st.notifs.lock().unwrap().is_empty()
}

fn render(st: &Arc<ShellState>) {
    CONTENT.with(|c| {
        let inner = c.borrow().clone();
        let Some(inner) = inner else { return };
        while let Some(child) = inner.first_child() {
            child.unparent();
        }

        let title = crate::ui::widget_helpers::ndot_label("NOTIFICATIONS", "ndot-xs");
        inner.append(&title);

        let d = st.notifs.lock().unwrap();
        if d.is_empty() {
            let body = crate::ui::widget_helpers::ui_label("No notifications.", "small dim");
            body.set_wrap(true);
            inner.append(&body);
            return;
        }
        let newest: Vec<&Notif> = d.iter().rev().take(MAX_SHOWN).collect();
        let st2 = Arc::clone(st);
        for n in newest {
            let row = notification_row(n, &st2);
            inner.append(&row);
        }
    });
}

fn notification_row(n: &Notif, st: &Arc<ShellState>) -> gtk::Button {
    let btn = gtk::Button::builder()
        .css_classes(cls("list-row notif-row"))
        .has_frame(false)
        .build();
    let col = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(2).build();

    let header = crate::ui::widget_helpers::ui_label(&n.summary, "ink");
    header.set_xalign(0.0);
    header.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let body = crate::ui::widget_helpers::ui_label(&n.body, "small dim");
    body.set_wrap(true);
    if n.body.is_empty() {
        body.set_visible(false);
    }
    let app = crate::ui::widget_helpers::ui_label(&n.app, "ndot-xs dim");
    app.set_xalign(0.0);

    col.append(&header);
    col.append(&body);
    col.append(&app);
    btn.set_child(Some(&col));

    let st_click = Arc::clone(st);
    let id = n.id;
    btn.connect_clicked(move |_| dismiss(&st_click, id));
    btn
}

fn present() {
    if let Some(w) = crate::dispatch::lookup("notify") {
        w.present();
    }
}

fn hide() {
    if let Some(w) = crate::dispatch::lookup("notify") {
        w.hide();
    }
}

fn schedule_hide() {
    let my = EPOCH.with(|e| {
        let v = e.get() + 1;
        e.set(v);
        v
    });
    glib::timeout_add_local(std::time::Duration::from_millis(HIDE_AFTER_MS), move || {
        let live = EPOCH.with(|e| e.get()) == my;
        if live {
            hide();
        }
        glib::ControlFlow::Break
    });
}

/// Space-separated class ids → the css_classes array GTK expects here.
fn cls(classes: &str) -> Vec<&str> {
    classes.split_whitespace().collect()
}
