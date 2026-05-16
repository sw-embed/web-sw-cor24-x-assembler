//! Assembler pipeline: COR24 assembly source → listing + bytes.
//!
//! Thin wrapper around `cor24_assembler::Assembler` to give the Yew
//! `App` in `main.rs` the same `compile()` -> `AssembleOutput` shape
//! that web-sw-cor24-x-tinyc uses for its C compiler — so the run loop
//! plumbing is identical between repos.

use cor24_assembler::{AssembledLine, Assembler, AssemblyResult};

pub struct AssembleError {
    pub message: String,
    /// 1-based source line number, when the assembler reports one.
    pub line: Option<usize>,
}

pub struct AssembleOutput {
    pub listing: Vec<AssembledLine>,
    pub bytes: Vec<u8>,
    pub error: Option<AssembleError>,
}

/// Run the assembler over `source` and produce a listing + flat byte
/// vec, plus a structured error if assembly failed.
pub fn assemble(source: &str) -> AssembleOutput {
    let mut asm = Assembler::new();
    let result: AssemblyResult = asm.assemble(source);

    let error = if result.errors.is_empty() {
        None
    } else {
        // cor24-assembler error messages start with "Line N:" — pull
        // the number out for inline highlighting.
        let first = result.errors[0].clone();
        let line = parse_line_from_error(&first);
        Some(AssembleError {
            message: result.errors.join("\n"),
            line,
        })
    };

    AssembleOutput {
        listing: result.lines,
        bytes: result.bytes,
        error,
    }
}

fn parse_line_from_error(msg: &str) -> Option<usize> {
    let rest = msg.strip_prefix("Line ")?;
    let end = rest.find(':')?;
    rest[..end].parse().ok()
}

/// Best-effort sniffer: does `source` look like a `.lgo` load file
/// rather than COR24 assembly source?
///
/// `.lgo` lines are `L<6-hex><payload>` or `G<6-hex>`. Returns true
/// as soon as any L-record line is found, allowing the run-button to
/// route the source through `EmulatorCore::load_lgo` instead of the
/// assembler. Assembly source uses `;` for comments and never has a
/// bare line of the form `L` + 6 hex digits.
pub fn looks_like_lgo(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.len() < 7 {
            continue;
        }
        if trimmed.starts_with('L') && trimmed[1..7].chars().all(|c| c.is_ascii_hexdigit()) {
            return true;
        }
    }
    false
}

/// Find the 1-based listing line whose address range contains the
/// given PC, so a runtime fault can highlight the right line.
pub fn pc_to_listing_line(listing: &[AssembledLine], pc: u32) -> Option<usize> {
    for (i, line) in listing.iter().enumerate() {
        if !line.bytes.is_empty() {
            let start = line.address;
            let end = start + line.bytes.len() as u32;
            if pc >= start && pc < end {
                return Some(i + 1);
            }
        }
    }
    None
}
