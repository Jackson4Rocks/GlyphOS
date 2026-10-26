//! The GlyphOS dock: pinned + running apps, painted in glass, NDot initials.

use crate::apps;
use crate::state::ShellState;
use crate::ui::widget_helpers::*;
use gtk::prelude::*;
use gtk::{glib, Box as DockBox, Orientation, Window};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::sync::Arc;

pub fn build(st: &Arc<ShellState>, app: &gtk::Application) {
    let win = Window::builder().application(app).title("GlyphOS Dock").build();
    glass_layer(
        &win,
        Layer::Top,
        KeyboardMode::None,
        &[Edge::Bottom],
        &[(Edge::Bottom, 10)],
        0,
        "glyphos-dock",
    );
    LayerShell::auto_exclusive_zone_enable(&win);
    let cfg_icon = st.config.read().unwrap().dock.icon_size as i32;
    let dock = DockBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    win.set_child(Some(&dock));

    let cfg = st.config.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(2000), move || {
        while let Some(child) = dock.first_child() {
            child.unparent();
        }
        let pinned = cfg.read().unwrap().dock.pinned.clone();
        for pin in pinned {
            dock.append(&dock_item(&pin, cfg_icon));
        }
        glib::ControlFlow::Continue
    });
}

fn dock_item(pin: &str, icon_size: i32) -> gtk::Button {
    let exec = resolve_exec(pin);
    let display = display_name(pin);
    let initial = display.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or("?".into());

    let btn = gtk::Button::builder()
        .tooltip_text(&display)
        .css_classes(cls(&["dock-item"]))
        .build();

    let inner = DockBox::builder().orientation(Orientation::Horizontal).build();
    let glyph = gtk::Label::builder()
        .css_classes(cls(&["ndot","ndot-md","ink"]))
        .label(initial)
        .build();
    glyph.set_width_request(icon_size);
    glyph.set_height_request(icon_size);
    glyph.set_valign(gtk::Align::Center);
    glyph.set_halign(gtk::Align::Center);
    inner.append(&glyph);
    btn.set_child(Some(&inner));

    // Launch the real resolved command through the shared launch helper.
    btn.connect_clicked(move |_| {
        crate::dispatch::launch(&exec);
    });
    btn
}

fn display_name(pin: &str) -> String {
    let apps_list = desktop_lookup();
    apps_list.iter().find(|a| a.name.to_lowercase() == pin.to_lowercase())
        .map(|a| a.name.clone())
        .unwrap_or_else(|| pin.to_string())
}

fn resolve_exec(pin: &str) -> String {
    let apps_list = desktop_lookup();
    apps_list.iter().find(|a| a.name.to_lowercase() == pin.to_lowercase())
        .map(|a| a.exec.clone())
        .unwrap_or_else(|| pin.to_string())
}

/// One-shot desktop registry lookup (cheap enough for 2s refresh).
fn desktop_lookup() -> Vec<apps::AppInfo> {
    apps::desktop_apps()
}
