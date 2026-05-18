//! SPI device-panels.
//!
//! Parallels `src/panels/i2c/` for SPI: each device has its own
//! module, the run loop in `main.rs` polls snapshots, and the
//! `SpiPanel` container renders the bus header + per-device cards.
//! SPI is single-slave today (per the cor24-emulator plan), so the
//! container is simpler than the I2C one — one device slot rather
//! than an address-routing table.

pub mod container;
pub mod echo;
pub mod sdcard;
pub mod tmp125;
pub mod w25q32;

pub use container::{SpiBusSnapshot, SpiPanel};
pub use echo::EchoSnapshot;
pub use sdcard::SdCardSnapshot;
pub use tmp125::Tmp125Snapshot;
pub use w25q32::W25q32Snapshot;
