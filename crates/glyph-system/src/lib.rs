//! GlyphOS system adapters.
//!
//! Thin, honest service layer over Linux system state used by the shell and
//! first-party apps. Each backend is a small module with a real data source:
//! NetworkManager (`nmcli`), PipeWire/WirePlumber (`pw-pulse` / `wpctl`),
//! BlueZ (`bluetoothctl`), backlight (`/sys/class/backlight`), battery
//! (`/sys/class/power_supply`).

pub mod battery;
pub mod bluetooth;
pub mod brightness;
pub mod events;
pub mod media;
pub mod network;
pub mod pulse;
