// SPDX-License-Identifier: AGPL-3.0-or-later
//! Componentes de UI de MathMaster.
//!
//! La UI no calcula: recibe estado ya resuelto y lo dibuja. El mismo árbol Leptos se usa
//! en la web (wasm) y dentro de la webview de Tauri.

#![forbid(unsafe_code)]
#![allow(missing_docs)] // Los props generados por `#[component]` no llevan doc individual.

use std::sync::Arc;

use crate::storage::{read_key, write_key};
use crate::tabs::{PanelBuilder, tab_signal};
use anzan_content::{MultiplicationContent, UiStrings};
use anzan_engine::ProgressState;
use leptos::prelude::*;

pub mod answer;
pub mod flash;
pub mod fundamentals;
pub mod lab;
pub mod multiplication;
pub mod progress_view;
pub mod soroban;
mod storage;
pub mod tabs;
pub mod training;

pub use flash::FlashAnzan;
pub use fundamentals::FundamentalsView;
pub use lab::{MovementDemo, SorobanLab};
pub use multiplication::{MultiplicationLearn, MultiplicationPractice, Table99View};
pub use progress_view::ProgressPanel;
pub use soroban::SorobanBoard;
pub use tabs::{ShortLabels, Tabs};
pub use training::{DirectedView, MitoriView};

const STYLE: &str = include_str!("../assets/style.css");

/// Clave propia del acento elegido (índice 0..9).
const ACCENT_KEY: &str = "anzan.accent";
/// Clave propia del tema elegido (`light` u `dark`).
const THEME_KEY: &str = "anzan.theme";
/// Clave del progreso compartido con la cabecera.
const PROGRESS_KEY: &str = "anzan.progress";
/// Número de acentos de la paleta.
const ACCENT_COUNT: usize = 10;
/// Pestaña activa por defecto: Entrenamiento, para exponer los controles de práctica.
const DEFAULT_MAIN_TAB: usize = 2;

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

