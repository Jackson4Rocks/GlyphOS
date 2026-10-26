//! Audio adapter. Reads Pulse/PipeWire-compatible sinks via the `pulse` crate
//! API that ships with `pipewire-pulse`. For now: `wpctl` JSON-free CLI state
//! with a typed hold (delta events) so the shell reacts quickly and honestly.

use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AudioState {
    /// Default sink name.
    pub sink: Option<String>,
    /// Volume as percentage 0..=100.
    pub volume: u8,
    pub muted: bool,
    /// Default source name (microphone).
    pub source: Option<String>,
    pub mic_muted: bool,
    pub available: bool,
}

fn wpctl(args: &[&str]) -> Option<std::process::Output> {
    Command::new("wpctl").args(args).output().ok()
}

pub fn audio_state() -> AudioState {
    let mut state = AudioState { available: false, ..Default::default() };
    let Some(out) = wpctl(&["status"]) else { return state };
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    state.available = wpctl(&["get-default-[@DEFAULT_AUDIO_SINK@]"]).map(|o| o.status.success()).unwrap_or(false);
    let mut section = "";
    for line in text.lines() {
        if line.contains("Sinks:") { section = "sink"; continue }
        if line.contains("Sources:") { section = "src"; continue }
        if line.contains(" Sink Endpoint") || line.contains(" Source Endpoint") { section = ""; }
        if section == "sink" && line.contains("*") && line.contains("[vol:") && state.sink.is_none() {
            state.sink = device_name(line);
            state.volume = parse_volume(line);
            state.muted = line.contains("MUTED");
        }
        if section == "src" && line.contains("*") && line.contains("[vol:") && state.source.is_none() {
            state.source = device_name(line);
            state.mic_muted = line.contains("MUTED");
        }
    }
    state
}

fn device_name(line: &str) -> Option<String> {
    let start = line.find(|c: char| c.is_uppercase())?;
    let tail = &line[start..];
    tail.split_whitespace().next().map(|s| s.trim_matches(|c| c == ',' || c == '(').to_string())
}

fn parse_volume(line: &str) -> u8 {
    let f = match line.find("[vol:") {
        Some(i) => &line[i + 5..],
        None => return 0,
    };
    f.chars().filter(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>().split('.').next()
        .and_then(|n| n.parse::<u8>().ok()).unwrap_or(0)
}

/// Set the default sink volume to a percentage.
pub fn set_volume(pct: u8) -> Result<(), String> {
    let v = (pct.clamp(0, 100)) as f32 / 100.0 * 65536.0 / 65536.0 * 100.0;
    let ok = wpctl(&["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{v:.2}%")])
        .map(|o| o.status.success()).unwrap_or(false);
    if ok { Ok(()) } else { Err("wpctl set-volume failed (is WirePlumber running?)".into()) }
}

pub fn set_muted(on: bool) -> Result<(), String> {
    let ok = wpctl(&["set-mute", "@DEFAULT_AUDIO_SINK@", if on { "1" } else { "0" }])
        .map(|o| o.status.success()).unwrap_or(false);
    if ok { Ok(()) } else { Err("wpctl set-mute failed".into()) }
}

pub fn set_mic_muted(on: bool) -> Result<(), String> {
    let ok = wpctl(&["set-mute", "@DEFAULT_AUDIO_SOURCE@", if on { "1" } else { "0" }])
        .map(|o| o.status.success()).unwrap_or(false);
    if ok { Ok(()) } else { Err("wpctl set-mute source failed".into()) }
}
