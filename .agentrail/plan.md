# Extract assembler live-demo into web-sw-cor24-x-assembler

This saga ports the assembler live-demo from cor24-rs (in deprecation) to web-sw-cor24-x-assembler, with the standalone cor24-assembler crate (in sibling sw-cor24-x-assembler) as the assembling backend and cor24-emulator (sibling, default-features=false) as the execution backend.

Closest scaffold reference: web-sw-cor24-x-tinyc. Examples live in sibling sw-cor24-x-assembler/src/examples/assembler/ and are surfaced to the web UI when the demo lands.

## Planned steps (will accrete as work happens)

1. bootstrap-saga -- Consolidate AGENTS.md / CLAUDE.md, init this saga, stage sibling clones for path-deps.
2. makerlisp-button-echo -- Add a Button Echo variation reproducing the 27-byte MakerLisp blinky_s2 listing to sibling sw-cor24-x-assembler's example set, registered in the integration-test catalog.
3. (TBD) Port the assembler tab from cor24-rs/src/app.rs into src/.
4. (TBD) Wire scripts/build-pages.sh and pages/ deployment per web-sw-cor24-x-tinyc's pattern.
