Add a new Button Echo variation to sibling sw-cor24-x-assembler that
reproduces the 27-byte MakerLisp blinky_s2 program (see
dcemu/sw-cor24-emulator/tests/programs/blinky_s2.lgo and the
docs/makerlisp-blinky_s2.md disassembly).

Scope:

1. Work on a new branch `feat/makerlisp-button-echo` in the sibling
   `../sw-cor24-x-assembler` clone (use `dg-new-feature` from that
   repo so it's based on origin/dev there).
2. Add `../sw-cor24-x-assembler/src/examples/assembler/button_echo_makerlisp.s`
   reproducing the disassembly: prologue (push fp / push r2 / push r1
   / mov fp,sp), `la r2,0xFF0000`, `lb r0,(r2)`, `sb r0,(r2)`, `bra`
   back; then a startup block at the end that sets `sp = 0xFEEC00`,
   parks `r1 = 0xFEE000`, and `call`s into the loop. Source order
   matters because the entry point is at byte offset 0x000E.
3. Register the example in `tests/integration_tests.rs`'s `examples()`
   vec as `("Button Echo (MakerLisp)", include_str!(...))`.
4. Verify with `cargo test` in the sibling that the example assembles
   and the test suite stays green.
5. Commit in the sibling repo on `feat/makerlisp-button-echo`.
6. Back in web-sw-cor24-x-assembler, this saga step records the cross-
   repo change in `.agentrail/steps/002-makerlisp-button-echo/` and
   commits that artifact here.

Exit criteria:
- New `.s` file present and registered.
- `cargo test` green in sw-cor24-x-assembler.
- Two commits land: one in sibling on `feat/makerlisp-button-echo`,
  one here that records the saga step.
- Cross-repo summary captured in the step's --summary at completion.
