With the MakerLisp Button Echo variant in place upstream in the
sibling sw-cor24-x-assembler crate, begin porting the assembler
live-demo from cor24-rs into this repo (web-sw-cor24-x-assembler).

Reference scaffolds:
- web-sw-cor24-x-tinyc (closest shape: Yew + Trunk, dist/->pages/,
  compile-then-emulate)
- cor24-rs/src/app.rs ("Assembler" tab) -- the upstream demo we are
  extracting
- cor24-rs/src/assembler.rs -- the inline assembler being replaced
  by the cor24-assembler path-dep from sw-cor24-x-assembler

Scope of this step:
1. Port the Assembler tab's view + state machine from cor24-rs into
   src/, wiring cor24-assembler for assembly and cor24-emulator
   (default-features=false) for execution.
2. Wire the bundled example list -- including the new
   "Button Echo (MakerLisp)" -- so the dropdown reflects whatever
   sw-cor24-x-assembler exposes (via the integration_tests.rs
   pattern or a published list helper).
3. Verify `trunk serve` runs the demo locally and the
   "Button Echo (MakerLisp)" example assembles, loads, and S2 toggles
   D2 in the in-browser I/O panel.

Exit criteria:
- src/ contains the ported demo (not the scaffold placeholder).
- `trunk build --release` succeeds; `cargo clippy -- -D warnings`
  is green.
- Both Button Echo examples appear in the in-app dropdown.
- A short note in README.md flips the project status from "Scaffold."
  to a "working demo" line.
