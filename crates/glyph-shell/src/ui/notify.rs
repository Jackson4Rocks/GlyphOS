//! Notification strip + honest placeholder history. A real notification
//! daemon (D-Bus `org.freedesktop.Notifications`) lands in phase 5; the
//! surface, registry and material are in place now.

use crate::ui::widget_helpers::*;
use gtk::prelude::*;
use gtk::Window;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};
use std::collections::VecDeque;
use std::sync::Arc;

pub fn build(_st: &Arc<crate::state::ShellState>, app: &gtk::Application) {
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
    win.set_default_size(300, 90);
    win.add_css_class("glyph-root");
    win.add_css_class("glass");

    let inner = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(6).build();

    let title = ndot_label("NOTIFICATIONS", "ndot-xs");
    let body = ui_label("No notifications yet. The D-Bus daemon ships in phase 5.", "small dim");
    body.set_wrap(true);

    inner.append(&title);
    inner.append(&body);
    windows_append(&win, &inner);

    crate::dispatch::register("notify", win.clone());
}

fn windows_append(win: &Window, w: &gtk::Box) {
    win.set_child(Some(w));
}

/// History (kept for phase 5 daemon; currently fed by test events only).
#[derive(Default)]
pub struct History {
    events: VecDeque<String>,
}

impl History {
    pub fn push_line(&mut self, s: &str) {
        self.events.push_back(s.to_string());
        if self.events.len() > 50 {
            self.events.pop_front();
        }
    }
}

pub fn push_notification(h: &Arc<std::sync::Mutex<History>>, text: &str) {
    h.lock().unwrap().push_line(text);
}
