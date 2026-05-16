Pick from the future-step seeds in .agentrail/plan.md when the user
directs the next step. Natural follow-ups:

1. i2c-test-device-panel: src/panels/i2c/test_device.rs that shows
   the slave's stored byte (Add1Device::peek()) with a poke control
   when the user attaches it. Update I2cPanel container to include
   the new card alongside TMP101. As the slave gains registers
   upstream, broaden the panel.

2. adaptive run-loop budget: replace 100k/tick static with a
   wall-clock budget (~8 ms/tick) via web_sys::Performance, so
   demos with varying loop bodies all stay snappy without per-demo
   tuning. Removes one of the few footguns left from the .lgo
   delay-loop debacle.

3. UI polish: Cmd/Ctrl+Enter to run, light/dark toggle, mobile
   layout pass.

4. RTC / LCD: blocked on emulator-side device implementations --
   if not yet in sw-cor24-emulator/src/peripherals/i2c/devices/,
   brief dcemu before adding panels here.

5. .lgo disassembly toggle (EmulatorCore::disassemble) for users
   who paste raw .lgo and want a readable listing.
