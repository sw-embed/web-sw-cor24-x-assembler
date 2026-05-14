//! web-sw-cor24-x-assembler — initial scaffold.
//!
//! This is a placeholder Yew app. The real work is to extract the
//! assembler live-demo from cor24-rs (being phased out) into this
//! repo, using cor24-assembler for assembling and cor24-emulator
//! for execution.

use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <main style="padding: 2rem;">
            <h1>{ "web-sw-cor24-x-assembler" }</h1>
            <p>{ "Scaffold. The assembler live-demo will land here." }</p>
            <p>
                { "See " }
                <a href="https://github.com/sw-embed/sw-cor24-x-assembler" style="color:#89b4fa;">
                    { "sw-cor24-x-assembler" }
                </a>
                { " for the underlying assembler crate." }
            </p>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
