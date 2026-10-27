//! System event polling: small periodic state snapshots the shell polls so
//! panel indicators remain truthful without needing DBus subscriptions yet.

use crate::battery::BatteryState;
use crate::bluetooth::BluetoothState;
use crate::brightness::BrightnessState;
use crate::media::MediaState;
use crate::network::NetworkState;
use crate::pulse::AudioState;

/// A one-shot snapshot of system state used to drive the panel/Control Center.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SystemSnapshot {
    pub network: NetworkState,
    pub audio: AudioState,
    pub battery: BatteryState,
    pub brightness: BrightnessState,
    pub bluetooth: BluetoothState,
    pub media: MediaState,
}

impl SystemSnapshot {
    /// Poll all adapters (background thread only; never the UI thread).
    pub fn snapshot() -> Self {
        Self {
            network: crate::network::network_state(),
            audio: crate::pulse::audio_state(),
            battery: crate::battery::battery_state(),
            brightness: crate::brightness::brightness_state(),
            bluetooth: crate::bluetooth::bluetooth_state(),
            media: crate::media::media_state(),
        }
    }
}

/// Cached snapshot so UI ticks never block on subprocess IPC.
pub fn snap_cache() -> &'static std::sync::RwLock<SystemSnapshot> {
    static CACHE: std::sync::OnceLock<std::sync::RwLock<SystemSnapshot>> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::RwLock::new(SystemSnapshot::default()))
}

/// Launch the background refresh loop (one shared thread, real adapters).
pub fn start_polling() {
    std::thread::Builder::new().name("glyph-sys-poll".into()).spawn(|| loop {
        let snap = SystemSnapshot::snapshot();
        if let Ok(mut w) = snap_cache().write() { *w = snap; }
        std::thread::sleep(std::time::Duration::from_millis(700));
    }).ok();
}

/// Focused-time health: CPU load and memory from `/proc` (real data, no deps).
pub fn system_load() -> (f32, u64, u64) {
    use std::fs;
    let load1 = fs::read_to_string("/proc/loadavg").ok()
        .and_then(|s| s.split(' ').next().and_then(|v| v.parse::<f32>().ok())).unwrap_or(0.0);
    let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut mem_total = 0u64; let mut mem_avail = 0u64;
    for line in meminfo.lines() {
        if let Some(v) = line.strip_prefix("MemTotal:") { mem_total = v.trim().trim_end_matches(" kB").parse().unwrap_or(0) }
        if let Some(v) = line.strip_prefix("MemAvailable:") { mem_avail = v.trim().trim_end_matches(" kB").parse().unwrap_or(0) }
    }
    (load1, mem_total, mem_available_kb(mem_total, mem_avail))
}

fn mem_available_kb(total: u64, avail: u64) -> u64 {
    if avail > 0 { avail } else { total }
}
