//! SPI Echo (test) device panel.
//!
//! Mirrors the I2C test-device card for the SPI bus: shows the
//! slave's buffered MISO byte. The buffered byte is what the slave
//! drives on the next 8-clock exchange (the bus's one-byte echo
//! delay; subsequent exchanges latch the just-clocked MOSI back
//! into the buffer). Read-only — the demo overwrites the buffer
//! on every exchange, so a user-facing poke slider would just
//! snap back within a frame.

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
}

#[function_component(EchoPanel)]
pub fn echo_panel(props: &EchoPanelProps) -> Html {
    let s = &props.snapshot;

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
        </div>
    }
}
