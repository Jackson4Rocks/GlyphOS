//! Shell IPC server: accepts `glyphctl` / keybind connections on a Unix
//! socket and injects signals into the GTK main loop via a std mpsc channel.

use crate::state::{Overlay, ShellSignal, SignalTx, ZAction};
use glyph_core::{ipc, Paths};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::time::Instant;

/// Uptime marker used in `Ping` responses.
static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

pub fn spawn(tx: SignalTx) -> Result<(), String> {
    let paths = Paths::discover();
    let file = ipc::socket_path(&paths);
    let _ = std::fs::remove_file(&file);
    let listener = UnixListener::bind(&file).map_err(|e| format!("bind {}: {e}", file.display()))?;
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).ok();
    let _ = START.set(Instant::now());
    std::thread::Builder::new().name("glyph-ipc".into()).spawn(move || {
        for stream in listener.incoming().flatten() {
            handle_stream(stream, &tx);
        }
    }).map_err(|e| e.to_string())?;
    Ok(())
}

fn to_z(a: ipc::SyncAction) -> ZAction {
    match a {
        ipc::SyncAction::Toggle => ZAction::Toggle,
        ipc::SyncAction::Show => ZAction::Show,
        ipc::SyncAction::Hide => ZAction::Hide,
    }
}

fn handle_stream(stream: UnixStream, tx: &SignalTx) {
    let reader = BufReader::new(stream.try_clone().expect("stream clone"));
    let mut out = stream;
    for line in reader.lines() {
        let Ok(line) = line else { break };
        if line.is_empty() { continue }
        let reply = match serde_json::from_str::<ipc::Request>(&line) {
            Ok(req) => handle_one(req, tx),
            Err(e) => ipc::send_err(&e.to_string()),
        };
        if writeln!(out, "{reply}").is_err() { break }
        let _ = out.flush();
    }
}

fn handle_one(req: ipc::Request, tx: &SignalTx) -> String {
    match req {
        ipc::Request::Ping => {
            let up = START.get().map(|t| t.elapsed().as_secs()).unwrap_or(0);
            serde_json::to_string(&ipc::Response::Pong {
                version: glyph_core::VERSION.into(),
                uptime_secs: up,
            }).unwrap()
        }
        ipc::Request::Command { action } => { tx.send(ShellSignal::Window(Overlay::Command, to_z(action))).ok(); ipc::send_ok() }
        ipc::Request::ControlCenter { action } => { tx.send(ShellSignal::Window(Overlay::Control, to_z(action))).ok(); ipc::send_ok() }
        ipc::Request::Notifications { action } => { tx.send(ShellSignal::Window(Overlay::Notify, to_z(action))).ok(); ipc::send_ok() }
        ipc::Request::Launch { exec } => { tx.send(ShellSignal::Launch(exec)).ok(); ipc::send_ok() }
        ipc::Request::Workspace { index } => { tx.send(ShellSignal::Workspace(index)).ok(); ipc::send_ok() }
        ipc::Request::ReloadTheme => { tx.send(ShellSignal::ReloadTheme).ok(); ipc::send_ok() }
        ipc::Request::Quick => { tx.send(ShellSignal::Window(Overlay::Quick, ZAction::Toggle)).ok(); ipc::send_ok() }
        ipc::Request::Notify { summary, body } => {
            tx.send(ShellSignal::Notification(crate::state::Notif {
                id: crate::state::next_local_notif_id(),
                app: "GlyphOS".into(),
                summary,
                body,
                urgency: 0,
            })).ok();
            ipc::send_ok()
        }
    }
}
