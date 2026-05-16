//! I2C device-panels.
//!
//! Each peripheral has its own file: `tmp101.rs` for the TMP101
//! temperature sensor today, future modules for RTC / LCD / extra
//! sensors / switches / displays drop in alongside it without
//! touching the I/O column container.
//!
//! Architecture (matches the rest of `src/panels/`):
//! - Each device's run-time state is captured in a small
//!   `PartialEq` snapshot struct so Yew can avoid re-rendering when
//!   nothing changes.
//! - The run loop in `main.rs` polls each attached `I2cHandle` once
//!   per tick and writes the snapshot into a `use_state`.
//! - `container::I2cPanel` accepts the bus snapshot plus a list of
//!   device snapshots and renders the header + device cards.

pub mod container;
pub mod test_device;
pub mod tmp101;

pub use container::{BusSnapshot, I2cPanel};
pub use test_device::TestDeviceSnapshot;
pub use tmp101::Tmp101Snapshot;
