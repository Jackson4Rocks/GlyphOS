//! The GlyphOS desktop: a bottom-layer surface that owns the wallpaper and
//! behaves like a real DE desktop — clicking empty space dismisses every
//! open overlay (quick panel, control center, popups like these).

use crate::state::ShellState;
use crate::ui::widget_helpers::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};
use gtk::prelude::*;
use gtk::Window;
use std::sync::Arc;

pub fn build(st: &Arc<ShellState>, app: &gtk::Application) {
    let win = Window::builder().application(app).title("GlyphOS Desktop").build();
    glass_layer(
        &win,
        Layer::Bottom,
        KeyboardMode::None,
        &[Edge::Top, Edge::Bottom, Edge::Left, Edge::Right],
        &[],
        0,
        "glyphos-desktop",
    );
    win.add_css_class("glyph-root");
    win.add_css_class("desktop");

    let root = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    win.set_child(Some(&root));

    match configured_wallpaper(st) {
        Some(path) => {
            let pic = gtk::Picture::builder()
                .content_fit(gtk::ContentFit::Cover)
                .build();
            pic.set_filename(Some(&path));
            root.append(&pic);
        }
        None => {
            // No wallpaper configured: paint the theme background and drop the
            // signature dot grid plus a quiet wordmark so the desktop still
            // reads as GlyphOS, not as an unstyled void.
            let mark = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .valign(gtk::Align::Center)
                .halign(gtk::Align::Center)
                .spacing(10)
                .css_classes(cls(&["dotgrid", "desktop-mark"]))
                .build();
            mark.append(&ndot_label("GLYPHOS", "ndot-xl dim"));
            mark.append(&ui_label("Every detail. By design.", "body dim"));
            root.append(&mark);
        }
    }

    // Clicking anywhere on the desktop is "dismiss", like every mainstream
    // desktop environment's root space.
    let click = gtk::GestureClick::new();
    click.connect_pressed(|_, _, _, _| crate::dispatch::hide_overlays());
    win.add_controller(click);

    win.present();
}

/// Wallpaper path from `wallpaper` in glyphos.toml. Accepts an absolute path
/// or the `assets-wallpapers:<file>` shorthand shipped with the OS.
fn configured_wallpaper(st: &Arc<ShellState>) -> Option<std::path::PathBuf> {
    let raw = st.config.read().ok()?.wallpaper.clone()?;
    if let Some(file) = raw.strip_prefix("assets-wallpapers:") {
        let p = st.paths.assets_dir.join("wallpapers").join(file);
        if p.exists() { return Some(p); }
        eprintln!("glyph-desktop: wallpaper asset missing: {}", p.display());
        return None;
    }
    let p = std::path::PathBuf::from(&raw);
    if p.exists() { Some(p) } else { eprintln!("glyph-desktop: wallpaper not found: {raw}"); None }
}
