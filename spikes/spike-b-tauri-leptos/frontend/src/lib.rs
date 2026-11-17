use leptos::prelude::*;
use wasm_bindgen::prelude::*;

const DIGIT_STYLE: &str = "position:fixed;inset:0;display:flex;align-items:center;justify-content:center;\
font:700 22vmin/1 ui-monospace,'DejaVu Sans Mono',monospace;color:#e8e8e8;visibility:hidden;\
background:transparent;user-select:none;pointer-events:none;";

const HUD_STYLE: &str = "position:fixed;left:0;right:0;bottom:12px;text-align:center;\
font:400 13px/1.5 system-ui,sans-serif;color:#8a8a8a;pointer-events:none;";

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <div style="position:fixed;inset:0;background:#111;overflow:hidden;">
            <div id="d0" style=DIGIT_STYLE>"7"</div>
            <div id="d1" style=DIGIT_STYLE>"9"</div>
            <div id="hud" style=HUD_STYLE>
                "spike B · Tauri 2 + Leptos CSR · estímulos de dígitos sobre fondo estable"
            </div>
        </div>
    }
}
