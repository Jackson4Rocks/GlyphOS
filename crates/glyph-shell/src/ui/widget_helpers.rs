//! Shared builders for shell surfaces (glass layer windows, pills, dots).
//!
//! Font sizing/colors go through semantic CSS classes defined in
//! glyph-core::tokens::stylesheet, so every surface shares one design system.

use gtk::prelude::*;
use gtk::{Box, Label, Widget, Window};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

/// Turns &str list into GString vec (avoids Vec::into type inference pain).
pub fn cls(items: &[&str]) -> Vec<glib::GString> {
    items.iter().map(|s| glib::GString::from(*s)).collect()
}

/// Uniform margins on any widget.
pub fn margin_all<W: gtk::prelude::WidgetExt>(w: &W, n: i32) {
    w.set_margin_top(n);
    w.set_margin_bottom(n);
    w.set_margin_start(n);
    w.set_margin_end(n);
}


/// Turn a plain GTK window into a GlyphOS glass layer-surface.
pub fn glass_layer(
    win: &Window,
    layer: Layer,
    keyboard: KeyboardMode,
    anchors: &[Edge],
    margins: &[(Edge, i32)],
    exclusive: i32,
    namespace: &'static str,
) {
    win.init_layer_shell();
    win.set_layer(layer);
    win.set_keyboard_mode(keyboard);
    win.set_namespace(Some(namespace));
    for e in anchors {
        win.set_anchor(*e, true);
    }
    for (e, m) in margins {
        win.set_margin(*e, *m);
    }
    if exclusive != 0 {
        win.set_exclusive_zone(exclusive);
    }
}

/// NDot display-face label (the Doto dot-matrix face or user-supplied NDot).
/// Size is one of the `.ndot-*` scales registered in the shared stylesheet.
pub fn ndot_label(text: &str, size_class: &str) -> Label {
    Label::builder()
        .label(text)
        .css_classes(cls(&["ndot",size_class.into()]))
        .halign(gtk::Align::Start)
        .build()
}

/// Monospace technical label.
pub fn mono_label(text: &str) -> Label {
    Label::builder()
        .label(text)
        .css_classes(cls(&["mono","dim"]))
        .halign(gtk::Align::Start)
        .build()
}

/// Interface-text label (takes one or more CSS class names). Text aligns
/// left — the natural reading start — unlike GTK's centered default.
pub fn ui_label(text: &str, classes: &str) -> Label {
    let class_list: Vec<&str> = classes.split(' ').collect();
    Label::builder()
        .label(text)
        .css_classes(cls(&class_list))
        .halign(gtk::Align::Start)
        .xalign(0.0)
        .build()
}

/// Pill-shaped stat widget (icon + text) used across the panel.
pub fn pill(contents: &[Widget]) -> Box {
    let b = Box::builder()
        .css_classes(cls(&["pill"]))
        .spacing(6)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();
    for w in contents {
        b.append(w);
    }
    b
}

/// Thin vertical separator used across the panel.
pub fn vsep() -> Widget {
    let b = Box::builder().height_request(16).width_request(1).build();
    b.into()
}

/// Horizontal filler.
pub fn hsp() -> Widget {
    let b = Box::builder().build();
    b.set_hexpand(true);
    b.into()
}

/// Button of GlyphOS app style with an ndot or unicode glyph as icon.
pub fn icon_button(glyph: &str, tooltip: &str, action: impl Fn() + 'static) -> gtk::Button {
    let btn = gtk::Button::builder()
        .css_classes(cls(&["glyph-icon-btn"]))
        .tooltip_text(tooltip)
        .build();
    btn.set_child(Some(&ui_label(glyph, "body")));
    let action = std::sync::Arc::new(action);
    btn.connect_clicked(move |_| (action.clone())());
    btn
}
