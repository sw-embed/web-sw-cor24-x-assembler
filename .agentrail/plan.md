# Extract assembler live-demo into web-sw-cor24-x-assembler

This saga ports the assembler live-demo from cor24-rs (in deprecation) to web-sw-cor24-x-assembler, with the standalone cor24-assembler crate (in sibling sw-cor24-x-assembler) as the assembling backend and cor24-emulator (sibling, default-features=false) as the execution backend.

Closest scaffold reference: web-sw-cor24-x-tinyc (Yew + Trunk shape, panels/ pattern); deployment / pages workflow reference: web-sw-cor24-basic (scripts/build-pages.sh, .github/workflows/pages.yml, pages/ as GH Actions artifact root).

## Planned steps (will accrete as work happens)

1. bootstrap-saga -- Consolidate AGENTS.md / CLAUDE.md, init this saga, stage sibling clones for path-deps. (DONE)

2. makerlisp-button-echo -- Add a Button Echo variation reproducing the 27-byte MakerLisp blinky_s2 listing to sibling sw-cor24-x-assembler's example set. (DONE)

3. pages-pipeline-bootstrap -- ./scripts/build-pages.sh, ./scripts/serve.sh, GH-Actions deploy of ./pages/. (DONE)

4. demo-extraction-port -- Yew app: editor + listing + emulator I/O panel; LED/switch/UART/registers panels; assembler-driven run loop. (DONE)

5. i2c-panels-framework -- src/panels/i2c/ skeleton (BusSnapshot, container, Tmp101Snapshot stub). (DONE)

6. i2c-tmp101-panel -- TMP101 slider control + .lgo demo. (DONE)

7. fix-tmp101-demo -- Bumped per-tick run_batch budget so slider visibly drives output through the upstream .lgo's idle loop. (DONE)

8. demo-polish-and-more-devices -- Readable .s TMP101 demo, "I2C Test Device Ping" demo wired in (per dcxas brief), UART autoscroll. (DONE -- demo originally named after the Add1 chip; renamed to "test device" in step 10 because the upstream slave's role generalised beyond +1.)

9. spi-tmp125-and-followups -- SPI panel framework + TMP125 device card + spi tight-loop .s demo + run-loop budget pulled back to 100k/tick + UART display capped at 4 KB tail. (DONE)

10. per-demo-device-config -- Each bundled demo now declares which simulated peripherals it expects (DemoConfig in src/demos.rs); only those devices are attached on Assemble & Run, so picking 'I2C TMP101 Read' shows only the TMP101 card and 'SPI TMP125 Read' shows only the TMP125 card. Renamed dropdown entries: 'I2C TMP101 Read', 'I2C Test Device Ping', 'SPI TMP125 Read' -- all bus demos start with their bus name; full list re-alphabetized. Fixed UART layout (was absorbing column space and overlapping the next panel). Renamed the Add1 slave to 'I2C Test Device' in the UI -- the device will grow registers beyond +1 as we add functionality, so the name reflects that broader role going forward.

## Future-step seeds (not yet stepped)

- i2c-test-device-panel -- src/panels/i2c/test_device.rs that shows the device's stored byte (`Add1Device::peek`) and exposes a poke control as it gains registers. Naming: "test device" in UI; underlying type stays `Add1Device` until a cross-repo brief renames it.
- i2c-rtc-panel  (DS3231-style real-time clock; needs device impl in sw-cor24-emulator first -- brief dcemu if absent)
- i2c-lcd-panel  (HD44780-style character LCD; same precondition)
- adaptive run-loop budget: replace the static 100k/tick with a wall-clock budget (~8 ms/tick) using web_sys Performance so demos with varying loop bodies all stay snappy without per-demo tuning
- ui-polish: keyboard shortcut to Run (Cmd/Ctrl + Enter), light/dark theme toggle, mobile responsive layout
- .lgo disassembly toggle (cor24_emulator::EmulatorCore::disassemble) for users who paste raw .lgo into the editor

## Naming convention going forward

Bus-using demos are prefixed by their bus (I2C / SPI) and capitalized words; the list stays alphabetical on the bus prefix so I2C demos cluster between 'I' words and SPI demos cluster between 'S' words.
