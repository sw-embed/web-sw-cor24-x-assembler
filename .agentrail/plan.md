# Extract assembler live-demo into web-sw-cor24-x-assembler

This saga ports the assembler live-demo from cor24-rs (in deprecation) to web-sw-cor24-x-assembler, with the standalone cor24-assembler crate (in sibling sw-cor24-x-assembler) as the assembling backend and cor24-emulator (sibling, default-features=false) as the execution backend.

Closest scaffold reference: web-sw-cor24-x-tinyc (Yew + Trunk shape, panels/ pattern); deployment / pages workflow reference: web-sw-cor24-basic (scripts/build-pages.sh, .github/workflows/pages.yml, pages/ as GH Actions artifact root).

## Planned steps (will accrete as work happens)

1. bootstrap-saga -- Consolidate AGENTS.md / CLAUDE.md, init this saga, stage sibling clones for path-deps. (DONE)

2. makerlisp-button-echo -- Add a Button Echo variation reproducing the 27-byte MakerLisp blinky_s2 listing to sibling sw-cor24-x-assembler's example set. (DONE -- handed off to dcxas via brief; awaiting relay.)

3. pages-pipeline-bootstrap -- Set up the pages-deploy infrastructure modelled on web-sw-cor24-basic: scripts/build-pages.sh, scripts/serve.sh, .github/workflows/pages.yml, pages/ + pages/.nojekyll seeded from current scaffold, README live-demo link. Repo settings already configured for GH Actions Pages deployment by mike.

4. demo-extraction-port -- Port the Assembler tab from cor24-rs/src/app.rs into src/, with src/panels/{led,switch,uart,registers,listing}.rs following the web-sw-cor24-x-tinyc pattern. Wire cor24-assembler for assembly and cor24-emulator for execution. Bundled example list including both Button Echo variants.

5. i2c-panels-framework -- Add src/panels/i2c/ with an extensible device-panel trait + registry so future I2C devices (RTC, LCD, sensors, switches, displays) drop in without touching the I/O panel container. Hooks into cor24-emulator's I2cHandle/I2cDevice plumbing.

6. i2c-tmp101-panel -- First concrete I2C device-panel: TMP101 temperature probe. Shows current temperature register value (with active resolution), lets the user drag a slider to set the simulated °C, reflects guest reads. Verifies the i2c/tmp101.lgo demo runs end-to-end in the browser.

## Future-step seeds (not yet stepped)

- i2c-rtc-panel  (DS3231-style real-time clock; date/time controls)
- i2c-lcd-panel  (HD44780-style character LCD; mirror the 16x2 display)
- i2c-additional-sensors / -switches / -displays as device implementations land in cor24-emulator
- ui-polish: keyboard shortcuts, dark/light theme, mobile responsive
