//! Glyph Control Center: the unified quick-settings glass panel with real
//! system backends (NetworkManager, PipeWire, BlueZ, backlight) and honest
//! failure reporting. Live theme switching writes glyphos.toml immediately.

use crate::state::ShellState;
use crate::ui::widget_helpers::*;
use gtk::prelude::*;
use gtk::{glib, Adjustment, Box as VBox, Button, Label, Orientation, Scale, Window};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};
use std::sync::Arc;

pub fn build(st: &Arc<ShellState>, app: &gtk::Application) {
    let win = Window::builder().application(app).title("Glyph Control").build();
    glass_layer(
        &win,
        Layer::Top,
        KeyboardMode::OnDemand,
        &[Edge::Top, Edge::Right],
        &[(Edge::Top, 46), (Edge::Right, 8)],
        0,
        "glyphos-control",
    );
    win.set_default_size(340, 430);
    win.add_css_class("glyph-root");
    win.add_css_class("glass");

    let root = VBox::builder().orientation(Orientation::Vertical).spacing(10).build();
    margin_all(&root, 12);
    win.set_child(Some(&root));

    root.append(&ndot_label("CONTROL", "ndot-sm"));

    // ——— tiles with real behavior ———
    wifi_tile(&root);
    blue_tile(&root);
    audio_tile(&root, st);
    bright_tile(&root);
    theme_tile(&root, st);

    // Footer: honest hints + real session actions where supported.
    let hint = ui_label("Toggles reflect real services · unsupported shown as such", "small dim");
    hint.set_wrap(true);
    root.append(&hint);

    let lock = Button::builder()
        .label("lock screen (phase 5)")
        .css_classes(cls(&["list-row","small","dim"]))
        .build();
    lock.connect_clicked(|_| {
        eprintln!("glyph-control: session locking lands with glyphwm (phase 5)");
    });
    root.append(&lock);

    crate::dispatch::register("control", win.clone());
}

// ---------------------------------------------------------------- tiles
fn title_sub_tile(title: &str, sub: &str) -> (VBox, Label) {
    let vbox = VBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .css_classes(cls(&["list-row","hoverable"]))
        .build();
    vbox.append(&ndot_label_u(title));
    let sub_lbl = ui_label(sub, "small dim");
    vbox.append(&sub_lbl);
    (vbox, sub_lbl)
}

fn ndot_label_u(title: &str) -> Label {
    ndot_label(title, "ndot-xs")
}

fn wifi_tile(root: &VBox) {
    let (tile, sub) = title_sub_tile("WI-FI", "checking…");
    root.append(&tile);
    glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        let snap = glyph_system::events::snap_cache().read().map(|s| s.clone()).unwrap_or_default();
        let net_text = match (snap.network.wifi_enabled, snap.network.ssid.as_deref()) {
            (true, Some(ssid)) => format!("{ssid} · {}%", snap.network.strength),
            (true, None) => "Wi-Fi on · not connected".into(),
            (false, _) => "Wi-Fi off".into(),
        };
        sub.set_text(&net_text);
        glib::ControlFlow::Continue
    });
}

fn blue_tile(root: &VBox) {
    let (tile, sub) = title_sub_tile("BLUETOOTH", "checking…");
    root.append(&tile);
    glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
        let bt = glyph_system::bluetooth::bluetooth_state();
        sub.set_text(&if bt.powered { "Bluetooth on" } else { "Bluetooth off" });
        glib::ControlFlow::Continue
    });
}

fn audio_tile(root: &VBox, _st: &Arc<ShellState>) {
    let tile = VBox::builder().orientation(Orientation::Vertical).spacing(4)
        .css_classes(cls(&["list-row"])).build();
    tile.append(&ndot_label_u("VOLUME"));
    let adj = Adjustment::new(50.0, 0.0, 100.0, 1.0, 5.0, 0.0);
    let scale = Scale::builder().orientation(Orientation::Horizontal).adjustment(&adj).build();
    tile.append(&scale);
    root.append(&tile);

    let _s = scale.clone();
    tile_guard(&scale);
}

fn tile_guard(scale: &Scale) {
    // Live feedback text (draw_value off keeps it clean; report in the tile).
    let s = scale.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(1200), move || {
        let snap = glyph_system::events::snap_cache().read().map(|x| x.clone()).unwrap_or_default();
        if s.is_sensitive() {
            s.set_value(snap.audio.volume as f64);
        }
        glib::ControlFlow::Continue
    });
    let wid = scale.clone();
    scale.connect_value_changed(move |sc| {
        let v = sc.value() as u8;
        if let Err(e) = glyph_system::pulse::set_volume(v) {
            eprintln!("glyph-control: volume: {e}");
        }
        let _ = &wid;
    });
}

fn bright_tile(root: &VBox) {
    let tile = VBox::builder().orientation(Orientation::Vertical).spacing(4)
        .css_classes(cls(&["list-row"])).build();
    tile.append(&ndot_label_u("BRIGHTNESS"));
    let note = ui_label("", "small dim");
    tile.append(&note);
    let adj = Adjustment::new(80.0, 1.0, 100.0, 1.0, 5.0, 0.0);
    let scale = Scale::builder().orientation(Orientation::Horizontal).adjustment(&adj).build();
    tile.append(&scale);
    root.append(&tile);

    let sc = scale.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
        let snap = glyph_system::events::snap_cache().read().map(|s| s.clone()).unwrap_or_default();
        if snap.brightness.available {
            sc.set_value(snap.brightness.percent as f64);
            note.set_text("");
        } else {
            note.set_text(&snap.brightness.reason);
            sc.set_sensitive(false);
        }
        glib::ControlFlow::Continue
    });
    scale.connect_value_changed(|s| {
        if let Err(e) = glyph_system::brightness::set_brightness(s.value() as u8) {
            eprintln!("glyph-control: brightness: {e}");
        }
    });
}

fn theme_tile(root: &VBox, st: &Arc<ShellState>) {
    let (tile, sub) = title_sub_tile("MODE", "AMOLED · Crystal");
    root.append(&tile);

    let mode_input = Button::builder().label("Switch").css_classes(cls(&["pill","body"])).build();
    tile.append(&mode_input);

    // Sync label with the stored appearance on a slow refresh.
    let cfg = st.config.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(2000), move || {
        let mode = cfg.read().unwrap().theme.appearance;
        sub.set_text(&match mode {
            glyph_core::tokens::AppearanceMode::Dark => "AMOLED BLACK",
            glyph_core::tokens::AppearanceMode::Light => "CRYSTAL WHITE",
            _ => "Auto",
        });
        glib::ControlFlow::Continue
    });

    // Real action: persist appearance and hand the new stylesheet to the shell.
    let cfg = st.config.clone();
    let tx = st.tx.clone();
    mode_input.connect_clicked(move |_| {
        let mut g = cfg.write().unwrap();
        g.theme.appearance = match g.theme.appearance {
            glyph_core::tokens::AppearanceMode::Dark => glyph_core::tokens::AppearanceMode::Light,
            _ => glyph_core::tokens::AppearanceMode::Dark,
        };
        let new_cfg = g.clone();
        drop(g);
        if let Err(e) = new_cfg.save(&glyph_core::Paths::discover()) {
            eprintln!("glyph-control: config save failed: {e}");
        } else {
            tx.send(crate::state::ShellSignal::ReloadTheme).ok();
        }
    });

    let _ = sub;
}
