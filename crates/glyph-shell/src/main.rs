//! GlyphOS Shell — the integrated desktop shell.
//!
//! One GTK4 process that creates every shell surface via the wlr-layer-shell
//! protocol: top panel, dock, Glyph Command bar, Control Center, and the
//! notification strip. Runs on any Wayland compositor with layer-shell
//! support (glyphwm, Hyprland, sway, ...).

mod apps;
mod compositor;
mod dispatch;
mod ipc_server;
mod notify_bus;
mod state;
mod ui;

use glyph_core::{ipc, Paths};
use gtk::prelude::*;
use std::sync::Arc;
use state::{ShellSignal, ShellState};

fn main() {
    // Single instance: if a shell answers on the IPC socket, do not start.
    let paths = Paths::discover();
    paths.ensure();
    if let Ok(ipc::Response::Pong { .. }) = ipc::send(&paths, ipc::Request::Ping) {
        eprintln!("glyph-shell already running (IPC socket responsive).");
        std::process::exit(3);
    }

    gtk::init().expect("glyph-shell requires a usable display connection");
    println!("glyph-shell v{} · session {}", glyph_core::VERSION, std::env::var("WAYLAND_DISPLAY").unwrap_or_default());

    let config = glyph_core::Config::load_or_default(&paths);
    install_fonts(&paths);
    install_css(&config);
    let config = std::sync::Arc::new(std::sync::RwLock::new(config));

    let app = gtk::Application::builder()
        .application_id("os.glyph.Shell")
        .build();

    app.connect_activate(move |app| {
        // IPC channel: background thread → GTK main loop (std mpsc + pump).
        let (tx, rx) = std::sync::mpsc::channel::<ShellSignal>();
        // Notification daemon bridge: UI closes are echoed onto the bus.
        let (notif_close_tx, notif_close_rx) = std::sync::mpsc::channel::<(u32, u32)>();
        let notifs: Arc<std::sync::Mutex<std::collections::VecDeque<crate::state::Notif>>> =
            Arc::new(std::sync::Mutex::new(Default::default()));
        let st: Arc<ShellState> = Arc::new(ShellState {
            paths: paths.clone(),
            config: config.clone(),
            tx: tx.clone(),
            snap: std::sync::Mutex::new(glyph_system::events::SystemSnapshot::snapshot()),
            notifs: notifs.clone(),
            notif_close_tx: notif_close_tx.clone(),
        });

        // Real D-Bus notification daemon (org.freedesktop.Notifications).
        if let Err(e) = notify_bus::spawn(tx.clone(), notif_close_rx) {
            eprintln!("glyph-shell: notification daemon failed to spawn: {e}");
        }

        // Build every shell surface, desktop first (it sits below the rest).
        ui::desktop::build(&st, app);
        ui::panel::build(&st, app);
        ui::osd::build(&st, app);
        ui::dock::build(&st, app);
        ui::command::build(&st, app);
        ui::control::build(&st, app);
        ui::notify::build(&st, app);
        ui::quick::build(&st, app);

        // Background pollers: real system + compositor state (never on UI thread).
        glyph_system::events::start_polling();
        crate::compositor::start_polling();

        // Dispatch injected signals into registered windows.
        {
            let st_dispatch = Arc::clone(&st);
            let rx = rx;
            // Idle pump: drains the std mpsc every frame-ish; cheap when empty.
            gtk::glib::timeout_add_local(std::time::Duration::from_millis(24), move || {
                let mut more = true;
                while more {
                    match rx.try_recv() {
                        Ok(sig) => dispatch::handle(&st_dispatch, sig),
                        Err(std::sync::mpsc::TryRecvError::Empty) => more = false,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            more = false;
                        }
                    }
                }
                gtk::glib::ControlFlow::Continue
            });
        }

        // Real IPC server honoring glyphctl / keybinds.
        if let Err(e) = ipc_server::spawn(tx) {
            eprintln!("glyph-shell: IPC server failed: {e}");
        }
    });

    app.run();
}

/// Copy bundled fonts into the user font dir so fontconfig picks them up; the
/// very first launch finds Doto + Space Mono without any system package.
fn install_fonts(paths: &Paths) {
    let dest = dirs_home().join(".local/share/fonts/glyphos");
    let _ = std::fs::create_dir_all(&dest);
    for d in ["doto", "spacemono"] {
        let dir = paths.assets_dir.join("fonts").join(d);
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for e in entries.flatten() {
                let target = dest.join(e.file_name());
                if !target.exists() {
                    std::fs::copy(e.path(), &target).ok();
                }
            }
        }
    }
    lazily_refresh_font_cache();
}

fn dirs_home() -> std::path::PathBuf {
    std::env::var_os("HOME").map(std::path::PathBuf::from).unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
}

fn lazily_refresh_font_cache() {
    std::thread::spawn(|| {
        std::process::Command::new("fc-cache").args(["-f", "/home/leon/.local/share/fonts/glyphos"]).output().ok();
    });
}

/// Register the token-derived CSS globally; the shell and every first-party
/// app read this exact stylesheet from glyph-core.
fn install_css(config: &glyph_core::Config) {
    let css = glyph_core::tokens::stylesheet(&config.theme);
    println!("glyph-shell: stylesheet → {}", css.lines().next().unwrap_or(""));
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&css);
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("glyph-shell requires a display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );
}

