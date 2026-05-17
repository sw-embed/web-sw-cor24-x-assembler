//! localStorage glue for the DS1307 "battery-backed" RTC.
//!
//! Two keys:
//!
//! - `ds1307.battery_enabled`: `"on"` / `"off"`. Drives the panel
//!   radio's initial state.
//! - `ds1307.battery`: JSON `{ "set_value": {"h", "m", "s"}, "set_at_ms" }`.
//!   The last time the user wrote into the DS1307 plus the
//!   `Date.now()` at the moment of that write. Effective time at
//!   attach is `(set_value + elapsed) % 86400` — time-of-day only,
//!   no date.
//!
//! Naming: the JSON schema is "feature-shaped" per the brief — the
//! word "battery" lives here and in the panel UI; the emulator side
//! sees only `Ds1307Device::with_initial_registers(...)`.

use serde::{Deserialize, Serialize};

const KEY_ENABLED: &str = "ds1307.battery_enabled";
const KEY_DATA: &str = "ds1307.battery";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SetValue {
    pub h: u8,
    pub m: u8,
    pub s: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Persisted {
    pub set_value: SetValue,
    pub set_at_ms: f64,
}

/// Total seconds expressed by `(h, m, s)` plus the elapsed wall-clock
/// since `set_at_ms`, taken `mod 86400` (time-of-day only).
pub fn effective_now(p: Persisted, now_ms: f64) -> SetValue {
    let elapsed_s = ((now_ms - p.set_at_ms).max(0.0) / 1000.0) as i64;
    let total = (p.set_value.h as i64) * 3600
        + (p.set_value.m as i64) * 60
        + (p.set_value.s as i64)
        + elapsed_s;
    let wrapped = total.rem_euclid(86400);
    SetValue {
        h: (wrapped / 3600) as u8,
        m: ((wrapped % 3600) / 60) as u8,
        s: (wrapped % 60) as u8,
    }
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

pub fn load_enabled() -> bool {
    storage()
        .and_then(|s| s.get_item(KEY_ENABLED).ok().flatten())
        .map(|v| v == "on")
        .unwrap_or(false)
}

pub fn save_enabled(enabled: bool) {
    if let Some(s) = storage() {
        let _ = s.set_item(KEY_ENABLED, if enabled { "on" } else { "off" });
    }
}

pub fn load() -> Option<Persisted> {
    let raw = storage()?.get_item(KEY_DATA).ok().flatten()?;
    serde_json::from_str(&raw).ok()
}

pub fn save(p: Persisted) {
    if let (Some(s), Ok(raw)) = (storage(), serde_json::to_string(&p)) {
        let _ = s.set_item(KEY_DATA, &raw);
    }
}
