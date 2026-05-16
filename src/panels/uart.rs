use web_sys::{HtmlElement, KeyboardEvent};
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
    // latest UART characters stay visible. Triggers on every change
    // to `props.output`; the resize-to-content does no work if the
    // div hasn't actually overflowed yet.
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

    html! {
        <div style="flex:1; min-height:80px;">
            <div style="color:#bac2de; font-size:0.8rem; margin-bottom:2px;">
                {"UART"}
                if props.running {
                    <span style="color:#a6adc8;">{" (type here for input)"}</span>
                }
            </div>
            <div ref={log_ref}
                onkeydown={props.on_key.clone()} tabindex="0"
                style="background:#11111b; color:#a6e3a1; padding:8px; border-radius:4px; \
                       font-family:monospace; font-size:13px; white-space:pre-wrap; \
                       min-height:40px; max-height:200px; overflow:auto; \
                       outline:none; cursor:text; \
                       border:1px solid transparent;">
                { if props.output.is_empty() && !props.running && !props.halted {
                    html! { <span style="color:#a6adc8;">{"(no output)"}</span> }
                } else {
                    html! { {props.output.clone()} }
                }}
            </div>
        </div>
    }
}
