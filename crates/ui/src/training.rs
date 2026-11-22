// SPDX-License-Identifier: AGPL-3.0-or-later
//! Modos de entrenamiento avanzados: Mitori Anzan y práctica dirigida por debilidad.

#![allow(missing_docs)]

use crate::answer::{Action, AnswerBox};
#[cfg(target_arch = "wasm32")]
use crate::storage::read_key;
use anzan_content::UiStrings;
#[cfg(target_arch = "wasm32")]
use anzan_engine::decode_log;
use anzan_engine::{
    FocusedDrill, MitoriConfig, MitoriDrill, PedagogicalCase, SessionRecord, case_stats,
};
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
type TimerClosure = wasm_bindgen::closure::Closure<dyn FnMut()>;
#[cfg(not(target_arch = "wasm32"))]
type TimerClosure = ();

#[cfg(target_arch = "wasm32")]
const SESSIONS_KEY: &str = "anzan.sessions";

#[cfg(target_arch = "wasm32")]
fn load_sessions() -> Vec<SessionRecord> {
    read_key(SESSIONS_KEY).map_or_else(Vec::new, |raw| decode_log(&raw))
}

#[cfg(not(target_arch = "wasm32"))]
fn load_sessions() -> Vec<SessionRecord> {
    Vec::new()
}

#[cfg(target_arch = "wasm32")]
fn seed() -> u64 {
    js_sys::Date::now().to_bits()
}

#[cfg(not(target_arch = "wasm32"))]
fn seed() -> u64 {
    1
}

/// Caso con menor precisión histórica; `None` si no hay datos bastante.
#[must_use]
pub fn weakest_case(sessions: &[SessionRecord]) -> Option<PedagogicalCase> {
    case_stats(sessions)
        .into_iter()
        .filter(|stat| stat.attempts > 0)
        .min_by(|left, right| {
            left.accuracy()
                .partial_cmp(&right.accuracy())
                .unwrap_or(core::cmp::Ordering::Equal)
        })
        .map(|stat| stat.case)
}

/// Etiqueta legible de un caso pedagógico.
fn case_label(strings: &UiStrings, case: PedagogicalCase) -> String {
    match case {
        PedagogicalCase::Direct => strings.case_direct.clone(),
        PedagogicalCase::Complement5 => strings.case_complement5.clone(),
        PedagogicalCase::Complement10 => strings.case_complement10.clone(),
        PedagogicalCase::Carry => strings.case_carry.clone(),
    }
}

/// Operación representativa para visualizar una regla en el demostrador.
fn demo_for(case: PedagogicalCase) -> (u32, bool, u32) {
    match case {
        PedagogicalCase::Direct => (1, true, 1),
        PedagogicalCase::Complement5 => (1, true, 4),
        PedagogicalCase::Complement10 => (9, true, 1),
        PedagogicalCase::Carry => (99, true, 1),
    }
}

