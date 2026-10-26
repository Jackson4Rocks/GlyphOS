//! Brightness adapter: real `/sys/class/backlight` control with polite
//! failure reporting when the device is absent or permission is denied.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BrightnessState {
    pub available: bool,
    pub percent: u8,
    pub reason: String,
}

fn backlights() -> Vec<PathBuf> {
    std::fs::read_dir("/sys/class/backlight")
        .map(|it| it.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default()
}

fn read_int(p: &PathBuf) -> Option<u32> {
    std::fs::read_to_string(p).ok()?.trim().parse().ok()
}

pub fn brightness_state() -> BrightnessState {
    let devs = backlights();
    if devs.is_empty() {
        return BrightnessState { available: false, percent: 100, reason: "no backlight device (virtual display?)".into() };
    }
    let dev = &devs[0];
    let Some(max) = read_int(&dev.join("max_brightness")) else {
        return BrightnessState { available: false, percent: 100, reason: "cannot read max_brightness".into() };
    };
    let Some(cur) = read_int(&dev.join("brightness")) else {
        return BrightnessState { available: false, percent: 100, reason: "cannot read brightness".into() };
    };
    BrightnessState {
        available: true,
        percent: ((cur as f32 / max as f32) * 100.0).clamp(1.0, 100.0) as u8,
        reason: String::new(),
    }
}

pub fn set_brightness(percent: u8) -> Result<(), String> {
    let devs = backlights();
    if devs.is_empty() { return Err("no backlight device".into()) }
    let dev = &devs[0];
    let Some(max) = read_int(&dev.join("max_brightness")) else { return Err("read max_brightness failed".into()) };
    let val = ((percent as f32 / 100.0) * max as f32) as u32;
    let res = std::fs::write(dev.join("brightness"), format!("{val}"));
    if res.is_ok() { Ok(()) } else {
        // Fallback: brightnessctl if the user has write access via sudo rule.
        let ok = std::process::Command::new("brightnessctl")
            .arg("set").arg(&format!("{percent}%"))
            .output().map(|o| o.status.success()).unwrap_or(false);
        if ok { Ok(()) } else { Err("cannot write backlight (user not authorized; run glyph-installer polkit rule)".into()) }
    }
}
