//! TMP125 SPI device-panel.
//!
//! Mirrors the TMP101 panel shape (slider + temperature readout) but
//! reflects the TMP125's 10-bit 0.25 °C resolution instead of the
//! TMP101's 12-bit 0.0625 °C.

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tmp125Snapshot {
    /// Currently-configured temperature in °C (the value the guest
    /// would read back via the SPI exchange, quantized to 0.25 °C).
    pub temperature_c: f32,
}

#[derive(Properties, PartialEq)]
pub struct Tmp125PanelProps {
    pub snapshot: Tmp125Snapshot,
    pub on_set_temperature: Callback<f32>,
}

#[function_component(Tmp125Panel)]
pub fn tmp125_panel(props: &Tmp125PanelProps) -> Html {
    let s = &props.snapshot;

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
                <span style="color:#cdd6f4; font-weight:600; font-size:0.85rem;">{"TMP125"}</span>
                <span style="color:#a6adc8; font-family:monospace; font-size:0.75rem;">
                    {"SPI \u{00b7} 10-bit \u{00b7} 0.25 \u{00b0}C/LSB"}
                </span>
            </div>
            <div style="display:flex; align-items:baseline; gap:8px; font-family:monospace;">
                <span style="color:#fab387; font-size:1.4rem;">
                    {format!("{:.2}", s.temperature_c)}
                </span>
                <span style="color:#bac2de; font-size:0.85rem;">{"\u{00b0}C"}</span>
            </div>
            <input type="range"
                   min="-128.0"
                   max="127.75"
                   step="0.25"
                   value={format!("{}", s.temperature_c)}
                   oninput={on_input}
                   style="width:100%; accent-color:#fab387;" />
            <div style="display:flex; justify-content:space-between; \
                        color:#6c7086; font-size:0.7rem; font-family:monospace;">
                <span>{"-128 \u{00b0}C"}</span>
                <span>{"drag to change simulated temperature"}</span>
                <span>{"+128 \u{00b0}C"}</span>
            </div>
        </div>
    }
}
