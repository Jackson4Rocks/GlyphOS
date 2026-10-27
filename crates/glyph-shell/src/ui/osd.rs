//! On-screen display: the small bubble near the dock that mirrors volume and
//! brightness while the keys are pressed (like a real desktop's OSD). The
//! value shown is always the level the adapter reports back after the change.

use crate::state::{OsdKind, ShellSignal, ShellState, SignalTx};
use crate::ui::widget_helpers::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};
use gtk::glib;
use gtk::prelude::*;
use gtk::Window;
use std::sync::Arc;

struct OsdState {
    icon: gtk::Label,
    level: gtk::Label,
    win: Window,
}

thread_local! {
    static OSD: std::cell::RefCell<Option<OsdState>> = const { std::cell::RefCell::new(None) };
}

pub fn build(st: &Arc<ShellState>, app: &gtk::Application) {
    let win = Window::builder().application(app).title("GlyphOS OSD").build();
    // Anchored to both side edges so the bubble can genuinely center above
    // the dock; explicit geometry keeps it a tight bubble, not a full strip.
    glass_layer(
        &win,
        Layer::Top,
        KeyboardMode::None,
        &[Edge::Bottom, Edge::Left, Edge::Right],
        &[(Edge::Bottom, 96)],
        0,
        "glyphos-osd",
    );
    win.add_css_class("glyph-root");

    let bubble = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .css_classes(cls(&["glass-pop", "osd"]))
        .build();
    let icon = ui_label("♪", "ndot-md ink");
    let level = ui_label("100", "ndot-md ink");
    level.set_width_chars(4);
    bubble.append(&icon);
    bubble.append(&level);
    win.set_child(Some(&bubble));

    // Bubble contents never change shape: one render, honest values after.
    OSD.with(|s| *s.borrow_mut() = Some(OsdState { icon, level, win: win.clone() }));
    crate::dispatch::register("osd", win);
    let _ = st;
}

/// Show the OSD with the value a worker thread just applied.
pub fn show(kind: OsdKind, level: i32, muted: bool) {
    OSD.with(|s| {
        let Some(ref state) = *s.borrow() else { return };
        let icon = kind.icon_glyph(muted);
        let text = if muted { "—".to_string() } else { level.to_string() };
        state.icon.set_text(icon);
        state.level.set_text(&text);
        if !state.win.is_visible() {
            state.win.present();
        }
    });

    // Pop out again shortly after the last key event.
    glib::timeout_add_local(std::time::Duration::from_millis(1600), || {
        OSD.with(|s| {
            if let Some(state) = s.borrow().as_ref() {
                if state.win.is_visible() { state.win.hide(); }
            }
        });
        glib::ControlFlow::Break
    });
}

impl OsdKind {
    fn icon_glyph(self, muted: bool) -> &'static str {
        match self {
            OsdKind::Volume => if muted { "♪✕" } else { "♪" },
            OsdKind::Brightness => "☀",
        }
    }
}

/// Apply a +/- step on the real adapter from a worker thread (the adapters
/// shell out; never run them with the UI blocked), then surface the result.
pub fn adjust(kind: OsdKind, dir: i32, tx: SignalTx) {
    std::thread::spawn(move || {
        let result = match kind {
            OsdKind::Volume => {
                let cur = glyph_system::pulse::audio_state().volume as i32;
                let next = (cur + dir).clamp(0, 100);
                match glyph_system::pulse::set_volume(next as u8) {
                    Ok(()) => (next, glyph_system::pulse::audio_state().muted),
                    Err(_) => (cur, glyph_system::pulse::audio_state().muted),
                }
            }
            OsdKind::Brightness => {
                let cur = glyph_system::brightness::brightness_state().percent as i32;
                let next = (cur + dir).clamp(0, 100);
                match glyph_system::brightness::set_brightness(next as u8) {
                    Ok(()) => (next, false),
                    Err(_) => (cur, false),
                }
            }
        };
        tx.send(ShellSignal::Osd(kind, result.0, result.1)).ok();
    });
}
