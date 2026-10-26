//! GlyphOS IPC: a minimal line-based JSON protocol over a Unix socket that
//! lets `glyphctl` (CLI / keybinds) drive the shell and query state.
//!
//! Each request is one JSON object per line; responses likewise.
//! Server lives in the shell; `glyphctl` is the thin client.

use crate::Paths;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

/// Wire commands understood by the GlyphOS shell.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum Request {
    /// Toggle / show / hide the Glyph Command bar.
    Command { action: SyncAction },
    /// Toggle the Control Center.
    ControlCenter { action: SyncAction },
    /// Toggle the notification center.
    Notifications { action: SyncAction },
    /// Open an application by exec string.
    Launch { exec: String },
    /// Switch workspace.
    Workspace { index: u32 },
    /// Reload the rendered stylesheet (theme/glass hot reload).
    ReloadTheme,
    /// Local test/dev notification through the shell's own path.
    Notify { summary: String, body: String },
    /// Report basic runtime info.
    Ping,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SyncAction { Toggle, Show, Hide }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum Response {
    Ok,
    Err { message: String },
    Pong { version: String, uptime_secs: u64 },
}

/// Socket file inside the runtime dir: `shell.sock`.
pub fn socket_path(paths: &Paths) -> std::path::PathBuf {
    paths.runtime_dir.join("shell.sock")
}

/// Serialize an Ok response (helper used by the server thread).
pub fn send_ok() -> String {
    r#"{"t":"ok"}"#.into()
}

/// Serialize an Err response (helper used by the server thread).
pub fn send_err(message: &str) -> String {
    // JSON escaping for the message; minimal — messages are ASCII diagnostics.
    let esc = message.replace('\\', "\\\\").replace('"', "\\\"");
    format!(r#"{{"t":"err","message":"{esc}"}}"#)
}

/// Send one request and read one response from the shell socket.
pub fn send(paths: &Paths, req: Request) -> Result<Response, String> {
    use std::os::unix::net::UnixStream;
    let file = socket_path(paths);
    let mut s = UnixStream::connect(&file).map_err(|e| format!("connect {}: {e}", file.display()))?;
    s.set_read_timeout(Some(std::time::Duration::from_secs(3))).ok();
    let mut line = serde_json::to_string(&req).map_err(|e| e.to_string())?;
    line.push('\n');
    s.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
    let mut buf = String::new();
    let mut r = std::io::BufReader::new(s.try_clone().map_err(|e| e.to_string())?);
    r.read_line(&mut buf).map_err(|e| e.to_string())?;
    serde_json::from_str(&buf).map_err(|e| format!("bad response: {e}"))
}