/// Mitori Anzan (見取暗算): columnas estáticas con cronómetro y lectura guiada.
#[component]
pub fn MitoriView(
    strings: UiStrings,
    /// Petición de resolución en el soroban.
    #[prop(optional)]
    demo_request: Option<RwSignal<Option<(u32, bool, u32)>>>,
) -> impl IntoView {
    let digits = RwSignal::new(3u8);
    let rows = RwSignal::new(8u8);
    let limit = RwSignal::new(60u16);
    let guided = RwSignal::new(false);
    let visible = RwSignal::new(8usize);
    let drill = RwSignal::new(None::<MitoriDrill>);
    let answer = RwSignal::new(String::new());
    let expected = RwSignal::new(None::<u128>);
    let auto_advance = RwSignal::new(true);
    let elapsed = RwSignal::new(0u64);
    let timer: Rc<RefCell<Option<(i32, TimerClosure)>>> = Rc::new(RefCell::new(None));

    let stop_timer = {
        let timer = timer.clone();
        move || {
            if let Some((handle, _)) = timer.borrow_mut().take() {
                #[cfg(target_arch = "wasm32")]
                if let Some(window) = web_sys::window() {
                    window.clear_interval_with_handle(handle);
                }
                #[cfg(not(target_arch = "wasm32"))]
                let _ = handle;
            }
        }
    };

    let start: Rc<dyn Fn()> = Rc::new({
        let stop_timer = stop_timer.clone();
        move || {
            let config = MitoriConfig {
                digits: digits.get_untracked(),
                rows: rows.get_untracked(),
                time_limit_s: limit.get_untracked(),
            };
            let Ok(new_drill) = MitoriDrill::generate(config, seed()) else {
                return;
            };
            visible.set(if guided.get_untracked() {
                2
            } else {
                usize::from(config.rows)
            });
            expected.set(Some(new_drill.answer()));
            answer.set(String::new());
            elapsed.set(0);
            drill.set(Some(new_drill));
            stop_timer();

            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsCast;
                use wasm_bindgen::closure::Closure;

                if let Some(window) = web_sys::window() {
                    let tick = Rc::new(RefCell::new(0u64));
                    let counted = elapsed;
                    let max = u64::from(limit.get_untracked()) * 1000;
                    let closure = Closure::wrap(Box::new(move || {
                        let mut value = tick.borrow_mut();
                        *value += 1000;
                        counted.set(*value);
                        if max > 0 && *value >= max {
                            *value = max;
                        }
                    }) as Box<dyn FnMut()>);
                    if let Ok(handle) = window
                        .set_interval_with_callback_and_timeout_and_arguments_0(
                            closure.as_ref().unchecked_ref(),
                            1000,
                        )
                    {
                        *timer.borrow_mut() = Some((handle, closure));
                    }
                }
            }
        }
    });

    let on_start = {
        let start = start.clone();
        move |_| start()
    };
    let on_next = Action::new(move || start());
    let show_solution = move || {
        if let Some(request) = demo_request {
            request.set(Some((
                0,
                true,
                u32::try_from(expected.get_untracked().unwrap_or(0)).unwrap_or(u32::MAX),
            )));
        }
    };
    let on_guided = move |event: leptos::ev::Event| guided.set(event_target_checked(&event));

    let columns = move || {
        let Some(current) = drill.get() else {
            return Vec::new();
        };
        let all = current.columns();
        let shown = visible.get().min(all.len());
        all[..shown].to_vec()
    };
    let timer_strings = strings.clone();
    let timer_text = move || {
        timer_strings
            .mitori_timer
            .replace("{ms}", &elapsed.get().to_string())
    };
    let can_reveal = move || {
        drill
            .get()
            .is_some_and(|current| visible.get() < usize::from(current.config().rows))
    };

    view! {
        <section class="anzan-sub-card anzan-mitori" aria-labelledby="anzan-mitori-title">
            <h3 id="anzan-mitori-title">{strings.mitori_title.clone()}</h3>
            <p>{strings.mitori_intro.clone()}</p>
            <div class="anzan-mitori-controls">
                <label class="anzan-field">
                    <span>{strings.mitori_digits.clone()}</span>
                    <input type="number" min="2" max="6"
                        prop:value=move || digits.get()
                        on:input=move |event| {
                            if let Ok(value) = event_target_value(&event).parse::<u8>()
                                && (2..=6).contains(&value) { digits.set(value); }
                        }
                    />
                </label>
                <label class="anzan-field">
                    <span>{strings.mitori_rows.clone()}</span>
                    <input type="number" min="3" max="15"
                        prop:value=move || rows.get()
                        on:input=move |event| {
                            if let Ok(value) = event_target_value(&event).parse::<u8>()
                                && (3..=15).contains(&value) { rows.set(value); }
                        }
                    />
                </label>
                <label class="anzan-field">
                    <span>{strings.mitori_limit.clone()}</span>
                    <input type="number" min="0" max="600"
                        prop:value=move || limit.get()
                        on:input=move |event| {
                            if let Ok(value) = event_target_value(&event).parse::<u16>()
                                && value <= 600 { limit.set(value); }
                        }
                    />
                </label>
                <label class="anzan-field anzan-check">
                    <input type="checkbox" prop:checked=move || guided.get() on:change=on_guided />
                    <span>{strings.mitori_guided.clone()}</span>
                </label>
                <button type="button" class="anzan-mitori-start" on:click=on_start>
                    {strings.mitori_start.clone()}
                </button>
            </div>

            <div class="anzan-mitori-board">
                <ol class="anzan-mitori-column">
                    {move || columns().into_iter().map(|line| view! {
                        <li>{line}</li>
                    }.into_any()).collect_view()}
                </ol>
                <div class="anzan-mitori-side">
                    <output>{timer_text}</output>
                    <button type="button" class="anzan-stage" disabled=move || !can_reveal()
                        on:click=move |_| visible.update(|value| *value += 1)>
                        {strings.mitori_next_row.clone()}
                    </button>
                </div>
            </div>

            <AnswerBox
                strings=strings
                answer=answer
                expected=expected
                active=RwSignal::new(drill.get().is_some())
                auto_advance=auto_advance
                on_correct=Action::new(|| {})
                on_wrong=Action::new(|| {})
                on_next=on_next
                on_show_solution=Action::new(show_solution)
            />
        </section>
    }
}

