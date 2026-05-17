//! DS1307 I2C RTC panel.
//!
//! Shows the chip's HH:MM:SS readout plus a "Battery off / Battery on"
//! toggle that controls whether the time the user wrote into the chip
//! survives a page reload (see brief
//! `tools/briefs/dwxas-battery-backed-rtc.md`). The persistence
//! contract is **time-of-day only** — `% 86400` — date fields are
//! intentionally out of scope.
//!
//! Naming separation (load-bearing): emulator-side types stay
//! device-shaped (`Ds1307Device::with_initial_registers`); the
//! "battery" metaphor lives entirely in this panel + the
//! `ds1307.battery*` localStorage keys.

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

/// What the UI shows for the RTC. Read out of the device through
/// `Ds1307HandleExt` once per tick (or synthesised pre-Run).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ds1307Snapshot {
    pub address: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Properties, PartialEq)]
pub struct Ds1307PanelProps {
    pub snapshot: Ds1307Snapshot,
    /// `true` when the "Battery on" radio is selected. Drives whether
    /// the next Run seeds the device from localStorage.
    pub battery_enabled: bool,
    /// Toggle handler: emits the new `battery_enabled` value.
    pub on_toggle_battery: Callback<bool>,
    /// Click handler for the "Set to system time" button. Reads the
    /// host's `Date.now()`, decomposes to local HH:MM:SS, sets the
    /// device (if attached) and the persisted state (if battery on).
    pub on_set_system_time: Callback<MouseEvent>,
}

#[function_component(Ds1307Panel)]
pub fn ds1307_panel(props: &Ds1307PanelProps) -> Html {
    let s = &props.snapshot;

    let on_change = {
        let on_toggle = props.on_toggle_battery.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                on_toggle.emit(input.value() == "on");
            }
        })
    };

    let radio = |label: &'static str, value: &'static str, selected: bool| -> Html {
        html! {
            <label style="display:inline-flex; align-items:center; gap:4px; \
                          cursor:pointer; color:#bac2de; font-size:0.8rem;">
                <input type="radio"
                       name="ds1307_battery"
                       value={value}
                       checked={selected}
                       onchange={on_change.clone()} />
                {label}
            </label>
        }
    };

    html! {
        <div style="background:#11111b; padding:8px; border-radius:4px; \
                    border:1px solid #313244; display:flex; flex-direction:column; gap:6px;">
            <div style="display:flex; justify-content:space-between; align-items:baseline;">
                <span style="color:#cdd6f4; font-weight:600; font-size:0.85rem;">
                    {"DS1307 RTC"}
                </span>
                <span style="color:#a6adc8; font-family:monospace; font-size:0.75rem;">
                    {format!("@ 0x{:02X} \u{00b7} BCD", s.address)}
                </span>
            </div>
            <div style="display:flex; align-items:baseline; gap:8px; font-family:monospace;">
                <span style="color:#89b4fa; font-size:1.4rem;">
                    {format!("{:02}:{:02}:{:02}", s.hour, s.minute, s.second)}
                </span>
                <span style="color:#bac2de; font-size:0.85rem;">{"HH:MM:SS"}</span>
            </div>
            <div style="display:flex; gap:12px; align-items:center; flex-wrap:wrap;">
                { radio("Battery off", "off", !props.battery_enabled) }
                { radio("Battery on", "on", props.battery_enabled) }
                <button onclick={props.on_set_system_time.clone()}
                        style="padding:3px 10px; background:#89b4fa; color:#1e1e2e; \
                               border:none; border-radius:4px; font-size:0.75rem; \
                               font-weight:600; cursor:pointer;">
                    {"Set to system time"}
                </button>
            </div>
            <div style="color:#6c7086; font-size:0.7rem;">
                if props.battery_enabled {
                    {"persists across reloads (time-of-day only, % 86400)"}
                } else {
                    {"boots at 00:00:00 every Run unless you 'Set to system time'"}
                }
            </div>
        </div>
    }
}
