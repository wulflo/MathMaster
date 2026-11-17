use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            style: "font-family: sans-serif; padding: 24px;",
            h1 { "AnzanLab · spike C" }
            p { "dioxus-native 0.8.0-alpha.3 (blitz/WGPU, sin webview)" }
        }
    }
}
