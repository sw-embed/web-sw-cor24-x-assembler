//! TMP101 device-panel — current temperature readout + slider control.
//!
//! The slider mutates the device through a typed `I2cHandle` held in
//! `main.rs`. Range matches the TMP101's 12-bit signed temperature
//! register (~-128 °C to +127.94 °C at 0.0625 °C / LSB), so the
//! user can sweep the entire chip range; the per-resolution
//! quantization the panel shows reflects what the guest would read
//! back at the device's currently-configured resolution.

use cor24_emulator::peripherals::i2c::Tmp101Resolution;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
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
    /// Invoked when the user drags the temperature slider. The
    /// emulator side does the actual `handle.with(|d| d.set_temperature(v))`.
    pub on_set_temperature: Callback<f32>,
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

    let on_input = {
        let on_set_temperature = props.on_set_temperature.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                && let Ok(v) = input.value().parse::<f32>()
            {
                on_set_temperature.emit(v);
            }
        })
    };

    html! {
        <div style="background:#11111b; padding:8px; border-radius:4px; \
                    border:1px solid #313244; display:flex; flex-direction:column; gap:6px;">
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
            <input type="range"
                   min="-128.0"
                   max="127.9375"
                   step="0.0625"
                   value={format!("{}", s.temperature_c)}
                   oninput={on_input}
                   style="width:100%; accent-color:#89b4fa;" />
            <div style="display:flex; justify-content:space-between; \
                        color:#6c7086; font-size:0.7rem; font-family:monospace;">
                <span>{"-128 \u{00b0}C"}</span>
                <span>{"drag to change simulated temperature"}</span>
                <span>{"+128 \u{00b0}C"}</span>
            </div>
        </div>
    }
}
