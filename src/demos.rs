//! Bundled COR24 assembly example catalog for the Load-demo dropdown.
//!
//! Sources are embedded via `include_str!` from sibling
//! `sw-cor24-x-assembler/src/examples/assembler/` so the web UI
//! doesn't need a runtime fetch. Display names mirror the
//! `examples()` list in `sw-cor24-x-assembler/tests/integration_tests.rs`
//! so anyone cross-referencing between the two finds the same labels.
//!
//! Adding a new example: drop the .s file in the sibling repo, append
//! the `(name, include_str!(...))` tuple here, and (if it's not
//! halting) make sure the existing `non_halting` UI logic in `main.rs`
//! knows to expect that.

pub const DEFAULT_SOURCE: &str = include_str!(
    "../../sw-cor24-x-assembler/src/examples/assembler/button_echo.s"
);

/// All bundled examples, ordered to match
/// sw-cor24-x-assembler/tests/integration_tests.rs::examples().
///
/// NOTE: "Button Echo (MakerLisp)" lives on
/// sibling pr/makerlisp-button-echo and lands once the coordinator
/// relays the brief at tools/briefs/dcxas-makerlisp-button-echo.md —
/// add it here in the same commit that re-builds pages/ after the
/// sibling's pr/ merges.
pub const EXAMPLES: &[(&str, &str)] = &[
    ("Add", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/add.s")),
    ("Assert", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/assert.s")),
    ("Blink LED", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/blink_led.s")),
    ("Button Echo", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/button_echo.s")),
    ("Comments", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/comments.s")),
    ("Countdown", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/countdown.s")),
    ("Echo", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/echo.s")),
    ("Fibonacci", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/fibonacci.s")),
    ("Literals", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/literals.s")),
    ("Loop Trace", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/loop_trace.s")),
    ("Memory Access", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/memory_access.s")),
    ("Multiply", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/multiply.s")),
    ("Nested Calls", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/nested_calls.s")),
    ("Stack Variables", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/stack_variables.s")),
    ("UART Hello", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/uart_hello.s")),
    ("Variables", include_str!("../../sw-cor24-x-assembler/src/examples/assembler/variables.s")),
];

/// Look up a bundled example by display name.
pub fn lookup(name: &str) -> Option<&'static str> {
    EXAMPLES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, src)| *src)
}
