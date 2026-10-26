//! The GlyphOS top panel: wordmark, workspace indicators (real), clock in the
//! display face, honest system stats pulled from glyph-system caches.

use crate::state::ShellState;
use crate::ui::widget_helpers::*;
use gtk::prelude::*;
use gtk::{glib, Box, Label, Orientation, Window};
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

    // Fixed-height sliver so the top layer surface maps with real content.
    root.set_height_request(36);

    // Fixed heights: a 34px sliver, styled via .panel class padding.
    win.set_child(Some(&root));

    // ——— brand ———
    root.append(&ndot_label("GLYPHOS", "ndot-sm"));
    root.append(&vsep());

    // ——— workspace indicators (real, from the compositor adapter) ———
    let row = Box::builder().orientation(Orientation::Horizontal).spacing(6).build();
    root.append(&row);
    crate::ui::panel_parts::workspaces_row(&row);

    // ——— spacer ———
    root.append(&hsp());

    // ——— honest system indicators ———
    crate::ui::panel_parts::stats_cluster(&root);

    // Panel clicks only; keyboard stays with apps.
}
