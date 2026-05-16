Add an extensible I2C device-panel framework to web-sw-cor24-x-assembler
so the next step can drop TMP101 in and future steps can drop RTC, LCD,
extra sensors, switches, and displays in by adding a new file under
src/panels/i2c/.

Reference:
- sw-cor24-emulator/src/peripherals/i2c/{device.rs, registry.rs,
  handle.rs, mod.rs} for the back-end plumbing (I2cDevice trait,
  I2cHandle<D> shared-state pattern, registry).
- sw-cor24-emulator/src/peripherals/i2c/devices/{tmp101.rs, add1.rs}
  as the two existing device implementations to attach panels to.
- cor24_emulator::EmulatorCore::{attach_i2c_device,
  attach_i2c_device_shared, detach_i2c_devices, i2c, i2c_log,
  format_i2c_log} for the surface the UI calls.

Scope of this step (no new device behavior yet — that's step 006):
1. src/panels/i2c/mod.rs -- registry + dispatch. Defines a
   DevicePanel trait with `name(&self) -> &str`,
   `address(&self) -> u8`, and `render(&self, ctx: &PanelCtx) -> Html`.
   PanelCtx carries an Rc<RefCell<EmulatorCore>> and the shared
   I2cHandle for the device so the panel can both read state and
   mutate it (set_temperature, etc.).
2. src/panels/i2c/container.rs -- a YEW component that takes a Vec
   of Box<dyn DevicePanel> and renders them in order, with a header
   showing the bus state from emu.i2c() (SCL/SDA, last byte, ack
   count if available).
3. src/panels/i2c/tmp101.rs -- empty stub: TMP101 panel with no
   slider yet, just renders "TMP101 @ 0x4A: <register-byte hex>"
   to prove the wiring. Full UX lands in step 006.
4. Wire the container into the I/O panel column in main.rs below
   the existing LED+switch row, behind a feature toggle (just
   `let i2c_devices = vec![...]` for now -- no compile-time gate).
5. Rebuild pages/, cargo clippy clean.

Exit criteria:
- src/panels/i2c/{mod, container, tmp101}.rs all present.
- TMP101 panel shows up in the deployed UI (even if just a label
  with the current pointer-register byte).
- One commit; pages rebuilt.
