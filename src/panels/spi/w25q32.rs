//! SPI W25Q32 NOR flash device-panel.
//!
//! 4 MiB of flash visualized as a 64×16 sector heatmap (one pixel
//! per 4 KiB sector; 1024 sectors total = 4 MiB). Recently-written
//! pixels glow warm; the heat decays each tick so quiescent regions
//! fade back to neutral. Untouched all-`0xFF` "erased" sectors render
//! pale to distinguish them from "written but cooled" sectors.
//!
//! Buttons: Reset / Chip Erase (calls `erase_chip` + clears IndexedDB
//! + repaints the heatmap pale), Load from file (4 MiB upload).

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, File, HtmlCanvasElement, HtmlInputElement};
use yew::prelude::*;

/// Sectors are 4 KiB each; 4 MiB / 4 KiB = 1024 sectors.
pub const SECTOR_COUNT: usize = 1024;
/// Heatmap canvas dimensions (in sectors). 64×16 = 1024. Wide-and-
/// short to fit under the existing panel column.
const HEATMAP_COLS: u32 = 64;
const HEATMAP_ROWS: u32 = 16;
/// One screen pixel per sector at 3× scale -- canvas is 192×48,
/// matches the OLED panel's footprint roughly.
const HEATMAP_SCALE: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct W25q32Snapshot {
    pub cs: u8,
    pub jedec_id: [u8; 3],
    pub wip: bool,
    pub wel: bool,
    pub last_accessed_address: Option<u32>,
    /// Per-4KB-sector cooling counter: 0 = cold (untouched lately),
    /// 255 = just-written. Decays in `main.rs`'s tick loop so the
    /// heatmap fades over a few seconds after each program/erase.
    pub sector_activity: Vec<u8>,
    /// Per-4KB-sector "is all 0xFF?" flag, used to color erased
    /// sectors distinctly from cold-but-written ones. Recomputed on
    /// each program/erase trap from the live image.
    pub sector_erased: Vec<bool>,
}

#[derive(Properties, PartialEq)]
pub struct W25q32PanelProps {
    pub snapshot: W25q32Snapshot,
    /// Fires when the user picks a file. `main.rs` reads the bytes
    /// and calls `replace_image` + IDB save.
    pub on_upload: Callback<File>,
    /// Fires when the user clicks Reset / Chip Erase. `main.rs`
    /// calls `erase_chip` + IDB delete + repaints sector flags.
    pub on_reset: Callback<()>,
}

