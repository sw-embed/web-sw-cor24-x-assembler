With the TMP101 slice live the saga's "Plan + working TMP101 slice"
deliverable is met. Pick from the remaining seed list when you start
the next saga; each item is its own step.

Likely next batch (any order):

1. i2c-rtc-panel -- attach a DS3231-style real-time-clock device
   (sw-cor24-emulator side already has the I2cDevice trait
   surface; the device implementation may or may not exist yet --
   check and brief dcemu if not).
2. i2c-lcd-panel -- HD44780-style character LCD; mirror the 16x2
   display as ASCII in the panel.
3. ui-polish: keyboard shortcut to Run (Cmd/Ctrl + Enter), light
   theme toggle (the Catppuccin Mocha is fixed today), responsive
   layout for narrow viewports.
4. assembler-direct: today the .lgo demo skips listing/highlighting.
   Add a "Show LGO" toggle that disassembles loaded bytes back to a
   listing (cor24_emulator::EmulatorCore::disassemble) so the demo
   user sees what's running.
5. examples sourcing: consider promoting the I2C .lgo demo to a
   first-class .s example in sw-cor24-x-assembler (cross-repo brief
   to dcxas, same shape as the makerlisp button echo handoff) so it
   round-trips through the assembler.
