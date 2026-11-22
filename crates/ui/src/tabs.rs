// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pestañas horizontales adosadas con navegación de teclado.
//!
//! El componente es puramente presentacional: recibe la señal de selección del padre, de
//! modo que cualquier vista puede pedir el cambio de pestaña (por ejemplo, el botón
//! «Ver en el soroban» después de un fallo).

#![allow(missing_docs)] // Props generados por `#[component]`.

use leptos::prelude::*;

/// Enfoca una pestaña por su identificador; en nativo no hay documento que enfocar.
#[cfg(target_arch = "wasm32")]
fn focus_tab(id: &str) {
    use wasm_bindgen::JsCast;

    let _ = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(id))
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
        .map(|element| element.focus());
}

/// Enfoca una pestaña por su identificador; en nativo no hay documento que enfocar.
#[cfg(not(target_arch = "wasm32"))]
fn focus_tab(_id: &str) {}

/// Índice siguiente según la tecla pulsada, con envolvente.
fn navigate(key: &str, current: usize, count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    match key {
        "ArrowRight" => Some((current + 1) % count),
        "ArrowLeft" => Some((current + count - 1) % count),
        "Home" => Some(0),
        "End" => Some(count - 1),
        _ => None,
    }
}

/// Barra de pestañas y sus paneles.
#[component]
pub fn Tabs(
    /// Prefijo único de identificadores para este grupo.
    id: &'static str,
    /// Etiquetas visibles, en orden.
    labels: Vec<String>,
    /// Etiqueta accesible de la barra.
    aria_label: String,
    /// Señal con el índice seleccionado (controlada por el padre).
    selected: RwSignal<usize>,
    /// Contenido de cada panel, en el mismo orden que `labels`.
    panels: Vec<AnyView>,
) -> impl IntoView {
    let count = labels.len();
    let tabs = labels
        .into_iter()
        .enumerate()
        .map(|(index, label)| {
            let tab_id = format!("anzan-tab-{id}-{index}");
            let panel_id = format!("anzan-panel-{id}-{index}");
            let focus_next_id = format!("anzan-tab-{id}-");
            let is_active = move || selected.get() == index;
            view! {
                <button
                    type="button"
                    role="tab"
                    id=tab_id
                    class="anzan-tab"
                    class:is-active=is_active
                    aria-selected=move || if is_active() { "true" } else { "false" }
                    aria-controls=panel_id
                    tabindex=move || if is_active() { 0 } else { -1 }
                    on:click=move |_| selected.set(index)
                    on:keydown=move |event: leptos::ev::KeyboardEvent| {
                        if let Some(next) = navigate(&event.key(), index, count) {
                            event.prevent_default();
                            selected.set(next);
                            focus_tab(&format!("{focus_next_id}{next}"));
                        }
                    }
                >
                    <span class="anzan-tab-label">{label}</span>
                    <span class="anzan-tab-indicator" aria-hidden="true"></span>
                </button>
            }
            .into_any()
        })
        .collect_view();

    let bodies = panels
        .into_iter()
        .enumerate()
        .map(|(index, panel)| {
            let tab_id = format!("anzan-tab-{id}-{index}");
            let panel_id = format!("anzan-panel-{id}-{index}");
            view! {
                <div
                    role="tabpanel"
                    id=panel_id
                    class="anzan-tabpanel"
                    aria-labelledby=tab_id
                    hidden=move || selected.get() != index
                >
                    {panel}
                </div>
            }
            .into_any()
        })
        .collect_view();

    view! {
        <div class="anzan-tabs" data-tabs=id>
            <div class="anzan-tablist" role="tablist" aria-label=aria_label>
                {tabs}
            </div>
            <div class="anzan-tabpanels">{bodies}</div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_keys_wrap_and_home_end_jump() {
        assert_eq!(navigate("ArrowRight", 0, 4), Some(1));
        assert_eq!(navigate("ArrowRight", 3, 4), Some(0));
        assert_eq!(navigate("ArrowLeft", 0, 4), Some(3));
        assert_eq!(navigate("Home", 2, 4), Some(0));
        assert_eq!(navigate("End", 1, 4), Some(3));
        assert_eq!(navigate("Enter", 1, 4), None);
        assert_eq!(navigate("ArrowRight", 0, 0), None);
    }
}
