//! Panel internals split from panel.rs.

use crate::compositor;
use crate::ui::widget_helpers::*;
use gtk::prelude::*;
use gtk::{glib, Box, Button, Label};
use std::sync::Arc;

/// Real workspace indicator row: one dot per existing workspace, active one
/// highlighted. Reads from the cached compositor state (no main-thread IPC).
pub fn workspaces_row(row: &Box) {
    let row = row.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(700), move || {
        // Remove stale children.
        while let Some(child) = row.first_child() {
            child.unparent();
        }
        for (id, _name, active) in compositor::ws_cache().read().unwrap().iter().cloned() {
            let btn = Button::builder()
                .label(workspace_label(active))
                .css_classes(cls(&["workspace-pill", if active { "focused" } else { "dim" }]))
                .tooltip_text(format!("Workspace {id}"))
                .build();
            btn.set_margin_start(2);
            btn.set_margin_end(2);
            let idc = id;
            btn.connect_clicked(move |_| compositor::switch_workspace(idc as u32));
            row.append(&btn);
        }
        glib::ControlFlow::Continue
    });
}

fn workspace_label(active: bool) -> String {
    if active {
        "•••".into()
    } else {
        "•".into()
    }
}

/// Honest system-stat cluster with 1-second refresh, driven by real adapters
/// through the glyph-system cached snapshot. Each entry opens the Control
/// Center on click (real destination, not decoration).
pub fn stats_cluster(root: &Box, st: &Arc<crate::state::ShellState>) {
    use crate::state::{Overlay, ShellSignal, ZAction};
    let net0 = Label::builder().css_classes(cls(&["dim","body"])).build();
    let audio0 = Label::builder().css_classes(cls(&["dim","body"])).build();
    let bt0 = Label::builder().css_classes(cls(&["dim","body"])).build();
    let bat0 = Label::builder().css_classes(cls(&["dim","body"])).build();
    let net = net0.clone();
    let audio_label = audio0.clone();
    let bt = bt0.clone();
    let bat = bat0.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        let snap = glyph_system::events::snap_cache().read().map(|s| s.clone()).unwrap_or_default();
        let net_text = match snap.network.state.as_str() {
            "connected" => format!("⬈ {}{}", snap.network.strength, if snap.network.ethernet_connected { "⎆" } else { "" }),
            "connecting" => "⬈…".into(),
            _ => "⬈✕".into(),
        };
        net.set_text(&net_text);
        if snap.audio.muted {
            audio_label.set_text("♪✕");
        } else {
            audio_label.set_text(&format!("♪ {}%", snap.audio.volume));
        }
        bt.set_text(if !snap.bluetooth.available { "" } else if snap.bluetooth.powered { "ᛒ" } else { "ᛒ✕" });
        if snap.battery.available {
            let arrow = match snap.battery.status.as_str() {
                "Charging" => "⯈",
                "Discharging" => "⯇",
                "Full" => "⯈",
                _ => "",
            };
            bat.set_text(&format!("⯀ {}%{}", snap.battery.percent, arrow));
        } else {
            bat.set_text("");
        }
        glib::ControlFlow::Continue
    });

    // Right-side entries are one-button access to the Control Center.
    for w in [net0, audio0, bt0, bat0] {
        let btn = Button::builder().css_classes(cls(&["pill", "panel-stat"])).build();
        btn.set_child(Some(&w));
        let tx = st.tx.clone();
        btn.connect_clicked(move |_| {
            tx.send(ShellSignal::Window(Overlay::Control, ZAction::Toggle)).ok();
        });
        root.append(&btn);
    }
}
/// Dot-matrix clock text; respects nothing extra yet (24h default).
fn local_clock() -> String {
    use chrono::Local;
    Local::now().format("%H:%M").to_string()
}
