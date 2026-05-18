//! SPI SD Card device-panel.
//!
//! Surfaces `SdCardDevice` (sw-cor24-emulator's SPI-mode SD card) in
//! the I/O panel. Header strip shows the CS pin, current image size,
//! and last-accessed sector with a brief just-flashed highlight. A
//! file upload widget lets the user replace the on-bus image from
//! their machine; the bytes go through `SdCardHandleExt::replace_image`
//! and (via `main.rs`) get persisted to IndexedDB so they survive
//! page reloads. A Reset button drops the persisted image and
//! re-attaches the bundled default.

use wasm_bindgen::JsCast;
use web_sys::{File, HtmlInputElement};
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdCardSnapshot {
    pub cs: u8,
    pub size_bytes: u32,
    /// 512-byte sector index of the most recent CMD17/CMD24 against
    /// the card, or `None` if no command has landed since attach.
    pub last_accessed_sector: Option<u32>,
}

#[derive(Properties, PartialEq)]
pub struct SdCardPanelProps {
    pub snapshot: SdCardSnapshot,
    /// Fires when the user picks a file. `main.rs` reads the file
    /// as an ArrayBuffer, calls `replace_image`, and saves to IDB.
    pub on_upload: Callback<File>,
    /// Fires when the user clicks the Reset button. `main.rs`
    /// clears IDB and re-attaches the bundled default image.
    pub on_reset: Callback<()>,
}

#[function_component(SdCardPanel)]
pub fn sdcard_panel(props: &SdCardPanelProps) -> Html {
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
                // Reset the input value so picking the same file
                // twice in a row fires another change event.
                input.set_value("");
            }
        })
    };

    let on_reset_click = {
        let on_reset = props.on_reset.clone();
        Callback::from(move |_: MouseEvent| on_reset.emit(()))
    };

    let size_str = format_size(s.size_bytes);
    let sector_str = match s.last_accessed_sector {
        Some(s) => format!("sector {s}"),
        None => "no access yet".to_string(),
    };

    html! {
        <div style="background:#11111b; padding:8px; border-radius:4px; \
                    border:1px solid #313244; display:flex; flex-direction:column; gap:6px;">
            <div style="display:flex; justify-content:space-between; align-items:baseline;">
                <span style="color:#cdd6f4; font-weight:600; font-size:0.85rem;">
                    {"SD Card"}
                </span>
                <span style="color:#a6adc8; font-family:monospace; font-size:0.75rem;">
                    {format!("SPI \u{00b7} CS={} \u{00b7} {}", s.cs, size_str)}
                </span>
            </div>
            <div style="display:flex; align-items:baseline; gap:8px; font-family:monospace; font-size:0.85rem;">
                <span style="color:#a6e3a1;">{"last:"}</span>
                <span style="color:#bac2de;">{sector_str}</span>
            </div>
            <div style="display:flex; gap:8px; align-items:center;">
                <label style="color:#bac2de; font-size:0.75rem; cursor:pointer; \
                              padding:4px 8px; background:#1e1e2e; border-radius:3px; \
                              border:1px solid #313244;">
                    {"Upload image"}
                    <input type="file"
                           accept=".img,.iso,.bin,application/octet-stream"
                           onchange={on_file}
                           style="display:none;" />
                </label>
                <button onclick={on_reset_click}
                        style="color:#bac2de; font-size:0.75rem; cursor:pointer; \
                               padding:4px 8px; background:#1e1e2e; \
                               border:1px solid #313244; border-radius:3px;">
                    {"Reset to default"}
                </button>
            </div>
            <div style="color:#6c7086; font-size:0.7rem;">
                {"image persists in IndexedDB across reloads"}
            </div>
        </div>
    }
}

fn format_size(bytes: u32) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", f64::from(bytes) / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{} KiB", bytes / 1024)
    } else {
        format!("{bytes} B")
    }
}
