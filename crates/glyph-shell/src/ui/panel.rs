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

    // ——— left: app-grid mark + real workspace indicators ———
    let left = Box::builder().orientation(Orientation::Horizontal).spacing(10).build();
    left.append(&logo_mark());
    left.append(&vsep());
    let row = Box::builder().orientation(Orientation::Horizontal).spacing(6).build();
    left.append(&row);
    crate::ui::panel_parts::workspaces_row(&row);
    root.append(&left);

    // ——— center: the clock, clickable → Quick flyout ———
    // Two equal expanding spacers pin the button to the true center.
    // (Nested halign-Center inside hexpand doesn't center in this GTK combo.)
    let clk_btn = Button::builder()
        .css_classes(cls(&["pill", "ndot", "ndot-md", "ink"]))
        .tooltip_text("Calendar · Media · Notifications")
        .build();
    let clock = gtk::Label::builder().css_classes(cls(&["ndot", "ndot-md", "ink"])).build();
    clk_btn.set_child(Some(&clock));
    let tx = st.tx.clone();
    clk_btn.connect_clicked(move |_| {
        tx.send(crate::state::ShellSignal::Window(
            crate::state::Overlay::Quick,
            crate::state::ZAction::Toggle,
        )).ok();
    });
    let sp1 = Box::builder().orientation(Orientation::Horizontal).hexpand(true).build();
    let sp2 = Box::builder().orientation(Orientation::Horizontal).hexpand(true).build();
    root.append(&sp1);
    root.append(&clk_btn);
    root.append(&sp2);

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

/// 2×2 app-grid mark: rounded tiles, asymmetric fills, single red accent tile.
/// An original mark — a nod to launcher grids, not a Windows clone.
fn logo_mark() -> gtk::Grid {
    use gtk::Grid;
    let g = Grid::builder().row_spacing(3).column_spacing(3).margin_start(4).margin_end(6).build();
    let mk = |cells: &[(&str, i32, i32)]| {
        for (c, col, row) in cells {
            let t = gtk::Box::builder()
                .width_request(9)
                .height_request(9)
                .css_classes(cls(&["logo-tile", c]))
                .build();
            g.attach(&t, *col, *row, 1, 1);
        }
    };
    // Arrangement: solid / accent / outlined / dim.
    mk(&[("t1", 0, 0), ("t2", 1, 0), ("t3", 0, 1), ("t4", 1, 1)]);
    g
}
