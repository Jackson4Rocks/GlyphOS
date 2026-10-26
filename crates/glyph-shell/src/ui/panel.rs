//! The GlyphOS top panel: wordmark, workspace indicators (real), centered
//! display-face clock (opens Control Center), honest system stats on the
//! right pulled from glyph-system caches. This IS the top bar of the OS.

use crate::state::ShellState;
use crate::ui::widget_helpers::*;
use gtk::prelude::*;
use gtk::{glib, Box, Button, Orientation, Window};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};
use std::sync::Arc;

pub fn build(st: &Arc<ShellState>, app: &gtk::Application) {
    let win = Window::builder().application(app).title("GlyphOS Panel").build();
    glass_layer(
        &win,
        Layer::Top,
        KeyboardMode::None,
        &[Edge::Top, Edge::Left, Edge::Right],
        &[],
        0,
        "glyphos-panel",
    );
    win.add_css_class("glyph-root");
    win.add_css_class("glass");
    win.add_css_class("panel");

    let root = Box::builder().orientation(Orientation::Horizontal).spacing(12).build();
    root.set_height_request(38);
    win.set_child(Some(&root));

    // ——— left: brand + real workspace indicators ———
    let left = Box::builder().orientation(Orientation::Horizontal).spacing(10).build();
    left.append(&ndot_label("GLYPHOS", "ndot-sm"));
    left.append(&vsep());
    let row = Box::builder().orientation(Orientation::Horizontal).spacing(6).build();
    left.append(&row);
    crate::ui::panel_parts::workspaces_row(&row);
    root.append(&left);

    // ——— center: the clock, clickable → Control Center ———
    let center = Box::builder().orientation(Orientation::Horizontal).hexpand(true).build();
    let clk_btn = Button::builder()
        .css_classes(cls(&["pill", "ndot", "ndot-md", "ink"]))
        .tooltip_text("GlyphOS Control Center")
        .build();
    let clock = gtk::Label::builder().css_classes(cls(&["ndot", "ndot-md", "ink"])).build();
    clk_btn.set_child(Some(&clock));
    let tx = st.tx.clone();
    clk_btn.connect_clicked(move |_| {
        tx.send(crate::state::ShellSignal::Window(
            crate::state::Overlay::Control,
            crate::state::ZAction::Toggle,
        )).ok();
    });
    center.append(&centered(&clk_btn));
    root.append(&center);

    // ——— right: honest system indicators ———
    let right = Box::builder().orientation(Orientation::Horizontal).spacing(8).build();
    root.append(&right);
    crate::ui::panel_parts::stats_cluster(&right, st);

    // Clock tick lives on the panel (1 s, Doto digits, 24h).
    glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        use chrono::Local;
        clock.set_text(&Local::now().format("%H:%M").to_string());
        glib::ControlFlow::Continue
    });

    // This GTK/gtk4-layer-shell combo no longer maps creation-time windows
    // implicitly (verified 2026-10: old build failed identically); the panel
    // is always visible, so present immediately.
    win.present();
}

/// Keep the clock pinned to the true center of a full-width panel.
fn centered(w: &Button) -> Box {
    let b = Box::builder().orientation(Orientation::Horizontal).build();
    b.set_halign(gtk::Align::Center);
    b.append(w);
    b
}