/// Indicador de cabecera: nivel y racha del progreso persistido.
fn initial_stats() -> (u8, u32) {
    read_key(PROGRESS_KEY)
        .and_then(|raw| ProgressState::decode(&raw))
        .map_or((1, 0), |progress| (progress.level, progress.streak))
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

/// Cabecera compacta de la aplicación.
#[component]
fn AppBar(
    app_name: String,
    kicker: String,
    tagline: String,
    progress_level: String,
    progress_streak: String,
    accent_picker_label: String,
    accent_names: Vec<String>,
    theme_label: String,
    theme_light: String,
    theme_dark: String,
    language: String,
    lang_es: String,
    lang_en: String,
    stats: RwSignal<(u8, u32)>,
    accent: RwSignal<usize>,
    dark: RwSignal<bool>,
) -> impl IntoView {
    let fallback_label = accent_picker_label.clone();
    let swatches = (0..ACCENT_COUNT)
        .map(|index| {
            let name = accent_names
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("{fallback_label} {}", index + 1));
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

    let label_attr = accent_picker_label.clone();
    let label_text = accent_picker_label;
    let theme_name = move || {
        if dark.get() {
            theme_dark.clone()
        } else {
            theme_light.clone()
        }
    };

    view! {
        <header class="anzan-appbar">
            <div class="anzan-appbar-brand">
                <p class="anzan-kicker">{kicker}</p>
                <h1 id="anzan-title">{app_name}</h1>
                <p class="anzan-tagline">{tagline}</p>
            </div>
            <div class="anzan-appbar-live" aria-live="polite">
                <span class="anzan-live-chip">
                    <span class="anzan-live-label">{progress_level}</span>
                    <strong class="anzan-live-value">{stats.get().0}</strong>
                </span>
                <span class="anzan-live-chip">
                    <span class="anzan-live-label">{progress_streak}</span>
                    <strong class="anzan-live-value">{stats.get().1}</strong>
                </span>
            </div>
            <div class="anzan-appbar-controls">
                <div class="anzan-accent-picker" role="group" aria-label=label_attr>
                    <span class="anzan-control-label">{label_text}</span>
                    <div class="anzan-accent-options">{swatches}</div>
                </div>
                <button
                    type="button"
                    class="anzan-theme-toggle"
                    role="switch"
                    aria-label=theme_label
                    aria-checked=move || if dark.get() { "true" } else { "false" }
                    on:click=move |_| {
                        let next = !dark.get_untracked();
                        dark.set(next);
                        apply_theme(next);
                        write_key(THEME_KEY, if next { "dark" } else { "light" });
                    }
                >
                    {theme_name}
                </button>
                <nav class="anzan-lang-compact" aria-label=language>
                    <a href="?lang=es" hreflang="es">{lang_es}</a>
                    <a href="?lang=en" hreflang="en">{lang_en}</a>
                </nav>
            </div>
        </header>
    }
}

/// Pestañas de aprendizaje: fundamentos, multiplicación, atajos y tabla 大九九.
#[component]
fn LearnTabs(strings: UiStrings, multiplication: MultiplicationContent) -> impl IntoView {
    let selected = tab_signal("learn", 4, 0);
    let fundamentals = multiplication.fundamentals.clone();
    let panels: Vec<PanelBuilder> = vec![
        Arc::new({
            let strings = strings.clone();
            let fundamentals = fundamentals.clone();
            move || {
                view! { <FundamentalsView strings=strings.clone() content=fundamentals.clone() /> }
                    .into_any()
            }
        }),
        Arc::new({
            let strings = strings.clone();
            let multiplication = multiplication.clone();
            move || {
                view! {
                    <MultiplicationLearn strings=strings.clone() content=multiplication.clone() kind=1 />
                }
                .into_any()
            }
        }),
        Arc::new({
            let strings = strings.clone();
            let multiplication = multiplication.clone();
            move || {
                view! {
                    <MultiplicationLearn strings=strings.clone() content=multiplication.clone() kind=2 />
                }
                .into_any()
            }
        }),
        Arc::new({
            let strings = strings.clone();
            move || view! { <Table99View strings=strings.clone() /> }.into_any()
        }),
    ];
    view! {
        <Tabs
            id="learn"
            labels=vec![
                strings.sub_fundamentals.clone(),
                strings.sub_multiplication.clone(),
                strings.sub_shortcuts.clone(),
                strings.sub_table99.clone(),
            ]
            aria_label=strings.subtablist_label
            selected=selected
            panels=panels
            short_labels=vec![
                "F".into(),
                "M".into(),
                "A".into(),
                "T".into(),
            ]
        />
    }
}

/// Pestañas del laboratorio: ábaco libre y demostrador de movimientos.
#[component]
fn LabTabs(
    strings: UiStrings,
    selected: RwSignal<usize>,
    demo_request: RwSignal<Option<(u32, bool, u32)>>,
) -> impl IntoView {
    let panels: Vec<PanelBuilder> = vec![
        Arc::new({
            let strings = strings.clone();
            move || view! { <SorobanLab strings=strings.clone() /> }.into_any()
        }),
        Arc::new({
            let strings = strings.clone();
            move || {
                view! { <MovementDemo strings=strings.clone() demo_request=demo_request /> }
                    .into_any()
            }
        }),
    ];
    view! {
        <Tabs
            id="lab"
            labels=vec![strings.sub_soroban.clone(), strings.sub_movements.clone()]
            aria_label=strings.subtablist_label
            selected=selected
            panels=panels
            short_labels=vec!["S".into(), "M".into()]
        />
    }
}

/// Pestañas de entrenamiento: Flash, Mitori, multiplicación y dirigido.
#[component]
fn TrainTabs(
    strings: UiStrings,
    stats: RwSignal<(u8, u32)>,
    refresh: RwSignal<u32>,
    demo_request: RwSignal<Option<(u32, bool, u32)>>,
) -> impl IntoView {
    let selected = tab_signal("train", 4, 0);
    let panels: Vec<PanelBuilder> = vec![
        Arc::new({
            let strings = strings.clone();
            move || {
                view! {
                    <FlashAnzan strings=strings.clone() header_stats=stats refresh=refresh demo_request=demo_request />
                }
                .into_any()
            }
        }),
        Arc::new({
            let strings = strings.clone();
            move || {
                view! { <MitoriView strings=strings.clone() demo_request=demo_request /> }
                    .into_any()
            }
        }),
        Arc::new({
            let strings = strings.clone();
            move || view! { <MultiplicationPractice strings=strings.clone() /> }.into_any()
        }),
        Arc::new({
            let strings = strings.clone();
            move || {
                view! { <DirectedView strings=strings.clone() demo_request=demo_request /> }
                    .into_any()
            }
        }),
    ];
    view! {
        <Tabs
            id="train"
            labels=vec![
                strings.sub_flash.clone(),
                strings.sub_mitori.clone(),
                strings.sub_mul_training.clone(),
                strings.sub_directed.clone(),
            ]
            aria_label=strings.subtablist_label
            selected=selected
            panels=panels
            short_labels=vec!["F".into(), "M".into(), "M".into(), "D".into()]
        />
    }
}

/// Pestañas de progreso: dominio, repaso y diagnóstico.
#[component]
fn ProgressTabs(strings: UiStrings, refresh: RwSignal<u32>) -> impl IntoView {
    let selected = tab_signal("progress", 3, 0);
    let panels = (0..3)
        .map(|section| {
            let strings = strings.clone();
            Arc::new(move || {
                view! { <ProgressPanel strings=strings.clone() refresh=refresh section=section /> }
                    .into_any()
            }) as PanelBuilder
        })
        .collect::<Vec<_>>();
    view! {
        <Tabs
            id="progress"
            labels=vec![
                strings.sub_mastery.clone(),
                strings.sub_review.clone(),
                strings.sub_diagnostics.clone(),
            ]
            aria_label=strings.subtablist_label
            selected=selected
            panels=panels
            short_labels=vec!["M".into(), "R".into(), "D".into()]
        />
    }
}

/// Raíz visual de MathMaster.
#[component]
pub fn App(strings: UiStrings, multiplication: MultiplicationContent) -> impl IntoView {
    let UiStrings {
        app_name,
        kicker,
        tagline,
        accent_picker_label,
        accent_names,
        theme_label,
        theme_light,
        theme_dark,
        language,
        lang_es,
        lang_en,
        progress_level,
        progress_streak,
        source_label,
        license_label,
        separator,
        ..
    } = strings.clone();

    let accent = RwSignal::new(initial_accent());
    let dark = RwSignal::new(initial_dark());
    apply_theme(dark.get_untracked());

    let stats = RwSignal::new(initial_stats());
    let refresh = RwSignal::new(0u32);
    let main_tab = tab_signal("main", 4, DEFAULT_MAIN_TAB);
    let lab_tab = tab_signal("lab", 2, 0);
    let demo_request = RwSignal::new(None::<(u32, bool, u32)>);
    Effect::new(move || {
        if demo_request.get().is_some() {
            main_tab.set(1);
            lab_tab.set(1);
        }
    });
    let main_panels: Vec<PanelBuilder> = vec![
        Arc::new({
            let strings = strings.clone();
            let multiplication = multiplication.clone();
            move || {
                view! { <LearnTabs strings=strings.clone() multiplication=multiplication.clone() /> }
                    .into_any()
            }
        }),
        Arc::new({
            let strings = strings.clone();
            move || {
                view! { <LabTabs strings=strings.clone() selected=lab_tab demo_request=demo_request /> }
                    .into_any()
            }
        }),
        Arc::new({
            let strings = strings.clone();
            move || {
                view! {
                    <TrainTabs strings=strings.clone() stats=stats refresh=refresh demo_request=demo_request />
                }
                .into_any()
            }
        }),
        Arc::new({
            let strings = strings.clone();
            move || view! { <ProgressTabs strings=strings.clone() refresh=refresh /> }.into_any()
        }),
    ];

    view! {
        <style>{STYLE}</style>
        <main
            class="anzan-root"
            style=move || format!("--anzan-accent: var(--anzan-accent-{});", accent.get() + 1)
        >
            <AppBar
                app_name=app_name
                kicker=kicker
                tagline=tagline
                progress_level=progress_level
                progress_streak=progress_streak
                accent_picker_label=accent_picker_label
                accent_names=accent_names
                theme_label=theme_label
                theme_light=theme_light
                theme_dark=theme_dark
                language=language.clone()
                lang_es=lang_es
                lang_en=lang_en
                stats=stats
                accent=accent
                dark=dark
            />
            <Tabs
                id="main"
                labels=vec![
                    strings.tab_learn.clone(),
                    strings.tab_lab.clone(),
                    strings.tab_train.clone(),
                    strings.tab_progress.clone(),
                ]
                aria_label=strings.tablist_label.clone()
                selected=main_tab
                panels=main_panels
                short_labels=vec!["A".into(), "L".into(), "E".into(), "P".into()]
            />
            <footer class="anzan-footer">
                <nav class="anzan-lang" aria-label=language>
                    <a href="?lang=es">{strings.spanish.clone()}</a>
                    <span class="anzan-sep">{separator.clone()}</span>
                    <a href="?lang=en">{strings.english.clone()}</a>
                </nav>
                <nav class="anzan-links">
                    <a href="https://github.com/wulflo/MathMaster" target="_blank" rel="noreferrer">{source_label}</a>
                    <span class="anzan-sep">{separator}</span>
                    <a href="https://github.com/wulflo/MathMaster/blob/main/LICENSE" target="_blank" rel="noreferrer">{license_label}</a>
                </nav>
            </footer>
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
