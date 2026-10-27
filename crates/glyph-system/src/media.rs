//! MPRIS media adapter: honest read/control via `playerctl` (real players only).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaState {
    pub available: bool,
    pub playing: bool,
    pub player: String,
    pub title: String,
    pub artist: String,
}

fn ctl(args: &[&str]) -> Option<String> {
    std::process::Command::new("playerctl").args(args).output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
}

pub fn media_state() -> MediaState {
    let Some(out) = ctl(&["-s", "metadata", "--format", "{status}|{playerName}|{xesam:title}|{xesam:artist}"]) else {
        return MediaState::default();
    };
    let mut it = out.trim_end_matches('\n').splitn(4, '|');
    let status = it.next().unwrap_or("");
    let player = it.next().unwrap_or("").to_string();
    let title = it.next().unwrap_or("").to_string();
    let artist = it.next().unwrap_or("").to_string();
    if player.is_empty() && title.is_empty() {
        return MediaState::default();
    }
    MediaState {
        available: true,
        playing: status.eq_ignore_ascii_case("playing"),
        player, title, artist,
    }
}

pub fn media_cmd(cmd: &str) -> Result<(), String> {
    ctl(&[cmd]).ok_or_else(|| "playerctl failed (no player?)".to_string())?;
    Ok(())
}
