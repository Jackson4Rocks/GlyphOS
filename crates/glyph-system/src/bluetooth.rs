//! Bluetooth adapter: BlueZ via `bluetoothctl` — device discovery / pairing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BluetoothState {
    pub available: bool,
    pub powered: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtDevice {
    pub name: String,
    pub connected: bool,
    pub paired: bool,
}

fn btctl(args: &[&str]) -> Option<String> {
    std::process::Command::new("bluetoothctl").args(args).output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
}

pub fn bluetooth_state() -> BluetoothState {
    let Some(out) = btctl(&["show"]) else {
        return BluetoothState { available: false, powered: false, reason: "bluetoothctl unavailable".into() };
    };
    let text = out;
    let powered = text.lines().find_map(|l| l.trim().strip_prefix("Powered: ")).map(|s| s == "yes").unwrap_or(false);
    BluetoothState { available: true, powered, reason: String::new() }
}

pub fn devices() -> Vec<BtDevice> {
    let Some(out) = btctl(&["devices"]) else { return vec![] };
    out.lines().filter_map(|l| {
        l.trim().strip_prefix("Device ")
            .map(|r| BtDevice { name: r.split_once(' ').map(|(_, n)| n.to_string()).unwrap_or_default(), connected: false, paired: true })
    }).collect()
}

pub fn set_powered(on: bool) -> Result<(), String> {
    let ok = btctl(&["power", if on { "on" } else { "off" }]).map(|s| s.contains("succeeded")).unwrap_or(false);
    if ok { Ok(()) } else { Err("bluetooth power change failed (service running?)".into()) }
}
