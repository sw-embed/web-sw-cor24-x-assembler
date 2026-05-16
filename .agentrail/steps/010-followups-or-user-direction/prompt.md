The 'more devices' theme has shipped enough for now (LED, switch,
UART, registers, listing, TMP101 i2c, Add1 i2c, TMP125 spi). Pick
from any remaining seed when the user steers:

1. Add1Device i2c panel under src/panels/i2c/add1.rs that exposes
   peek() so the user sees the stored byte change as the demo runs.
2. RTC (DS3231) or LCD (HD44780): blocked on emulator-side device
   implementations -- if not yet upstream in
   sw-cor24-emulator/src/peripherals/i2c/devices/, brief dcemu to
   add them before adding panels here.
3. UI polish: keyboard shortcut Cmd/Ctrl+Enter to run, light/dark
   toggle, mobile responsive layout.
4. Adaptive run-loop budget: replace the static 100k/tick with a
   wall-clock budget (~8 ms/tick) using web_sys Performance, so
   demos with varying loop bodies all stay snappy without per-demo
   tuning.
5. .lgo disassembly toggle (cor24_emulator::EmulatorCore::disassemble)
   for users who paste raw .lgo into the editor and want a
   readable listing alongside.
