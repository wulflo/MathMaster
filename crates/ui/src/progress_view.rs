// SPDX-License-Identifier: AGPL-3.0-or-later
//! Paneles de progreso, repaso espaciado y diagnóstico.
//!
//! Leen el estado persistido y se vuelven a evaluar cuando `refresh` cambia, de modo que
//! un drill completado en la pestaña de entrenamiento actualiza estas vistas sin duplicar
//! el estado entre componentes.

#![allow(missing_docs)]

#[cfg(target_arch = "wasm32")]
use crate::storage::read_key;
use anzan_content::UiStrings;
#[cfg(target_arch = "wasm32")]
use anzan_engine::decode_log;
use anzan_engine::{
    DAY_MS, PedagogicalCase, ProgressState, ReviewState, SessionRecord, case_stats,
};
use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
const PROGRESS_KEY: &str = "anzan.progress";
#[cfg(target_arch = "wasm32")]
const SESSIONS_KEY: &str = "anzan.sessions";
#[cfg(target_arch = "wasm32")]
const REVIEW_KEY_PREFIX: &str = "anzan.review.";

#[cfg(target_arch = "wasm32")]
fn load_progress() -> ProgressState {
    read_key(PROGRESS_KEY)
        .and_then(|raw| ProgressState::decode(&raw))
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_progress() -> ProgressState {
    ProgressState::default()
}

#[cfg(target_arch = "wasm32")]
fn load_sessions() -> Vec<SessionRecord> {
    read_key(SESSIONS_KEY).map_or_else(Vec::new, |raw| decode_log(&raw))
}

#[cfg(not(target_arch = "wasm32"))]
fn load_sessions() -> Vec<SessionRecord> {
    Vec::new()
}

#[cfg(target_arch = "wasm32")]
fn load_review(level: u8) -> ReviewState {
    read_key(&format!("{REVIEW_KEY_PREFIX}{level}"))
        .and_then(|raw| ReviewState::decode(&raw))
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_review(_level: u8) -> ReviewState {
    ReviewState::default()
}

#[cfg(target_arch = "wasm32")]
fn now_ms() -> u64 {
    js_sys::Date::now().max(0.0) as u64
}

#[cfg(not(target_arch = "wasm32"))]
fn now_ms() -> u64 {
    0
}

fn fill(template: &str, value: u64) -> String {
    template.replace("{n}", &value.to_string())
}

fn case_label(strings: &UiStrings, case: PedagogicalCase) -> String {
    match case {
        PedagogicalCase::Direct => strings.case_direct.clone(),
        PedagogicalCase::Complement5 => strings.case_complement5.clone(),
        PedagogicalCase::Complement10 => strings.case_complement10.clone(),
        PedagogicalCase::Carry => strings.case_carry.clone(),
    }
}

/// Panel de dominio, repaso o diagnóstico según `section` (0, 1 o 2).
#[component]
pub fn ProgressPanel(strings: UiStrings, refresh: RwSignal<u32>, section: usize) -> impl IntoView {
    let progress_text = move || {
        refresh.get();
        let progress = load_progress();
        let spec = progress.spec();
        (
            progress.level,
            (progress.accuracy() * 100.0).round() as u32,
            progress.streak,
            progress.best_streak,
            spec.digits,
            spec.terms,
            progress.attempts,
        )
    };
    let progress_bar = move || {
        let (_, accuracy, _, _, _, _, _) = progress_text();
        accuracy
    };

    match section {
        0 => view! {
            <section class="anzan-sub-card" aria-labelledby="anzan-mastery-title">
                <h3 id="anzan-mastery-title">{strings.mastery_title.clone()}</h3>
                <dl class="anzan-stats">
                    <div><dt>{strings.progress_level.clone()}</dt><dd>{move || progress_text().0.to_string()}</dd></div>
                    <div><dt>{strings.progress_accuracy.clone()}</dt><dd>{move || format!("{}%", progress_text().1)}</dd></div>
                    <div><dt>{strings.progress_streak.clone()}</dt><dd>{move || progress_text().2.to_string()}</dd></div>
                    <div><dt>{strings.progress_best.clone()}</dt><dd>{move || progress_text().3.to_string()}</dd></div>
                    <div><dt>{strings.progress_attempts.clone()}</dt><dd>{move || progress_text().6.to_string()}</dd></div>
                </dl>
                <div
                    class="anzan-mastery-bar"
                    role="progressbar"
                    aria-valuemin="0"
                    aria-valuemax="100"
                    aria-valuenow=move || progress_bar().to_string()
                >
                    <span style=move || format!("width:{}%", progress_bar())></span>
                </div>
                <p>{move || fill(&strings.mastery_goal, u64::from(progress_text().5))}</p>
            </section>
        }
        .into_any(),
        1 => {
            let rows = (1..=6u8)
                .map(|level| {
                    let strings = strings.clone();
                    let detail = move || {
                        refresh.get();
                        let state = load_review(level);
                        let now = now_ms();
                        if state.is_due(now) {
                            strings.review_now.clone()
                        } else {
                            let remaining = state.due_ms.saturating_sub(now);
                            if remaining < DAY_MS {
                                fill(&strings.review_hours, remaining.div_ceil(3_600_000).max(1))
                            } else {
                                fill(&strings.review_days, remaining.div_ceil(DAY_MS))
                            }
                        }
                    };
                    view! {
                        <li>
                            <span>{format!("{}", level)}</span>
                            <span>{detail}</span>
                            <button type="button" class="anzan-review-start">
                                {strings.review_start.clone()}
                            </button>
                        </li>
                    }
                    .into_any()
                })
                .collect_view();
            view! {
                <section class="anzan-sub-card" aria-labelledby="anzan-review-title">
                    <h3 id="anzan-review-title">{strings.review_title.clone()}</h3>
                    <ul class="anzan-review-list">{rows}</ul>
                </section>
            }
            .into_any()
        }
        _ => {
            let labels = strings.clone();
            let case_labels_for_rows = labels.clone();
            let session_labels = labels.clone();
            let case_rows = move || {
                refresh.get();
                let sessions = load_sessions();
                case_stats(&sessions)
                    .into_iter()
                    .map(|stat| {
                        let accuracy = if stat.attempts == 0 {
                            0
                        } else {
                            (stat.accuracy() * 100.0).round() as u32
                        };
                        view! {
                            <div>
                                <dt>{case_label(&case_labels_for_rows, stat.case)}</dt>
                                <dd>{format!("{} {}/{} ({}%)", labels.case_stats_accuracy, stat.correct, stat.attempts, accuracy)}</dd>
                            </div>
                        }
                        .into_any()
                    })
                    .collect_view()
            };
            let session_rows = move || {
                refresh.get();
                load_sessions()
                    .iter()
                    .rev()
                    .take(100)
                    .map(|session| {
                        view! {
                            <li>
                                <span>{session.timestamp_ms.to_string()}</span>
                                <span>{format!("{}", session.level)}</span>
                                <span>{format!("{} {} ms", session_labels.session_elapsed, session.elapsed_ms)}</span>
                                <span>{case_label(&session_labels, session.case)}</span>
                            </li>
                        }
                        .into_any()
                    })
                    .collect_view()
            };
            view! {
                <section class="anzan-sub-card" aria-labelledby="anzan-diagnostics-title">
                    <h3 id="anzan-diagnostics-title">{strings.diagnostics_title.clone()}</h3>
                    <dl class="anzan-case-grid">{case_rows}</dl>
                    <h4>{strings.diagnostics_history.clone()}</h4>
                    <ul class="anzan-session-list">{session_rows}</ul>
                </section>
            }
            .into_any()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_are_filled_with_the_given_number() {
        assert_eq!(fill("En {n} h", 3), "En 3 h");
        assert_eq!(fill("{n}", 0), "0");
    }
}
