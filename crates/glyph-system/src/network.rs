//! NetworkManager adapter over `nmcli`. Real state in/out.

use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NetworkState {
    pub wifi_enabled: bool,
    pub ethernet_connected: bool,
    pub ssid: Option<String>,
    pub strength: u8,
    pub state: String, // disconnected | connecting | connected
}

fn run_nmcli(args: &[&str]) -> String {
    Command::new("nmcli")
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// Pull a structured snapshot of connectivity state.
pub fn network_state() -> NetworkState {
    let status = run_nmcli(&["-t", "-f", "STATE,WIFI,CONNECTIVITY", "general"]);
    let mut state = NetworkState { wifi_enabled: false, ethernet_connected: false, ssid: None, strength: 0, state: "disconnected".into() };
    let parts: Vec<&str> = status.trim().split(':').collect();
    if parts.len() >= 3 {
        state.wifi_enabled = parts[1] == "enabled";
        state.state = parts[0].to_lowercase();
        state.ethernet_connected = state.state == "connected" && status.contains("LIMITED") == false;
    }
    // Active wifi SSID + strength.
    let dev = run_nmcli(&["-t", "-f", "DEVICE,TYPE", "device", "status"]);
    let _ = dev;
    let active = run_nmcli(&["-t", "-f", "ACTIVE,SIGNAL,IN-USE", "device", "wifi", "list"]);
    for line in active.lines() {
        let mut f = line.split(':');
        let active = f.next().unwrap_or("no") == "yes";
        let signal: u8 = f.next().unwrap_or("0").parse().unwrap_or(0);
        let in_use = f.next().unwrap_or("").contains('*');
        if active && in_use {
            state.ssid = Some(line.split(':').last().unwrap_or("").to_string());
            state.strength = signal;
            state.state = "connected".into();
        }
    }
    state
}

/// Visible Wi-Fi networks sorted by strength.
pub fn wifi_networks() -> Vec<WifiNetwork> {
    let out = run_nmcli(&["-t", "-f", "IN-USE,SSID,SIGNAL,SECURITY", "device", "wifi", "list"]);
    let mut networks = out.lines().filter_map(|l| {
        let mut f = l.split(':');
        let in_use = f.next()? == "*";
        let ssid = f.next()?.to_string();
        if ssid.is_empty() { return None; }
        let signal: u8 = f.next()?.parse().ok()?;
        let security = f.next().unwrap_or("").to_string();
        Some(WifiNetwork { in_use, ssid, signal, security })
    }).collect::<Vec<_>>();
    networks.sort_by(|a, b| b.signal.cmp(&a.signal));
    networks.dedup_by(|a, b| a.ssid == b.ssid);
    networks
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub in_use: bool,
    pub ssid: String,
    pub signal: u8,
    pub security: String,
}

/// Toggle Wi-Fi radio.
pub fn set_wifi_enabled(on: bool) -> Result<(), String> {
    let res = Command::new("nmcli").args(["radio", "wifi", if on { "on" } else { "off" }]).output();
    if res.map(Ok::<_, ()>).is_ok() { Ok(()) } else { Err("nmcli radio failed".into()) }
}

/// Connect to a Wi-Fi network, asking the user to supply a key (prompt).
pub fn connect_wifi(ssid: &str, password: Option<&str>) -> Result<(), String> {
    let mut cmd = Command::new("nmcli");
    cmd.arg("dev").arg("wifi").arg("connect").arg(ssid);
    if let Some(p) = password {
        cmd.arg("password").arg(p);
    }
    let out = cmd.output().map_err(|e| e.to_string())?;
    if out.status.success() { Ok(()) } else {
        Err(String::from_utf8_lossy(&out.stderr).into_owned())
    }
}
