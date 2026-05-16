//! TMP101 device-panel — read-only view in this step.
//!
//! Step 005 wires the framework up to the emulator's I2cHandle path
//! and displays the live device state. The interactive slider/drag
//! control that lets the user set the simulated °C is step 006.

use cor24_emulator::peripherals::i2c::Tmp101Resolution;
use yew::prelude::*;

/// A per-tick snapshot of the TMP101 the run loop polls out of
/// `I2cHandle<Tmp101Device>` and feeds back into the panel as a
/// prop. Carrying a `PartialEq` snapshot — rather than a handle —
/// lets Yew skip re-renders when the device hasn't changed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tmp101Snapshot {
    pub address: u8,
    /// Currently-configured temperature in °C (the value the guest
    /// would read back at the active resolution).
    pub temperature_c: f32,
    /// Configuration register (low bits select resolution).
    pub config: u8,
    pub resolution: Tmp101Resolution,
}

#[derive(Properties, PartialEq)]
pub struct Tmp101PanelProps {
    pub snapshot: Tmp101Snapshot,
}

#[function_component(Tmp101Panel)]
pub fn tmp101_panel(props: &Tmp101PanelProps) -> Html {
    let s = &props.snapshot;
    let res_label = match s.resolution {
        Tmp101Resolution::Bits9 => "9-bit",
        Tmp101Resolution::Bits10 => "10-bit",
        Tmp101Resolution::Bits11 => "11-bit",
        Tmp101Resolution::Bits12 => "12-bit",
    };

    html! {
        <div style="background:#11111b; padding:8px; border-radius:4px; \
                    border:1px solid #313244; display:flex; flex-direction:column; gap:4px;">
            <div style="display:flex; justify-content:space-between; align-items:baseline;">
                <span style="color:#cdd6f4; font-weight:600; font-size:0.85rem;">{"TMP101"}</span>
                <span style="color:#a6adc8; font-family:monospace; font-size:0.75rem;">
                    {format!("@ 0x{:02X} \u{00b7} {}", s.address, res_label)}
                </span>
            </div>
            <div style="display:flex; align-items:baseline; gap:8px; font-family:monospace;">
                <span style="color:#89b4fa; font-size:1.4rem;">
                    {format!("{:.2}", s.temperature_c)}
                </span>
                <span style="color:#bac2de; font-size:0.85rem;">{"\u{00b0}C"}</span>
                <span style="color:#a6adc8; font-size:0.7rem; margin-left:auto;">
                    {format!("config 0x{:02X}", s.config)}
                </span>
            </div>
            <div style="color:#6c7086; font-size:0.7rem;">
                {"step 006 will add a slider for live \u{00b0}C control"}
            </div>
        </div>
    }
}
