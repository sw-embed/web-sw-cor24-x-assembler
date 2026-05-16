//! Simple COR24-assembly syntax highlighter for the browser editor.
//!
//! Produces a Vec of colored spans from assembly source text, used to
//! render highlighted code in a `<pre>` element beneath the editor
//! textarea (same overlay technique as web-sw-cor24-x-tinyc's C
//! highlighter — only the lexer rules differ).
//!
//! COR24 syntax (matching what the cor24-assembler accepts):
//! - `;` to end-of-line: comments
//! - identifier `:` : labels
//! - leading `.` : directives (.org, .byte, .word, .zero, ...)
//! - bare identifiers : mnemonics (push, mov, la, lb, sb, bra, ...) or
//!   register names (r0, r1, r2, fp, sp, z, iv, ir)
//! - 0x-prefixed / decimal numbers

/// A colored fragment of source text.
pub struct Span {
    pub text: String,
    pub color: &'static str,
}

// Catppuccin Mocha palette (same as web-sw-cor24-x-tinyc).
const COMMENT: &str = "#a6adc8"; // overlay0
const LABEL: &str = "#f9e2af"; // yellow
const MNEMONIC: &str = "#cba6f7"; // mauve
const REGISTER: &str = "#89b4fa"; // blue
const NUMBER: &str = "#fab387"; // peach
const DIRECTIVE: &str = "#f38ba8"; // red
const PLAIN: &str = "#cdd6f4"; // text

const MNEMONICS: &[&str] = &[
    // Branches / jumps / calls
    "bra", "brf", "brt", "jmp", "jal",
    // Memory: load/store byte + word, load-address, load-constant
    "lb", "lbu", "sb", "lw", "sw", "la", "lc", "lcu",
    // ALU
    "add", "sub", "mul", "and", "or", "xor", "shl", "sra", "srl",
    // Compares
    "ceq", "cls", "clu",
    // Stack
    "push", "pop",
    // Misc
    "mov", "sxt", "zxt",
];

const REGISTERS: &[&str] = &["r0", "r1", "r2", "fp", "sp", "z", "c", "iv", "ir"];

/// Highlight COR24 assembly source into colored spans.
pub fn highlight(source: &str) -> Vec<Span> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut spans = Vec::new();
    let mut i = 0;
    let mut at_line_start = true;

    while i < len {
        let ch = bytes[i];

        // Comments to end of line
        if ch == b';' {
            let start = i;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            spans.push(Span {
                text: source[start..i].to_string(),
                color: COMMENT,
            });
            continue;
        }

        // Numbers (0x-hex and decimal; allow optional leading -)
        if ch.is_ascii_digit() || (ch == b'-' && i + 1 < len && bytes[i + 1].is_ascii_digit()) {
            let start = i;
            if ch == b'-' {
                i += 1;
            }
            if i + 1 < len && bytes[i] == b'0' && (bytes[i + 1] == b'x' || bytes[i + 1] == b'X') {
                i += 2;
                while i < len && bytes[i].is_ascii_hexdigit() {
                    i += 1;
                }
            } else {
                while i < len && bytes[i].is_ascii_digit() {
                    i += 1;
                }
            }
            spans.push(Span {
                text: source[start..i].to_string(),
                color: NUMBER,
            });
            at_line_start = false;
            continue;
        }

        // Directives: `.identifier` (must start at start of "word")
        if ch == b'.' && i + 1 < len && (bytes[i + 1].is_ascii_alphabetic() || bytes[i + 1] == b'_')
        {
            let start = i;
            i += 1;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            spans.push(Span {
                text: source[start..i].to_string(),
                color: DIRECTIVE,
            });
            at_line_start = false;
            continue;
        }

        // Identifiers: labels, mnemonics, registers, plain
        if ch.is_ascii_alphabetic() || ch == b'_' {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &source[start..i];

            // Peek for a `:` (label) — labels can have leading whitespace,
            // so the check is "followed by `:` after optional whitespace
            // that does not cross a newline".
            let mut peek = i;
            while peek < len && (bytes[peek] == b' ' || bytes[peek] == b'\t') {
                peek += 1;
            }
            let is_label = at_line_start && peek < len && bytes[peek] == b':';

            let word_lower = word.to_ascii_lowercase();
            let color = if is_label {
                LABEL
            } else if MNEMONICS.contains(&word_lower.as_str()) {
                MNEMONIC
            } else if REGISTERS.contains(&word_lower.as_str()) {
                REGISTER
            } else {
                PLAIN
            };

            spans.push(Span {
                text: word.to_string(),
                color,
            });
            at_line_start = false;
            continue;
        }

        // Everything else (whitespace, operators, punctuation). Batch
        // until the next "interesting" byte.
        let start = i;
        let was_newline = ch == b'\n';
        i += 1;
        while i < len
            && !bytes[i].is_ascii_alphanumeric()
            && bytes[i] != b'_'
            && bytes[i] != b';'
            && bytes[i] != b'.'
            && bytes[i] != b'-'
            && bytes[i] != b'\n'
        {
            i += 1;
        }
        spans.push(Span {
            text: source[start..i].to_string(),
            color: PLAIN,
        });
        if was_newline {
            at_line_start = true;
        }
    }

    spans
}
