//! I2C Test Device panel.
//!
//! Surfaces the simulated test slave (sw-cor24-emulator's
//! `Add1Device`) in the I/O panel. The chip's role is "exercise
//! every I2C bus path and accrete registers as we add features";
//! today the only register is the stored byte, exposed via
//! `peek()` and `poke()`. Read-only readout — the demo writes to
//! the slave on every loop iteration, so a user-facing poke
//! slider would just snap back within a frame. As the upstream
//! slave grows registers that *aren't* bus-driven, this panel can
//! grow controls for them.

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
}

#[function_component(TestDevicePanel)]
pub fn test_device_panel(props: &TestDevicePanelProps) -> Html {
    let s = &props.snapshot;

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
        </div>
    }
}
