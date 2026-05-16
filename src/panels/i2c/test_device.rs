//! I2C Test Device panel.
//!
//! Surfaces the simulated test slave (sw-cor24-emulator's
//! `Add1Device`) in the I/O panel. The chip's role is "exercise
//! every I2C bus path and accrete registers as we add features";
//! today the only register is the stored byte, exposed via
//! `peek()` and `poke()` -- so the panel shows the current stored
//! byte and lets the user nudge it. As the upstream slave grows
//! registers, this panel grows controls.

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestDeviceSnapshot {
    pub address: u8,
    /// The `last` byte the slave holds. A bus read returns
    /// `(last + 1) % wrap`, so this is the value the *next* read
    /// will increment off.
    pub last_byte: u8,
}

#[derive(Properties, PartialEq)]
pub struct TestDevicePanelProps {
    pub snapshot: TestDeviceSnapshot,
    /// Invoked when the user types a new byte; main.rs translates
    /// that to `handle.with(|d| d.poke(v))`.
    pub on_poke: Callback<u8>,
}

#[function_component(TestDevicePanel)]
pub fn test_device_panel(props: &TestDevicePanelProps) -> Html {
    let s = &props.snapshot;

    let on_input = {
        let on_poke = props.on_poke.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                && let Ok(v) = input.value().parse::<u8>()
            {
                on_poke.emit(v);
            }
        })
    };

    html! {
        <div style="background:#11111b; padding:8px; border-radius:4px; \
                    border:1px solid #313244; display:flex; flex-direction:column; gap:6px;">
            <div style="display:flex; justify-content:space-between; align-items:baseline;">
                <span style="color:#cdd6f4; font-weight:600; font-size:0.85rem;">
                    {"I2C Test Device"}
                </span>
                <span style="color:#a6adc8; font-family:monospace; font-size:0.75rem;">
                    {format!("@ 0x{:02X} \u{00b7} +1 register", s.address)}
                </span>
            </div>
            <div style="display:flex; align-items:baseline; gap:8px; font-family:monospace;">
                <span style="color:#a6e3a1; font-size:1.4rem;">
                    {format!("0x{:02X}", s.last_byte)}
                </span>
                <span style="color:#bac2de; font-size:0.85rem;">{"stored"}</span>
                <span style="color:#a6adc8; font-size:0.7rem; margin-left:auto;">
                    {format!("next read: 0x{:02X}", s.last_byte.wrapping_add(1))}
                </span>
            </div>
            <input type="range"
                   min="0"
                   max="255"
                   step="1"
                   value={format!("{}", s.last_byte)}
                   oninput={on_input}
                   style="width:100%; accent-color:#a6e3a1;" />
            <div style="display:flex; justify-content:space-between; \
                        color:#6c7086; font-size:0.7rem; font-family:monospace;">
                <span>{"0x00"}</span>
                <span>{"drag to poke the stored byte"}</span>
                <span>{"0xFF"}</span>
            </div>
        </div>
    }
}
