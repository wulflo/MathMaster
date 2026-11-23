// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pestañas horizontales adosadas con navegación de teclado.
//!
//! El componente es puramente presentacional: recibe la señal de selección del padre, de
//! modo que cualquier vista puede pedir el cambio de pestaña (por ejemplo, el botón
//! «Ver en el soroban» después de un fallo).
//!
//! Los paneles se construyen de forma diferida: cada `PanelBuilder` se invoca la primera
//! vez que se selecciona su pestaña y el view resultante se conserva en el DOM (oculto
//! con `hidden`) para no perder el estado al volver a cambiar de pestaña. Así la carga
//! inicial no monta el árbol completo de las cuatro secciones —importantísimo en móvil—.

#![allow(missing_docs)] // Props generados por `#[component]`.

use std::sync::Arc;

use crate::storage::{read_key, write_key};
use leptos::prelude::*;

/// Prefijo de las claves donde se recuerda la pestaña activa de cada grupo.
const TAB_KEY_PREFIX: &str = "anzan.tab.";

/// Clave de persistencia de un grupo de pestañas.
fn tab_key(id: &str) -> String {
    format!("{TAB_KEY_PREFIX}{id}")
}

/// Normaliza el índice guardado: fuera de rango o ilegible vuelve al valor por defecto.
fn normalize_tab(raw: Option<&str>, count: usize, fallback: usize) -> usize {
    if count == 0 {
        return 0;
    }
    raw.and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|index| *index < count)
        .unwrap_or_else(|| fallback.min(count - 1))
}

/// Señal de selección que arranca en la última pestaña recordada para este grupo.
///
/// Se usa en lugar de `RwSignal::new(...)` en los padres para que recargar la página
/// devuelva al usuario a la sección donde estaba.
pub fn tab_signal(id: &str, count: usize, fallback: usize) -> RwSignal<usize> {
    let stored = read_key(&tab_key(id));
    RwSignal::new(normalize_tab(stored.as_deref(), count, fallback))
}

/// Iniciales mostradas en pantallas estrechas (móvil). Cuando se omite, se usa la
/// etiqueta completa también en móvil.
pub type ShortLabels = Vec<String>;

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

/// Fábrica diferida del contenido de un panel.
pub type PanelBuilder = Arc<dyn Fn() -> AnyView + Send + Sync + 'static>;

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
    /// Fábricas de los paneles, en el mismo orden que `labels`.
    panels: Vec<PanelBuilder>,
    /// Etiquetas cortas (iniciales) para pantallas estrechas; si faltan, se usa la
    /// etiqueta completa en todos los anchos.
    #[prop(optional)]
    short_labels: Option<Vec<String>>,
) -> impl IntoView {
    let count = labels.len();
    let shorts = short_labels.as_deref().unwrap_or(&[]);

    // Un panel se monta la primera vez que se selecciona; a partir de ahí permanece
    // oculto en vez de desmontarse, para conservar el estado de los drills.
    let visited: Vec<RwSignal<bool>> = (0..count)
        .map(|index| RwSignal::new(selected.get_untracked() == index))
        .collect();
    let visited_on_change = visited.clone();
    Effect::new(move || {
        let active = selected.get();
        if let Some(flag) = visited_on_change.get(active)
            && !flag.get_untracked()
        {
            flag.set(true);
        }
    });

    // Recuerda la pestaña activa para restaurarla al recargar o reabrir la app.
    let persist_key = tab_key(id);
    Effect::new(move || {
        write_key(&persist_key, &selected.get().to_string());
    });

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
                    <span class="anzan-tab-label-full">{label}</span>
                    {shorts.get(index).map(|short| {
                        view! {
                            <span class="anzan-tab-label-short" aria-hidden="true">
                                {short.clone()}
                            </span>
                        }
                        .into_any()
                    })}
                    <span class="anzan-tab-indicator" aria-hidden="true"></span>
                </button>
            }
            .into_any()
        })
        .collect_view();

    let bodies = panels
        .into_iter()
        .enumerate()
        .map(|(index, build)| {
            let tab_id = format!("anzan-tab-{id}-{index}");
            let panel_id = format!("anzan-panel-{id}-{index}");
            let flag = visited[index];
            view! {
                <div
                    role="tabpanel"
                    id=panel_id
                    class="anzan-tabpanel"
                    aria-labelledby=tab_id
                    hidden=move || selected.get() != index
                >
                    {move || {
                        if flag.get() {
                            build()
                        } else {
                            // Panel no visitado aún: sin nodos hasta que se abra su pestaña.
                            view! { <span style="display:none"></span> }.into_any()
                        }
                    }}
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
        assert_eq!(navigate("ArrowLeft", 0, 3), Some(2));
        assert_eq!(navigate("Home", 2, 4), Some(0));
        assert_eq!(navigate("End", 1, 4), Some(3));
        assert_eq!(navigate("Enter", 1, 4), None);
        assert_eq!(navigate("ArrowRight", 0, 0), None);
    }

    #[test]
    fn saved_tab_only_restores_indices_inside_the_group() {
        assert_eq!(normalize_tab(None, 4, 2), 2);
        assert_eq!(normalize_tab(Some("1"), 4, 2), 1);
        assert_eq!(normalize_tab(Some(" 3 "), 4, 2), 3);
        assert_eq!(normalize_tab(Some("4"), 4, 2), 2);
        assert_eq!(normalize_tab(Some("-1"), 4, 2), 2);
        assert_eq!(normalize_tab(Some("azul"), 4, 2), 2);
        assert_eq!(normalize_tab(Some("0"), 0, 2), 0);
        assert_eq!(normalize_tab(None, 2, 9), 1);
    }
}
