//! Container for the SPI device-panel stack.
//!
//! Renders an SPI bus-state header (selected, last MOSI/MISO bytes,
//! bytes-exchanged counter) plus one card per attached device.

use yew::prelude::*;

use super::tmp125::{Tmp125Panel, Tmp125Snapshot};

/// Subset of `cor24_emulator::cpu::spi_bus::SpiBusState` that the
/// container needs for the header. Mirrors the I2C `BusSnapshot`
/// shape so Yew can short-circuit re-renders on equality.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SpiBusSnapshot {
    /// `true` when SELN is low (a device is currently selected).
    pub selected: bool,
    pub last_mosi: Option<u8>,
    pub last_miso: Option<u8>,
    pub bytes_exchanged: u32,
    /// Whether any SPI device is attached. Mirrors the I2C
    /// `attached` count semantically; SPI is single-slave today so
    /// this is a bool, not a count.
    pub attached: bool,
}

#[derive(Properties, PartialEq)]
pub struct SpiPanelProps {
    pub bus: SpiBusSnapshot,
    #[prop_or_default]
    pub tmp125: Option<Tmp125Snapshot>,
    pub on_set_tmp125_temperature: Callback<f32>,
}

#[function_component(SpiPanel)]
pub fn spi_panel(props: &SpiPanelProps) -> Html {
    if props.tmp125.is_none() && !props.bus.attached {
        return html! {};
    }

    let byte_str = |b: Option<u8>| -> String {
        match b {
            Some(v) => format!("0x{v:02X}"),
            None => "--".to_string(),
        }
    };

    html! {
        <div style="display:flex; flex-direction:column; gap:6px;">
            <div style="color:#bac2de; font-size:0.8rem;">{"SPI bus"}</div>
            <div style="background:#11111b; padding:6px 8px; border-radius:4px; \
                        font-family:monospace; font-size:12px; color:#bac2de; \
                        display:flex; gap:12px; flex-wrap:wrap;">
                <span>
                    {"SELN: "}
                    <span style="color:#89b4fa;">{
                        if props.bus.selected { "selected" } else { "deselected" }
                    }</span>
                </span>
                <span>
                    {"last MOSI: "}
                    <span style="color:#a6e3a1;">{byte_str(props.bus.last_mosi)}</span>
                </span>
                <span>
                    {"last MISO: "}
                    <span style="color:#fab387;">{byte_str(props.bus.last_miso)}</span>
                </span>
                <span>
                    {"bytes: "}
                    <span style="color:#cba6f7;">{props.bus.bytes_exchanged}</span>
                </span>
            </div>

            if let Some(snap) = props.tmp125 {
                <Tmp125Panel snapshot={snap}
                             on_set_temperature={props.on_set_tmp125_temperature.clone()} />
            }
        </div>
    }
}
