//! glyphctl — the GlyphOS control CLI. Talks to the running shell over IPC.

use glyph_core::{ipc, ipc::{Request, Response}, Paths};
use std::process::exit;

fn usage() {
    eprintln!("glyphctl {} — control the GlyphOS shell", glyph_core::VERSION);
    eprintln!("  command            toggle Glyph Command (Super+Space)");
    eprintln!("  control            toggle Control Center");
    eprintln!("  notify             toggle notifications");
    eprintln!("  notif-send SUMMARY [BODY]  local notification through the shell");
    eprintln!("  launch <exec>      launch an application");
    eprintln!("  workspace <1|2|3>  switch workspace");
    eprintln!("  quick              toggle the clock flyout");
    eprintln!("  osd <volume|brightness> <up|down|delta>  real change + OSD");
    eprintln!("  reload-theme       apply current glyphos.toml theme");
    eprintln!("  ping               check the running shell");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        usage();
        exit(2);
    };
    let req = match cmd.as_str() {
        "command" => Request::Command { action: ipc::SyncAction::Toggle },
        "control" => Request::ControlCenter { action: ipc::SyncAction::Toggle },
        "notify" => Request::Notifications { action: ipc::SyncAction::Toggle },
        "notif-send" => {
            let summary = args.get(1).cloned().unwrap_or_else(|| { eprintln!("notif-send SUMMARY [BODY]"); exit(2) });
            let body = args.get(2..).map(|rest| rest.join(" ")).unwrap_or_default();
            Request::Notify { summary, body }
        }
        "launch" => {
            let exec = args.get(1).unwrap_or_else(|| { eprintln!("launch requires an exec"); exit(2) });
            Request::Launch { exec: exec.clone() }
        }
        "workspace" => {
            let n: u32 = args.get(1).and_then(|a| a.parse().ok()).unwrap_or_else(|| { eprintln!("workspace <1..>"); exit(2) });
            Request::Workspace { index: n }
        }
        "reload-theme" | "theme" => Request::ReloadTheme,
        "quick" => Request::Quick,
        "osd" => {
            let kind = args.get(1).cloned().unwrap_or_else(|| { eprintln!("osd <volume|brightness> <up|down|delta>"); exit(2) });
            let dir = match args.get(2).map(|s| s.as_str()).unwrap_or("up") {
                "up" | "+" => 5,
                "down" | "-" => -5,
                p => p.parse::<i32>().unwrap_or(5),
            };
            Request::Osd { kind, dir }
        }
        "ping" => Request::Ping,
        "help" | "--help" | "-h" => { usage(); return }
        other => {
            eprintln!("unknown command: {other}");
            usage();
            exit(2);
        }
    };

    let paths = Paths::discover();
    match ipc::send(&paths, req) {
        Ok(Response::Pong { version, uptime_secs }) => {
            println!("glyph-shell {version} · uptime {uptime_secs}s");
        }
        Ok(Response::Ok) => println!("ok"),
        Ok(Response::Err { message }) => {
            eprintln!("error: {message}");
            exit(1);
        }
        Err(e) => {
            eprintln!("glyphctl: {e}");
            eprintln!("hint: is glyph-shell running?");
            exit(1);
        }
    }
}
