The 'TMP101 read (i2c, .lgo)' demo shipped in step 006 has two real
problems users hit on the deployed page:

1. The editor shows raw .lgo hex (L-record text) instead of readable
   assembly source. The user expects to see / edit source that maps
   to what's running.
2. The slider on the TMP101 panel changes the underlying device
   value but the printed UART output doesn't visibly track it. Root
   cause: the upstream tmp101.c demo from sw-cor24-emulator uses
   `t = -1; while (t--) {}` as its inter-read delay, which is
   ~2^31 iterations -- thousands of CPU-seconds between reads.

Fix:

1. Add a hand-written assembly demo at
   src/examples/tmp101_demo.s (lives in this repo, not in
   sw-cor24-x-assembler -- it's web-demo-specific). Minimal
   inlined I2C bit-bang that reads the TMP101 temperature
   register (config defaults to 12-bit on the simulated device,
   so 16-bit register layout is "value << 4" with the high byte
   carrying bits 11..4 of the 12-bit value) and prints it via
   UART. SHORT delay loop -- e.g. 200 iterations -- so the
   slider drives the printed output within a tick or two.
2. Wire the new .s into demos.rs as "TMP101 read (i2c)" --
   include_str! from the local path, dispatched through the
   normal assembler path (no .lgo).
3. Replace the existing "TMP101 read (i2c, .lgo)" entry with the
   new .s one. The .lgo dispatch path itself (added in step 006)
   stays in place since it's useful infrastructure for any
   future pre-built demo, but no bundled demo uses it today.
4. Rebuild pages/, clippy clean.

Exit criteria:
- Editor shows readable assembly when "TMP101 read (i2c)" is
  loaded.
- Dragging the slider while the demo runs visibly changes the
  printed UART output within one to two seconds of wall-clock
  time.
- cargo clippy -D warnings green.
- One commit; pages/ committed alongside source changes.
