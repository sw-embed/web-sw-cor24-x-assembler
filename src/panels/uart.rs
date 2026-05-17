use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement, KeyboardEvent};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct UartPanelProps {
    pub output: AttrValue,
    pub running: bool,
    pub halted: bool,
    pub on_key: Callback<KeyboardEvent>,
}

#[function_component(UartPanel)]
pub fn uart_panel(props: &UartPanelProps) -> Html {
    let log_ref = use_node_ref();

    // Auto-scroll to the bottom whenever the output grows so the
    // latest UART characters stay visible.
    {
        let log_ref = log_ref.clone();
        let output_len = props.output.len();
        use_effect_with(output_len, move |_| {
            if let Some(el) = log_ref.cast::<HtmlElement>() {
                el.set_scroll_top(el.scroll_height());
            }
            || ()
        });
    }

    // Cap the display to the trailing 4 KB of the buffer so a tight
    // read-print loop doesn't grow an unbounded DOM text node.
    const DISPLAY_TAIL_BYTES: usize = 4096;
    let raw = props.output.as_str();
    let display: &str = if raw.len() > DISPLAY_TAIL_BYTES {
        let cut = raw.len() - DISPLAY_TAIL_BYTES;
        let mut i = cut;
        while i < raw.len() && !raw.is_char_boundary(i) {
            i += 1;
        }
        &raw[i..]
    } else {
        raw
    };
    let truncated = raw.len() > DISPLAY_TAIL_BYTES;

    // Keystrokes go through a dedicated text-input element rather than
    // a div+tabindex+onkeydown. The earlier div-based capture was
    // brittle: the 60 Hz tick re-renders the App, and on some flows
    // focus didn't survive to the next key event (the demo never saw
    // the typed bytes). A real <input>:
    //   - is a reliable focus target (clicks always focus it),
    //   - holds focus through re-renders by default,
    //   - lets the user see what they typed (echoing the bytes
    //     visually in the input field), which is the missing
    //     feedback in non-echoing demos like 'I2C RTC Set'.
    //
    // We hook onkeydown on the input rather than oninput so we can
    // route Enter / Backspace through `on_key`'s existing logic
    // unchanged. The input's `value` is left alone — the browser
    // manages it and the user sees their characters accumulate.
    let on_input_keydown = props.on_key.clone();
    let on_input_click = Callback::from(|e: MouseEvent| {
        // Defensive: explicitly focus the input on click. Browsers
        // usually do this for free on type=text, but doing it
        // explicitly avoids any edge case where a parent stops
        // event propagation or focus management is off.
        if let Some(input) = e
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            let _ = input.focus();
        }
    });

    html! {
        <div style="display:flex; flex-direction:column; gap:2px; flex-shrink:0;">
            <div style="color:#bac2de; font-size:0.8rem;">
                {"UART"}
                if props.running {
                    <span style="color:#a6adc8;">{" (output below; type in the input box to send)"}</span>
                }
                if truncated {
                    <span style="color:#6c7086;">
                        {format!(" (showing last {} of {} bytes)", display.len(), raw.len())}
                    </span>
                }
            </div>
            <div ref={log_ref}
                style="background:#11111b; color:#a6e3a1; padding:8px; border-radius:4px; \
                       font-family:monospace; font-size:13px; white-space:pre-wrap; \
                       min-height:40px; max-height:140px; overflow:auto; \
                       border:1px solid transparent; box-sizing:border-box;">
                { if raw.is_empty() && !props.running && !props.halted {
                    html! { <span style="color:#a6adc8;">{"(no output)"}</span> }
                } else {
                    html! { display.to_string() }
                }}
            </div>
            <input type="text"
                   onkeydown={on_input_keydown}
                   onclick={on_input_click}
                   autocomplete="off"
                   spellcheck="false"
                   placeholder={ if props.running { "type here to send to UART RX" } else { "(start a demo to enable input)" } }
                   disabled={!props.running}
                   style="background:#11111b; color:#f5e0dc; padding:6px 8px; \
                          border-radius:4px; border:1px solid #313244; \
                          font-family:monospace; font-size:13px; outline:none; \
                          box-sizing:border-box;" />
        </div>
    }
}
