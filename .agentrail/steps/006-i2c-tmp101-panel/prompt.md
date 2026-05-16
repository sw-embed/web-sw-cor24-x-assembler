Make the TMP101 card interactive: a slider (range input) lets the
user set the simulated temperature in °C live while a program is
running, and the value the guest reads back from the device updates
accordingly.

Scope:
1. src/panels/i2c/tmp101.rs gains a `Callback<f32>` prop
   (`on_set_temperature`) and a range input bound to it. Range
   -55.0 to 127.9375 °C in 0.0625 °C steps (the TMP101's actual
   12-bit register range and LSB).
2. main.rs holds the slider state alongside the snapshot, wires the
   callback so it calls
   `handle.with(|d| d.set_temperature(v))` on the current
   tmp101_handle, then re-reads the snapshot so the UI stays in
   sync.
3. Optional: also expose the resolution as a small dropdown
   (9-bit / 10-bit / 11-bit / 12-bit) so users can see the
   quantization effect in real time. If it complicates the layout,
   defer to a future polish step.
4. Verify end-to-end with the existing
   sw-cor24-emulator/examples/i2c/tmp101/tmp101.lgo demo. The
   demo reads the temperature and prints it over UART; with the
   slider you should be able to drag the temp and see the printed
   value follow it.
5. pages/ rebuilt; cargo clippy -D warnings green.

Stretch: add `examples/i2c_tmp101.s` (or wire the existing
tmp101.lgo) into the demos.rs dropdown so the live URL ships a
ready-to-run demo program that exercises the new panel.
