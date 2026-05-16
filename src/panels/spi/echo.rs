//! SPI Echo (test) device panel.
//!
//! Mirrors the I2C test-device card for the SPI bus: shows the
//! slave's buffered MISO byte and exposes a "poke" control. The
//! buffered byte is what the slave drives on the next 8-clock
//! exchange (the bus's one-byte echo delay; subsequent exchanges
//! latch the just-clocked MOSI back into the buffer).

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EchoSnapshot {
    /// The byte the slave will drive on MISO during the next
    /// 8-clock exchange.
    pub buffer: u8,
}

#[derive(Properties, PartialEq)]
pub struct EchoPanelProps {
    pub snapshot: EchoSnapshot,
    /// Invoked when the user drags the buffer slider; `main.rs`
    /// translates to `handle.with(|d| d.poke(v))`.
    pub on_poke: Callback<u8>,
}

#[function_component(EchoPanel)]
pub fn echo_panel(props: &EchoPanelProps) -> Html {
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
                    {"SPI Test Device"}
                </span>
                <span style="color:#a6adc8; font-family:monospace; font-size:0.75rem;">
                    {"echo \u{00b7} one-byte buffer"}
                </span>
            </div>
            <div style="display:flex; align-items:baseline; gap:8px; font-family:monospace;">
                <span style="color:#fab387; font-size:1.4rem;">
                    {format!("0x{:02X}", s.buffer)}
                </span>
                <span style="color:#bac2de; font-size:0.85rem;">{"buffer"}</span>
                <span style="color:#a6adc8; font-size:0.7rem; margin-left:auto;">
                    {"next exchange drives this on MISO"}
                </span>
            </div>
            <input type="range"
                   min="0"
                   max="255"
                   step="1"
                   value={format!("{}", s.buffer)}
                   oninput={on_input}
                   style="width:100%; accent-color:#fab387;" />
            <div style="display:flex; justify-content:space-between; \
                        color:#6c7086; font-size:0.7rem; font-family:monospace;">
                <span>{"0x00"}</span>
                <span>{"drag to poke the buffered byte"}</span>
                <span>{"0xFF"}</span>
            </div>
        </div>
    }
}
