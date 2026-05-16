Pick from the open follow-up list. Concrete candidates:

1. SPI/TMP125 panel + demo. sw-cor24-emulator main (8faf01f+) ships
   the SPI bus, TMP125 device, and tmp125.lgo demo. Same shape as
   the i2c TMP101 panel: src/panels/spi/{mod,container,tmp125}.rs
   with a slider (-128 °C to +128 °C in 0.25 °C steps, matching the
   chip's 10-bit register). Bit-bang lib + driver loop for the .s
   demo can be patterned on the i2c tmp101_read.s shape.

2. Add1Device panel under src/panels/i2c/add1.rs that shows
   `peek()` so the user sees the running stored byte as the
   add1_ping demo executes.

3. Per-demo I2C/SPI slave config (brief option 3) -- as soon as
   3+ I2C demos exist with conflicting slave maps.

4. Polish: keyboard shortcut to run, light/dark toggle, mobile
   responsive layout.

5. .lgo disassembly toggle (use cor24_emulator::EmulatorCore::
   disassemble) -- now lower priority since the user picks a
   readable .s demo instead.

Pick whichever the user steers toward; SPI/TMP125 is the strongest
fit for the saga's original 'more devices' theme.
