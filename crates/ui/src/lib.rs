// SPDX-License-Identifier: AGPL-3.0-or-later
//! Componentes de UI de MathMaster.
//!
//! La UI no calcula: recibe estado ya resuelto y lo dibuja. El mismo árbol Leptos se usa
//! en la web (wasm) y dentro de la webview de Tauri.

#![forbid(unsafe_code)]
#![allow(missing_docs)] // Los props generados por `#[component]` no llevan doc individual.

use crate::storage::{read_key, write_key};
use anzan_content::{MultiplicationContent, UiStrings};
use leptos::prelude::*;

pub mod flash;
pub mod multiplication;
pub mod soroban;
mod storage;

pub use flash::FlashAnzan;
pub use multiplication::MultiplicationLearn;
pub use soroban::SorobanBoard;

const STYLE: &str = include_str!("../assets/style.css");

/// Clave propia del acento elegido (índice 0..9).
const ACCENT_KEY: &str = "anzan.accent";
/// Clave propia del tema elegido (`light` u `dark`).
const THEME_KEY: &str = "anzan.theme";
/// Número de acentos de la paleta.
const ACCENT_COUNT: usize = 10;

/// Normaliza el valor guardado del acento: fuera de rango o ilegible vuelve a 0.
fn accent_index(raw: Option<&str>) -> usize {
    raw.and_then(|value| value.trim().parse::<usize>().ok())
        .map_or(0, |value| value.min(ACCENT_COUNT - 1))
}

/// Acento guardado; si no hay dato válido empieza en el primero y reescribe normalizado.
fn initial_accent() -> usize {
    let stored = read_key(ACCENT_KEY);
    let index = accent_index(stored.as_deref());
    if stored.as_deref() != Some(index.to_string().as_str()) {
        write_key(ACCENT_KEY, &index.to_string());
    }
    index
}

/// Interpreta el tema guardado; `None` si falta o no es válido.
fn stored_theme(raw: Option<&str>) -> Option<bool> {
    match raw.map(str::trim) {
        Some("dark") => Some(true),
        Some("light") => Some(false),
        _ => None,
    }
}

/// Preferencia del sistema; en nativo siempre es tema claro.
#[cfg(target_arch = "wasm32")]
fn system_prefers_dark() -> bool {
    web_sys::window()
        .and_then(|window| {
            window
                .match_media("(prefers-color-scheme: dark)")
                .ok()
                .flatten()
        })
        .is_some_and(|query| query.matches())
}

#[cfg(not(target_arch = "wasm32"))]
fn system_prefers_dark() -> bool {
    false
}

/// Tema inicial: el guardado o, si no hay, la preferencia del sistema.
fn initial_dark() -> bool {
    stored_theme(read_key(THEME_KEY).as_deref()).unwrap_or_else(system_prefers_dark)
}

/// Aplica el tema al documento para que el CSS lo use en `:root[data-theme]`.
#[cfg(target_arch = "wasm32")]
fn apply_theme(dark: bool) {
    let Some(root) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.document_element())
    else {
        return;
    };
    let _ = root.set_attribute("data-theme", if dark { "dark" } else { "light" });
}

#[cfg(not(target_arch = "wasm32"))]
fn apply_theme(_dark: bool) {}

