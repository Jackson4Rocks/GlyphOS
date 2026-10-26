//! Battery adapter: `/sys/class/power_supply` — real values, no guessing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BatteryState {
    pub available: bool,
    pub percent: u8,
    /// Charging / discharging / full / unknown.
    pub status: String,
}

pub fn battery_state() -> BatteryState {
    let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") else {
        return BatteryState::default()
    };
    for e in entries.flatten() {
        let p = e.path();
        let typ = std::fs::read_to_string(p.join("type")).unwrap_or_default().trim().to_string();
        if typ != "Battery" { continue }
        let cap = std::fs::read_to_string(p.join("capacity")).ok().and_then(|s| s.trim().parse::<u8>().ok());
        let status = std::fs::read_to_string(p.join("status")).unwrap_or("Unknown".into()).trim().to_string();
        return BatteryState { available: true, percent: cap.unwrap_or(0), status };
    }
    BatteryState::default()
}
