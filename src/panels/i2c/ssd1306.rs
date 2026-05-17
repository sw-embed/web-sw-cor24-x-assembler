//! SSD1306 I2C OLED panel.
//!
//! Renders the chip's 128×64 (or 128×32) GDDRAM framebuffer onto an
//! HTML `<canvas>`. Per user direction the panel evokes the
//! common-as-mud two-color 0.96" OLED module (yellow band on rows
//! 0..16, blue band on rows 16..64) rather than a uniform pale-green
//! monochrome — that's a hardware feature of the physical display,
//! not the chip, so the colouring lives entirely here in the web
//! layer.
//!
//! Output-only: no sliders / toggles. The user can't poke the
//! framebuffer from this panel (the demo does that via I2C).

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

const PAGES: usize = 8;
const COLS: usize = 128;
pub const FRAMEBUFFER_LEN: usize = PAGES * COLS;

/// 3× scale for the rendered canvas — 128×64 → 384×192, comfortable
/// to read at a normal browser zoom.
const SCALE: u32 = 3;

/// Two-color hardware band: rows < THIS are rendered in the yellow
/// colour; the rest are blue. Standard 16-pixel yellow band.
const YELLOW_BAND_END: u16 = 16;

/// Catppuccin Mocha-tinted versions of the OLED's two colours that
/// keep enough saturation to read as the real screen would.
const COLOR_YELLOW: &str = "#f9e2af";
const COLOR_BLUE: &str = "#74c7ec";
const COLOR_OFF: &str = "#0a0a0a";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ssd1306Snapshot {
    pub address: u8,
    pub width: u16,
    pub height: u16,
    pub display_on: bool,
    /// Page-major, LSB-up — `framebuffer[page * 128 + col]` is 8
    /// vertical pixels of column `col` in page `page`, bit 0 at the
    /// top. Matches the SSD1306 GDDRAM wire format.
    pub framebuffer: Vec<u8>,
}

#[derive(Properties, PartialEq)]
pub struct Ssd1306PanelProps {
    pub snapshot: Ssd1306Snapshot,
}

#[function_component(Ssd1306Panel)]
pub fn ssd1306_panel(props: &Ssd1306PanelProps) -> Html {
    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        let snap = props.snapshot.clone();
        use_effect_with(snap, move |snap| {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                draw(&canvas, snap);
            }
            || ()
        });
    }

    let s = &props.snapshot;
    let canvas_width = u32::from(s.width) * SCALE;
    let canvas_height = u32::from(s.height) * SCALE;

    html! {
        <div style="background:#11111b; padding:8px; border-radius:4px; \
                    border:1px solid #313244; display:flex; flex-direction:column; gap:6px;">
            <div style="display:flex; justify-content:space-between; align-items:baseline;">
                <span style="color:#cdd6f4; font-weight:600; font-size:0.85rem;">
                    {"OLED Display"}
                </span>
                <span style="color:#a6adc8; font-family:monospace; font-size:0.75rem;">
                    {format!("@ 0x{:02X} \u{00b7} {}\u{00d7}{} \u{00b7} {}",
                        s.address, s.width, s.height,
                        if s.display_on { "on" } else { "off" })}
                </span>
            </div>
            // The "bezel" — a slim dark plastic surround that hints at
            // the physical module without trying to look like a PCB
            // render. The canvas itself is what carries the live state.
            <div style="display:inline-block; padding:6px; background:#000; \
                        border-radius:4px; border:1px solid #45475a; \
                        box-shadow:0 0 10px rgba(116,199,236,0.08), \
                                   inset 0 0 4px rgba(0,0,0,0.6);">
                <canvas ref={canvas_ref}
                        width={canvas_width.to_string()}
                        height={canvas_height.to_string()}
                        style="display:block; image-rendering:pixelated;" />
            </div>
            <div style="color:#6c7086; font-size:0.7rem;">
                {"two-color OLED: top 16 rows yellow, rest blue"}
            </div>
        </div>
    }
}

fn draw(canvas: &HtmlCanvasElement, snap: &Ssd1306Snapshot) {
    let Ok(Some(ctx_object)) = canvas.get_context("2d") else {
        return;
    };
    let Ok(ctx) = ctx_object.dyn_into::<CanvasRenderingContext2d>() else {
        return;
    };

    let w = u32::from(snap.width) * SCALE;
    let h = u32::from(snap.height) * SCALE;
    let scale_f = f64::from(SCALE);

    // Off / unlit background.
    ctx.set_fill_style_str(COLOR_OFF);
    ctx.fill_rect(0.0, 0.0, f64::from(w), f64::from(h));

    // When the chip is powered down, just leave the canvas dark and
    // overlay a label. Real OLEDs go fully black when off — no
    // residual ghosting — so a uniform fill is the right look.
    if !snap.display_on {
        ctx.set_fill_style_str("#6c7086");
        ctx.set_font("14px monospace");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        let _ = ctx.fill_text(
            "(display off)",
            f64::from(w) / 2.0,
            f64::from(h) / 2.0,
        );
        return;
    }

    // Walk the framebuffer. For each lit pixel, draw a SCALE×SCALE
    // rectangle in the colour that band-of-the-display calls for.
    // Doing the colour split per-pixel inside the loop is cheap; for
    // a band-aware fast path we could batch yellow + blue
    // separately, but the canvas operations dominate either way at
    // 8192 pixels max.
    let max_page = (snap.height / 8).min(PAGES as u16) as usize;
    for page in 0..max_page {
        for col in 0..(snap.width as usize).min(COLS) {
            let byte_idx = page * COLS + col;
            let Some(byte) = snap.framebuffer.get(byte_idx) else {
                continue;
            };
            if *byte == 0 {
                continue;
            }
            for bit in 0..8 {
                if byte & (1 << bit) == 0 {
                    continue;
                }
                let y_px = page as u16 * 8 + bit;
                if y_px >= snap.height {
                    break;
                }
                let color = if y_px < YELLOW_BAND_END {
                    COLOR_YELLOW
                } else {
                    COLOR_BLUE
                };
                ctx.set_fill_style_str(color);
                ctx.fill_rect(
                    f64::from(col as u32) * scale_f,
                    f64::from(u32::from(y_px)) * scale_f,
                    scale_f,
                    scale_f,
                );
            }
        }
    }
}