/// Entrenamiento focalizado en el caso pedagógico de menor precisión.
#[component]
pub fn DirectedView(
    strings: UiStrings,
    /// Petición de resolución en el soroban.
    #[prop(optional)]
    demo_request: Option<RwSignal<Option<(u32, bool, u32)>>>,
) -> impl IntoView {
    let weak = RwSignal::new(weakest_case(&load_sessions()));
    let drill = RwSignal::new(None::<FocusedDrill>);
    let answer = RwSignal::new(String::new());
    let expected = RwSignal::new(None::<u128>);
    let auto_advance = RwSignal::new(true);
    let feedback = RwSignal::new(None::<bool>);

    let start = move || {
        let Some(case) = weak.get_untracked() else {
            return;
        };
        let Some(new_drill) = FocusedDrill::generate(case, 3, 6, seed()) else {
            return;
        };
        expected.set(Some(new_drill.answer()));
        answer.set(String::new());
        feedback.set(None);
        drill.set(Some(new_drill));
    };
    let on_start = move |_| start();
    let on_next = Action::new(start);
    let on_correct = Action::new(move || feedback.set(Some(true)));
    let on_wrong = Action::new(move || feedback.set(Some(false)));
    let show_solution = move || {
        if let Some(request) = demo_request {
            let case = weak.get_untracked().unwrap_or(PedagogicalCase::Direct);
            request.set(Some(demo_for(case)));
        }
    };
    let on_refresh = move |_| weak.set(weakest_case(&load_sessions()));
    let weak_strings = strings.clone();
    let weak_text = move || match weak.get() {
        Some(case) => weak_strings
            .directed_weakest
            .replace("{case}", &case_label(&weak_strings, case)),
        None => weak_strings.directed_no_data.clone(),
    };
    let terms_text = move || match drill.get() {
        Some(current) => current
            .terms()
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(" + "),
        None => String::new(),
    };

    view! {
        <section class="anzan-sub-card" aria-labelledby="anzan-directed-title">
            <h3 id="anzan-directed-title">{strings.directed_title.clone()}</h3>
            <p>{strings.directed_intro.clone()}</p>
            <p role="status">{weak_text}</p>
            <button type="button" class="anzan-stage" on:click=on_refresh>
                {strings.review_start.clone()}
            </button>
            <p class="anzan-directed-terms">{terms_text}</p>
            <button type="button" class="anzan-directed-start" disabled=move || weak.get().is_none()
                on:click=on_start>
                {strings.demo_run.clone()}
            </button>
            <AnswerBox
                strings=strings
                answer=answer
                expected=expected
                active=RwSignal::new(drill.get().is_some())
                auto_advance=auto_advance
                on_correct=on_correct
                on_wrong=on_wrong
                on_next=on_next
                on_show_solution=Action::new(show_solution)
            />
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weakest_case_needs_at_least_one_attempt() {
        assert_eq!(weakest_case(&[]), None);
    }

    #[test]
    fn demo_operations_match_the_rule_examples() {
        assert_eq!(demo_for(PedagogicalCase::Complement5), (1, true, 4));
        assert_eq!(demo_for(PedagogicalCase::Carry), (99, true, 1));
    }
}