#[function_component(W25q32Panel)]
pub fn w25q32_panel(props: &W25q32PanelProps) -> Html {
    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        let snap = props.snapshot.clone();
        use_effect_with(snap, move |snap| {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                draw_heatmap(&canvas, snap);
            }
            || ()
        });
    }

    let s = &props.snapshot;

    let on_file = {
        let on_upload = props.on_upload.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                && let Some(files) = input.files()
                && let Some(file) = files.get(0)
            {
                on_upload.emit(file);
                input.set_value("");
            }
        })
    };

    let on_reset_click = {
        let on_reset = props.on_reset.clone();
        Callback::from(move |_: MouseEvent| on_reset.emit(()))
    };

    let jedec_str = format!(
        "{:02X} {:02X} {:02X}",
        s.jedec_id[0], s.jedec_id[1], s.jedec_id[2]
    );
    let addr_str = match s.last_accessed_address {
        Some(a) => format!("0x{a:06X}"),
        None => "--".to_string(),
    };

    let canvas_w = HEATMAP_COLS * HEATMAP_SCALE;
    let canvas_h = HEATMAP_ROWS * HEATMAP_SCALE;

    html! {
        <div style="background:#11111b; padding:8px; border-radius:4px; \
                    border:1px solid #313244; display:flex; flex-direction:column; gap:6px;">
            <div style="display:flex; justify-content:space-between; align-items:baseline;">
                <span style="color:#cdd6f4; font-weight:600; font-size:0.85rem;">
                    {"NOR Flash (W25Q32, 4 MiB)"}
                </span>
                <span style="color:#a6adc8; font-family:monospace; font-size:0.75rem;">
                    {format!("SPI \u{00b7} CS={}", s.cs)}
                </span>
            </div>
            <div style="display:flex; gap:12px; font-family:monospace; font-size:0.75rem; \
                        color:#bac2de;">
                <span>{"JEDEC: "}<span style="color:#a6e3a1;">{jedec_str}</span></span>
                <span>
                    {"WIP: "}
                    <span style={if s.wip { "color:#f38ba8;" } else { "color:#6c7086;" }}>
                        {if s.wip { "busy" } else { "idle" }}
                    </span>
                </span>
                <span>
                    {"WEL: "}
                    <span style={if s.wel { "color:#fab387;" } else { "color:#6c7086;" }}>
                        {if s.wel { "set" } else { "clear" }}
                    </span>
                </span>
                <span style="margin-left:auto;">
                    {"@ "}<span style="color:#cba6f7;">{addr_str}</span>
                </span>
            </div>
            <div style="background:#0a0a0a; padding:2px; border-radius:2px; border:1px solid #313244;">
                <canvas ref={canvas_ref}
                        width={canvas_w.to_string()}
                        height={canvas_h.to_string()}
                        style="display:block; image-rendering:pixelated;" />
            </div>
            <div style="display:flex; gap:8px; align-items:center;">
                <label style="color:#bac2de; font-size:0.75rem; cursor:pointer; \
                              padding:4px 8px; background:#1e1e2e; border-radius:3px; \
                              border:1px solid #313244;">
                    {"Load image"}
                    <input type="file"
                           accept=".bin,application/octet-stream"
                           onchange={on_file}
                           style="display:none;" />
                </label>
                <button onclick={on_reset_click}
                        style="color:#bac2de; font-size:0.75rem; cursor:pointer; \
                               padding:4px 8px; background:#1e1e2e; \
                               border:1px solid #313244; border-radius:3px;">
                    {"Chip Erase"}
                </button>
            </div>
            <div style="color:#6c7086; font-size:0.7rem;">
                {"each pixel = 4 KiB sector \u{00b7} warm = recent write \u{00b7} pale = erased (0xFF) \u{00b7} image persists in IndexedDB"}
            </div>
        </div>
    }
}

fn draw_heatmap(canvas: &HtmlCanvasElement, snap: &W25q32Snapshot) {
    let Ok(Some(ctx_object)) = canvas.get_context("2d") else {
        return;
    };
    let Ok(ctx) = ctx_object.dyn_into::<CanvasRenderingContext2d>() else {
        return;
    };

    let scale = f64::from(HEATMAP_SCALE);
    // Background: faint grid color so empty cells aren't pure black.
    ctx.set_fill_style_str("#181825");
    ctx.fill_rect(
        0.0,
        0.0,
        f64::from(HEATMAP_COLS * HEATMAP_SCALE),
        f64::from(HEATMAP_ROWS * HEATMAP_SCALE),
    );

    let sectors = snap.sector_activity.len().min(SECTOR_COUNT);
    let erased_len = snap.sector_erased.len();
    for i in 0..sectors {
        let row = (i as u32) / HEATMAP_COLS;
        let col = (i as u32) % HEATMAP_COLS;
        let activity = snap.sector_activity[i];
        let erased = i < erased_len && snap.sector_erased[i];
        let color = sector_color(activity, erased);
        ctx.set_fill_style_str(&color);
        ctx.fill_rect(f64::from(col) * scale, f64::from(row) * scale, scale, scale);
    }
}

/// Map (activity_heat, erased) to a CSS color string.
///
/// - Activity 0 + erased: pale yellow ("explicitly cleared, untouched lately").
/// - Activity 0 + not erased: dim slate ("written, since cooled").
/// - Activity > 0: warm gradient from amber (low) to red (high) so the
///   eye tracks recent writes; the erased flag is moot once a write
///   has landed on the sector.
fn sector_color(activity: u8, erased: bool) -> String {
    if activity == 0 {
        if erased {
            "#3a3a2a".to_string()
        } else {
            "#313244".to_string()
        }
    } else {
        // Linear ramp on red, fading green. activity = 255 -> #ff3030;
        // activity = 64 -> #ff9050 (amber-ish).
        let red: u32 = 0xFF;
        let green: u32 = u32::from(255 - activity) * 6 / 10 + 30;
        let blue: u32 = u32::from(255 - activity) * 3 / 10;
        format!("#{:02x}{:02x}{:02x}", red, green.min(0xFF), blue.min(0xFF))
    }
}
