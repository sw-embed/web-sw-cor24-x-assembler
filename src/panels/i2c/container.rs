//! Container for the I2C device-panel stack.
//!
//! Renders a bus-state header (phase, last byte, transactions count)
//! plus one child card per attached device. Devices declare their own
//! snapshot type and component; the container only wires them into the
//! layout so new device modules don't have to touch any glue here.

use yew::prelude::*;

use super::rtc::{Ds1307Panel, Ds1307Snapshot};
use super::test_device::{TestDevicePanel, TestDeviceSnapshot};
use super::tmp101::{Tmp101Panel, Tmp101Snapshot};

/// Subset of `cor24_emulator::cpu::i2c_bus::I2cBusState` that the
/// container needs to render the bus header. Kept as its own struct so
/// the panel doesn't have to depend on the full I2cBusState type
/// (which doesn't impl PartialEq because of the address-routing table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BusSnapshot {
    /// Has the bus seen any START / STOP since reset?
    pub idle: bool,
    /// Most recently transferred data byte (write or address byte),
    /// `None` until the first byte completes.
    pub last_byte: Option<u8>,
    /// Sticky 7-bit address most recently addressed.
    pub last_addressed: Option<u8>,
    /// Count of START conditions seen since reset.
    pub transactions: u32,
    /// Count of devices attached to the bus.
    pub attached: usize,
}

impl Default for BusSnapshot {
    fn default() -> Self {
        Self {
            idle: true,
            last_byte: None,
            last_addressed: None,
            transactions: 0,
            attached: 0,
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct I2cPanelProps {
    pub bus: BusSnapshot,
    /// `None` if the TMP101 isn't attached this run.
    #[prop_or_default]
    pub tmp101: Option<Tmp101Snapshot>,
    /// `None` if the test device isn't attached this run.
    #[prop_or_default]
    pub test_device: Option<TestDeviceSnapshot>,
    /// `None` if the DS1307 RTC isn't attached this run.
    #[prop_or_default]
    pub ds1307: Option<Ds1307Snapshot>,
    /// Whether the "Battery on" toggle is currently selected.
    /// Ignored when `ds1307` is `None`.
    #[prop_or_default]
    pub ds1307_battery_enabled: bool,
    /// Fires when the user drags the TMP101 slider. Plumbed through
    /// to `Tmp101Panel`; ignored when `tmp101` is `None`.
    pub on_set_tmp101_temperature: Callback<f32>,
    /// Fires when the user drags the test device's "poke" slider.
    /// Ignored when `test_device` is `None`.
    pub on_poke_test_device: Callback<u8>,
    /// Fires when the user flips the battery radio on/off.
    pub on_toggle_ds1307_battery: Callback<bool>,
    /// Fires when the user clicks "Set to system time" on the RTC card.
    pub on_set_ds1307_system_time: Callback<MouseEvent>,
}

#[function_component(I2cPanel)]
pub fn i2c_panel(props: &I2cPanelProps) -> Html {
    // Hide the whole block when no devices are attached to keep the
    // I/O column compact on programs that don't touch I2C.
    if props.tmp101.is_none()
        && props.test_device.is_none()
        && props.ds1307.is_none()
        && props.bus.attached == 0
    {
        return html! {};
    }

    html! {
        <div style="display:flex; flex-direction:column; gap:6px;">
            <div style="color:#bac2de; font-size:0.8rem;">{"I2C bus"}</div>
            <div style="background:#11111b; padding:6px 8px; border-radius:4px; \
                        font-family:monospace; font-size:12px; color:#bac2de; \
                        display:flex; gap:12px; flex-wrap:wrap;">
                <span>
                    {"state: "}
                    <span style="color:#89b4fa;">{
                        if props.bus.idle { "idle" } else { "active" }
                    }</span>
                </span>
                <span>
                    {"last byte: "}
                    <span style="color:#a6e3a1;">{
                        match props.bus.last_byte {
                            Some(b) => format!("0x{b:02X}"),
                            None    => "--".to_string(),
                        }
                    }</span>
                </span>
                <span>
                    {"last addr: "}
                    <span style="color:#fab387;">{
                        match props.bus.last_addressed {
                            Some(a) => format!("0x{a:02X}"),
                            None    => "--".to_string(),
                        }
                    }</span>
                </span>
                <span>
                    {"transactions: "}
                    <span style="color:#cba6f7;">{props.bus.transactions}</span>
                </span>
                <span>
                    {"attached: "}
                    <span style="color:#cba6f7;">{props.bus.attached}</span>
                </span>
            </div>

            // Per-device cards. Adding a new device = a new optional
            // prop above and a new render branch here.
            if let Some(snap) = props.tmp101 {
                <Tmp101Panel snapshot={snap}
                             on_set_temperature={props.on_set_tmp101_temperature.clone()} />
            }
            if let Some(snap) = props.test_device {
                <TestDevicePanel snapshot={snap}
                                 on_poke={props.on_poke_test_device.clone()} />
            }
            if let Some(snap) = props.ds1307 {
                <Ds1307Panel snapshot={snap}
                             battery_enabled={props.ds1307_battery_enabled}
                             on_toggle_battery={props.on_toggle_ds1307_battery.clone()}
                             on_set_system_time={props.on_set_ds1307_system_time.clone()} />
            }
        </div>
    }
}