/// Raíz visual de MathMaster.
#[component]
pub fn App(strings: UiStrings, multiplication: MultiplicationContent) -> impl IntoView {
    let strings_for_board = strings.clone();
    let strings_for_flash = strings.clone();
    let strings_for_multiplication = strings.clone();
    let UiStrings {
        app_name,
        kicker,
        tagline,
        accent_picker_label,
        accent_names,
        theme_label,
        theme_light,
        theme_dark,
        spanish,
        english,
        source_label,
        license_label,
        separator,
        ..
    } = strings;

    let accent = RwSignal::new(initial_accent());
    let dark = RwSignal::new(initial_dark());
    apply_theme(dark.get_untracked());
    let names_for_swatches = accent_names.clone();
    let picker_for_swatches = accent_picker_label.clone();
    let swatches = (0..ACCENT_COUNT)
        .map(|index| {
            let name = names_for_swatches
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("{} {}", picker_for_swatches, index + 1));
            view! {
                <button
                    type="button"
                    class="anzan-accent-swatch"
                    class:is-active=move || accent.get() == index
                    style=format!("background: var(--anzan-accent-{});", index + 1)
                    aria-label=name
                    aria-pressed=move || if accent.get() == index { "true" } else { "false" }
                    on:click=move |_| {
                        accent.set(index);
                        write_key(ACCENT_KEY, &index.to_string());
                    }
                ></button>
            }
            .into_any()
        })
        .collect_view();

    let theme_name = move || {
        if dark.get() {
            theme_dark.clone()
        } else {
            theme_light.clone()
        }
    };
    let on_theme = move |_| {
        let next = !dark.get_untracked();
        dark.set(next);
        apply_theme(next);
        write_key(THEME_KEY, if next { "dark" } else { "light" });
    };

    view! {
        <style>{STYLE}</style>
        <main
            class="anzan-root"
            style=move || format!("--anzan-accent: var(--anzan-accent-{});", accent.get() + 1)
        >
            <section class="anzan-card" aria-labelledby="anzan-title">
                <p class="anzan-kicker">{kicker}</p>
                <h1 id="anzan-title">{app_name}</h1>
                <p class="anzan-tagline">{tagline}</p>
                <div class="anzan-appearance">
                    <div
                        class="anzan-accent-picker"
                        role="group"
                        aria-label=accent_picker_label.clone()
                    >
                        <span class="anzan-control-label">{accent_picker_label.clone()}</span>
                        <div class="anzan-accent-options">{swatches}</div>
                    </div>
                    <div class="anzan-theme-picker" role="group" aria-label=theme_label.clone()>
                        <span class="anzan-control-label">{theme_label.clone()}</span>
                        <button
                            type="button"
                            class="anzan-theme-toggle"
                            role="switch"
                            aria-checked=move || if dark.get() { "true" } else { "false" }
                            on:click=on_theme
                        >
                            {theme_name}
                        </button>
                    </div>
                </div>
                <SorobanBoard strings=strings_for_board />
                <MultiplicationLearn
                    strings=strings_for_multiplication
                    content=multiplication
                />
                <FlashAnzan strings=strings_for_flash />
                <footer class="anzan-footer">
                    <nav class="anzan-lang" aria-label="Language">
                        <a href="?lang=es">{spanish}</a>
                        <span class="anzan-sep">{separator.clone()}</span>
                        <a href="?lang=en">{english}</a>
                    </nav>
                    <nav class="anzan-links">
                        <a href="https://github.com/wulflo/MathMaster" target="_blank" rel="noreferrer">{source_label}</a>
                        <span class="anzan-sep">{separator.clone()}</span>
                        <a href="https://github.com/wulflo/MathMaster/blob/main/LICENSE" target="_blank" rel="noreferrer">{license_label}</a>
                    </nav>
                </footer>
            </section>
        </main>
    }
}

/// Monta la aplicación en el `body` del documento (sólo wasm).
#[cfg(target_arch = "wasm32")]
pub fn mount_to_body(strings: UiStrings, multiplication: MultiplicationContent) {
    leptos::mount::mount_to_body(move || {
        let strings = strings.clone();
        let multiplication = multiplication.clone();
        view! { <App strings=strings multiplication=multiplication /> }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accent_index_normalizes_stored_values() {
        assert_eq!(accent_index(None), 0);
        assert_eq!(accent_index(Some("")), 0);
        assert_eq!(accent_index(Some("3")), 3);
        assert_eq!(accent_index(Some(" 9 ")), 9);
        assert_eq!(accent_index(Some("10")), 9);
        assert_eq!(accent_index(Some("-1")), 0);
        assert_eq!(accent_index(Some("azul")), 0);
    }

    #[test]
    fn stored_theme_only_accepts_known_values() {
        assert_eq!(stored_theme(Some("dark")), Some(true));
        assert_eq!(stored_theme(Some(" light ")), Some(false));
        assert_eq!(stored_theme(Some("auto")), None);
        assert_eq!(stored_theme(Some("")), None);
        assert_eq!(stored_theme(None), None);
    }

    #[test]
    fn accent_defaults_without_a_host() {
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(initial_accent(), 0);
    }
}
