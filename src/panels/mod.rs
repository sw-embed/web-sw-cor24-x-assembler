//! UI panels rendered inside the App layout.
//!
//! Each peripheral has its own component module so the run loop in `main.rs`
//! stays small. The `i2c/` submodule holds one file per simulated I2C
//! device — future devices (RTC, LCD, additional sensors, switches,
//! displays) land there alongside `tmp101.rs` without touching this file.

pub mod i2c;
pub mod led;
pub mod listing;
pub mod registers;
pub mod spi;
pub mod switch;
pub mod uart;

pub use i2c::{BusSnapshot, I2cPanel, Tmp101Snapshot};
pub use led::LedPanel;
pub use registers::RegistersPanel;
pub use spi::{SpiBusSnapshot, SpiPanel, Tmp125Snapshot};
pub use switch::SwitchPanel;
pub use uart::UartPanel;
