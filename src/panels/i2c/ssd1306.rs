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

    // PCB module render. Modelled on a real 0.96" two-color OLED
    // breakout (see docs/i2c-oled.png): royal-blue PCB with rounded
    // corners, four chrome mounting holes near the corners, a row
    // of four pin-header pads + GND/VCC/SCL/SDA silkscreen labels
    // along the top, and a black inset around the canvas. The
    // canvas itself carries the live framebuffer.
    let hole = |corner: &'static str| -> Html {
        html! {
            <div style={format!(
                "position:absolute; {corner}; width:14px; height:14px; \
                 background:radial-gradient(circle at 35% 30%, #f5f5f5, #888 65%, #2a2a2a 100%); \
                 border-radius:50%; box-shadow:inset 0 0 2px rgba(0,0,0,0.7);"
            )} />
        }
    };
    let pin = |label: &'static str| -> Html {
        html! {
            <div style="display:flex; flex-direction:column; align-items:center; gap:1px;">
                <div style="width:10px; height:10px; border-radius:50%; \
                            background:radial-gradient(circle at 35% 30%, #f0f0f0, #999 60%, #333); \
                            box-shadow:0 0 2px rgba(0,0,0,0.5);" />
                <span style="font-family:Arial, sans-serif; font-size:8px; color:#ffffff; \
                             text-shadow:0 0 1px rgba(0,0,0,0.6); letter-spacing:0.5px;">
                    {label}
                </span>
            </div>
        }
    };

    let pcb_blue = "#1565c0";
    let pcb_blue_dark = "#0d47a1";
    let pcb_width = canvas_width + 36; // 18px PCB margin each side

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
            <div style={format!(
                "position:relative; width:{pcb_width}px; \
                 background:linear-gradient(180deg, {pcb_blue} 0%, {pcb_blue_dark} 100%); \
                 border-radius:10px; padding:36px 18px 22px 18px; \
                 box-shadow:0 2px 8px rgba(0,0,0,0.4), inset 0 1px 0 rgba(255,255,255,0.1);"
            )}>
                // Corner mounting holes.
                { hole("top:6px; left:6px") }
                { hole("top:6px; right:6px") }
                { hole("bottom:6px; left:6px") }
                { hole("bottom:6px; right:6px") }

                // Top header row: 4 pin pads + silkscreen labels.
                <div style="position:absolute; top:4px; left:0; right:0; \
                            display:flex; justify-content:center; gap:14px;">
                    { pin("GND") }
                    { pin("VCC") }
                    { pin("SCL") }
                    { pin("SDA") }
                </div>

                // Black inset frame around the OLED itself.
                <div style="background:#000; padding:4px; border-radius:3px; \
                            border:1px solid #0a0a0a; \
                            box-shadow:inset 0 0 6px rgba(0,0,0,0.8), \
                                       0 0 8px rgba(116,199,236,0.06);">
                    <canvas ref={canvas_ref}
                            width={canvas_width.to_string()}
                            height={canvas_height.to_string()}
                            style="display:block; image-rendering:pixelated;" />
                </div>
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
