// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sección «Aprender a multiplicar» (空盘前乘法).
//!
//! La UI no calcula: recibe las lecciones ya resueltas del contenido y los pasos del
//! motor. El reproductor sólo avanza o retrocede el índice de paso; el estado de las
//! varillas se lee de `Multiplication::state_after`, sin animaciones por frame.

use crate::storage::{read_key, write_key};
use anzan_content::{MultiplicationContent, MultiplicationLesson, UiStrings};
use anzan_engine::multiplication::{
    MAX_MUL_LEVEL, MIN_MUL_LEVEL, MulProgress, Multiplication, MultiplicationDrill,
    multiplication_level,
};
use leptos::prelude::*;

/// Lección con su multiplicación ya construida (o `None` si el ejemplo no fuera válido).
#[derive(Clone)]
struct Entry {
    lesson: MultiplicationLesson,
    task: Option<Multiplication>,
}

/// Etiquetas de la tabla de productos parciales.
#[derive(Clone)]
struct StepLabels {
    title: String,
    step: String,
    product: String,
    record: String,
    total: String,
}

/// Ábaco de sólo lectura: una varilla por cifra, con bola superior y cuatro inferiores.
fn abacus_view(digits: &[u8], label: &str) -> AnyView {
    let rods = digits
        .iter()
        .map(|digit| {
            let digit = *digit;
            let heaven = digit >= 5;
            let earth = usize::from(digit % 5);
            let beads = (0..4)
                .map(|index| {
                    view! {
                        <span
                            class="anzan-abacus-bead is-earth"
                            class:is-active=(index < earth)
                        ></span>
                    }
                    .into_any()
                })
                .collect_view();
            view! {
                <li class="anzan-abacus-rod">
                    <span class="anzan-abacus-bead is-heaven" class:is-active=heaven></span>
                    <span class="anzan-abacus-beam"></span>
                    {beads}
                    <span class="anzan-abacus-digit">{digit.to_string()}</span>
                </li>
            }
            .into_any()
        })
        .collect_view();
    view! {
        <ol class="anzan-abacus" aria-label=label.to_owned()>
            {rods}
        </ol>
    }
    .into_any()
}

/// Tabla 大九九 completa, con el producto escrito siempre con dos cifras.
fn ninety_nine_table(title: &str, hint: &str, highlight: Option<(u32, u32)>) -> AnyView {
    let head = (1..=9)
        .map(|column| view! { <th scope="col">{column.to_string()}</th> }.into_any())
        .collect_view();
    let rows = (1..=9u32)
        .map(|row| {
            let cells = (1..=9u32)
                .map(|column| {
                    let product = row * column;
                    let selected = highlight == Some((row, column));
                    view! {
                        <td class:is-example=selected>{format!("{product:02}")}</td>
                    }
                    .into_any()
                })
                .collect_view();
            view! {
                <tr>
                    <th scope="row">{row.to_string()}</th>
                    {cells}
                </tr>
            }
            .into_any()
        })
        .collect_view();
    view! {
        <section class="anzan-mul-table" aria-label=title.to_owned()>
            <p class="anzan-mul-table-hint">{hint.to_owned()}</p>
            <div class="anzan-mul-table-scroll">
                <table>
                    <thead>
                        <tr>
                            <th scope="col" class="anzan-mul-table-corner">"×"</th>
                            {head}
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
        </section>
    }
    .into_any()
}

/// Tabla de productos parciales del ejemplo, con la fila actual resaltada.
fn step_table_view(task: &Multiplication, current: usize, labels: &StepLabels) -> AnyView {
    let rows = task
        .steps()
        .iter()
        .enumerate()
        .map(|(index, partial)| {
            let position = index + 1;
            let done = position < current;
            let active = position == current;
            view! {
                <tr class:is-done=done class:is-current=active>
                    <th scope="row">{position.to_string()}</th>
                    <td>{format!("{}×{}", partial.multiplier_digit, partial.multiplicand_digit)}</td>
                    <td class="anzan-mul-record">{format!("{}{}", partial.tens, partial.units)}</td>
                    <td>{partial.running_total.to_string()}</td>
                </tr>
            }
            .into_any()
        })
        .collect_view();
    view! {
        <section class="anzan-mul-steps" aria-labelledby="anzan-mul-steps-title">
            <header class="anzan-mul-steps-head">
                <h3 id="anzan-mul-steps-title">{labels.title.clone()}</h3>
                <p>{format!(
                    "{} × {} = {}",
                    task.multiplicand(),
                    task.multiplier(),
                    task.product()
                )}</p>
            </header>
            <div class="anzan-mul-table-scroll">
                <table class="anzan-mul-step-table">
                    <thead>
                        <tr>
                            <th scope="col">{labels.step.clone()}</th>
                            <th scope="col">{labels.product.clone()}</th>
                            <th scope="col">{labels.record.clone()}</th>
                            <th scope="col">{labels.total.clone()}</th>
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
        </section>
    }
    .into_any()
}

/// Panel de aprendizaje de la multiplicación.
#[component]
pub fn MultiplicationLearn(strings: UiStrings, content: MultiplicationContent) -> impl IntoView {
    let strings_for_practice = strings.clone();
    let UiStrings {
        mul_lessons,
        mul_goal,
        mul_example,
        mul_empty_abacus,
        mul_step_of,
        mul_step_product,
        mul_result,
        mul_prev,
        mul_next,
        mul_replay,
        mul_rods_label,
        mul_abacus_title,
        mul_table_title,
        mul_table_hint,
        mul_steps_title,
        mul_col_step,
        mul_col_product,
        mul_col_record,
        mul_col_total,
        ..
    } = strings;

    let title = content.title;
    let intro = content.intro;
    let entries: Vec<Entry> = content
        .lessons
        .into_iter()
        .map(|lesson| {
            let task =
                Multiplication::new(lesson.example.multiplicand, lesson.example.multiplier).ok();
            Entry { lesson, task }
        })
        .collect();

    let selected = RwSignal::new(0usize);
    let step = RwSignal::new(0usize);
    let lessons_nav_label = mul_lessons.clone();
    let lessons_caption = mul_lessons;

    let entries_nav = entries.clone();
    let lesson_buttons = entries_nav
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let id = entry.lesson.id.clone();
            let label = entry.lesson.title.clone();
            let example = &entry.lesson.example;
            let spec = format!(
                "{}×{}",
                example.multiplicand.to_string().len(),
                example.multiplier.to_string().len()
            );
            let selected_for_button = selected;
            let step_for_button = step;
            view! {
                <li>
                    <button
                        type="button"
                        class="anzan-mul-lesson-button"
                        class:is-active=move || selected_for_button.get() == index
                        aria-label=format!("{id}: {label}")
                        aria-pressed=move || if selected_for_button.get() == index { "true" } else { "false" }
                        on:click=move |_| {
                            selected_for_button.set(index);
                            step_for_button.set(0);
                        }
                    >
                        <span class="anzan-mul-lesson-id">{id.clone()}</span>
                        <span class="anzan-mul-lesson-title">{label.clone()}</span>
                        <span class="anzan-mul-lesson-spec">{spec.clone()}</span>
                    </button>
                </li>
            }
        })
        .collect_view();

    let entries_body = entries.clone();
    let goal_label = mul_goal;
    let example_label = mul_example.clone();
    let lesson_body = move || {
        let Some(entry) = entries_body.get(selected.get()) else {
            return view! { <p></p> }.into_any();
        };
        let lesson = &entry.lesson;
        let paragraphs = lesson
            .paragraphs
            .iter()
            .map(|text| view! { <p class="anzan-mul-paragraph">{text.clone()}</p> }.into_any())
            .collect_view();
        view! {
            <header class="anzan-mul-lesson-head">
                <h3>{lesson.title.clone()}</h3>
                <p class="anzan-mul-goal">
                    <span class="anzan-control-label">{goal_label.clone()}</span>
                    {lesson.goal.clone()}
                </p>
            </header>
            <div class="anzan-mul-prose">{paragraphs}</div>
            <div class="anzan-mul-example">
                <span class="anzan-control-label">{example_label.clone()}</span>
                <strong>{lesson.example.caption.clone()}</strong>
            </div>
        }
        .into_any()
    };

    let entries_table = entries.clone();
    let table_title = mul_table_title.clone();
    let table_hint = mul_table_hint;
    let table_view = move || {
        // La tabla sigue al paso actual: resalta el producto de una cifra que se está
        // calculando y, antes de empezar, el ejemplo de la lección si cabe en la tabla.
        let highlight = entries_table.get(selected.get()).and_then(|entry| {
            let task = entry.task.as_ref()?;
            if let Some(partial) = step
                .get()
                .checked_sub(1)
                .and_then(|index| task.steps().get(index))
            {
                return Some((
                    u32::from(partial.multiplier_digit),
                    u32::from(partial.multiplicand_digit),
                ));
            }
            let example = &entry.lesson.example;
            (example.multiplicand <= 9 && example.multiplier <= 9)
                .then_some((example.multiplicand, example.multiplier))
        });
        ninety_nine_table(&table_title, &table_hint, highlight)
    };

    let entries_abacus = entries.clone();
    let abacus_label = mul_rods_label;
    let abacus_digits = move || {
        entries_abacus
            .get(selected.get())
            .and_then(|entry| entry.task.as_ref())
            .map(|task| task.state_after(step.get()).digits().to_vec())
            .unwrap_or_default()
    };
    let abacus = move || abacus_view(&abacus_digits(), &abacus_label);

    let entries_narration = entries.clone();
    let empty_text = mul_empty_abacus;
    let step_of_text = mul_step_of;
    let step_product_text = mul_step_product;
    let narration = move || {
        let Some(entry) = entries_narration.get(selected.get()) else {
            return String::new();
        };
        let Some(task) = entry.task.as_ref() else {
            return String::new();
        };
        let total = task.steps().len();
        let visible = step.get().min(total);
        if visible == 0 {
            return empty_text.clone();
        }
        let Some(partial) = task.steps().get(visible - 1) else {
            return String::new();
        };
        let head = step_of_text
            .clone()
            .replace("{n}", &visible.to_string())
            .replace("{total}", &total.to_string());
        let body = step_product_text
            .clone()
            .replace("{a}", &partial.multiplier_digit.to_string())
            .replace("{b}", &partial.multiplicand_digit.to_string())
            .replace("{recorded}", &format!("{}{}", partial.tens, partial.units))
            .replace("{total}", &partial.running_total.to_string());
        format!("{head} — {body}")
    };

    let entries_counter = entries.clone();
    let counter = move || {
        let total = entries_counter
            .get(selected.get())
            .and_then(|entry| entry.task.as_ref())
            .map_or(0, |task| task.steps().len());
        format!("{}/{}", step.get().min(total), total)
    };

    let entries_result = entries.clone();
    let result_template = mul_result;
    let result_view = move || {
        let Some(entry) = entries_result.get(selected.get()) else {
            return view! { <p class="anzan-mul-result" hidden=true></p> }.into_any();
        };
        let Some(task) = entry.task.as_ref() else {
            return view! { <p class="anzan-mul-result" hidden=true></p> }.into_any();
        };
        if !entry.lesson.show_steps || step.get() < task.steps().len() {
            return view! { <p class="anzan-mul-result" hidden=true></p> }.into_any();
        }
        let text = result_template
            .clone()
            .replace("{product}", &task.product().to_string());
        view! {
            <p class="anzan-mul-result" role="status">
                <strong>{text}</strong>
                <span>{entry.lesson.example.conclusion.clone()}</span>
            </p>
        }
        .into_any()
    };

    let entries_steps = entries.clone();
    let can_previous = move || step.get() > 0;
    let can_next = move || {
        entries_steps
            .get(selected.get())
            .and_then(|entry| entry.task.as_ref())
            .is_some_and(|task| step.get() < task.steps().len())
    };

    let entries_steps_flag = entries.clone();
    let has_steps = Signal::derive(move || {
        entries_steps_flag
            .get(selected.get())
            .is_some_and(|entry| entry.lesson.show_steps)
    });

    let entries_conclusion = entries.clone();
    let conclusion_text = move || {
        entries_conclusion
            .get(selected.get())
            .filter(|entry| !entry.lesson.show_steps)
            .map(|entry| entry.lesson.example.conclusion.clone())
            .unwrap_or_default()
    };
    let entries_caption = entries.clone();
    let formula_caption = move || {
        entries_caption
            .get(selected.get())
            .filter(|entry| !entry.lesson.show_steps)
            .map(|entry| entry.lesson.example.caption.clone())
            .unwrap_or_default()
    };

    let entries_step_table = entries.clone();
    let step_labels = StepLabels {
        title: mul_steps_title,
        step: mul_col_step,
        product: mul_col_product,
        record: mul_col_record,
        total: mul_col_total,
    };
    let step_table = move || {
        let Some(entry) = entries_step_table.get(selected.get()) else {
            return view! { <span></span> }.into_any();
        };
        let Some(task) = entry.task.as_ref() else {
            return view! { <span></span> }.into_any();
        };
        step_table_view(task, step.get(), &step_labels)
    };

    let on_previous = move |_| step.update(|value| *value = value.saturating_sub(1));
    let entries_next = entries.clone();
    let on_next = move |_| {
        let total = entries_next
            .get(selected.get_untracked())
            .and_then(|entry| entry.task.as_ref())
            .map_or(0, |task| task.steps().len());
        step.update(|value| {
            if *value < total {
                *value += 1;
            }
        });
    };
    let on_replay = move |_| step.set(0);

    view! {
        <section class="anzan-mul" aria-labelledby="anzan-mul-title">
            <header class="anzan-mul-head">
                <div class="anzan-mul-heading">
                    <p class="anzan-mul-eyebrow">"空盘前乘法"</p>
                    <h2 id="anzan-mul-title">{title}</h2>
                </div>
                <p class="anzan-mul-intro">{intro}</p>
            </header>

            <nav class="anzan-mul-lessons" aria-label=lessons_nav_label>
                <span class="anzan-control-label">{lessons_caption}</span>
                <ol class="anzan-mul-lesson-grid">{lesson_buttons}</ol>
            </nav>

            <div class="anzan-mul-layout">
                <article class="anzan-mul-reading">
                    {lesson_body}
                    <details class="anzan-mul-table-details" open=move || selected.get() == 0>
                        <summary>{mul_table_title.clone()}</summary>
                        {table_view}
                    </details>
                </article>

                <aside class="anzan-mul-visual" hidden=move || !has_steps.get()>
                    <header class="anzan-mul-visual-head">
                        <h3>{mul_abacus_title.clone()}</h3>
                        <span class="anzan-mul-counter">{counter}</span>
                    </header>
                    {abacus}
                    <output class="anzan-mul-narration" aria-live="polite">
                        {narration}
                    </output>
                    <div class="anzan-mul-controls">
                        <button
                            type="button"
                            class="anzan-stage"
                            on:click=on_previous
                            disabled=move || !can_previous()
                        >
                            {mul_prev}
                        </button>
                        <button
                            type="button"
                            class="anzan-stage"
                            on:click=on_next
                            disabled=move || !can_next()
                        >
                            {mul_next}
                        </button>
                        <button type="button" class="anzan-reset" on:click=on_replay>
                            {mul_replay}
                        </button>
                    </div>
                    {result_view}
                </aside>

                <aside class="anzan-mul-formula" hidden=move || has_steps.get()>
                    <span class="anzan-control-label">{mul_example.clone()}</span>
                    <p class="anzan-mul-formula-caption">{formula_caption}</p>
                    <p class="anzan-mul-conclusion">{conclusion_text}</p>
                </aside>
            </div>

            <div hidden=move || !has_steps.get()>{step_table}</div>

            <MultiplicationPractice strings=strings_for_practice />
        </section>
    }
}

/// Etiqueta visible de un nivel de práctica (`M4 · 2×2`).
fn level_label(
    value: u8,
    spec: anzan_engine::multiplication::MultiplicationLevel,
    mental: &str,
) -> String {
    if spec.mental {
        format!(
            "M{value} · {}×{} · {mental}",
            spec.multiplicand_digits, spec.multiplier_digits
        )
    } else {
        format!(
            "M{value} · {}×{}",
            spec.multiplicand_digits, spec.multiplier_digits
        )
    }
}

/// Interpreta la respuesta escrita y la compara con el ejercicio.
///
/// Devuelve `None` si el texto no es un entero sin signo; en ese caso no se cuenta el
/// intento y la UI deja el campo como estaba.
fn parse_answer(drill: &MultiplicationDrill, raw: &str) -> Option<bool> {
    let value = raw.trim().parse::<u64>().ok()?;
    Some(drill.is_correct(value))
}

/// Clave propia del último nivel de práctica elegido.
const MUL_LEVEL_KEY: &str = "anzan.mul.level";
/// Clave propia del progreso agregado de multiplicación.
const MUL_PROGRESS_KEY: &str = "anzan.mul.progress";

/// Nivel guardado, saturado en `1..=7`; si no hay dato válido, empieza en M1.
///
/// Si el valor guardado estaba fuera de rango se reescribe ya normalizado, de modo que
/// una versión futura con más niveles no deje la clave en un estado inválido.
fn initial_level() -> u8 {
    let stored = read_key(MUL_LEVEL_KEY);
    let level = stored
        .as_deref()
        .and_then(|raw| raw.trim().parse::<u8>().ok())
        .map_or(MIN_MUL_LEVEL, |value| {
            value.clamp(MIN_MUL_LEVEL, MAX_MUL_LEVEL)
        });
    if stored.as_deref() != Some(level.to_string().as_str()) {
        write_key(MUL_LEVEL_KEY, &level.to_string());
    }
    level
}

/// Progreso guardado; un formato desconocido o roto vuelve a cero sin romper la UI.
fn initial_progress() -> MulProgress {
    match read_key(MUL_PROGRESS_KEY) {
        Some(raw) => MulProgress::decode(&raw).unwrap_or_else(|| {
            let fresh = MulProgress::new();
            write_key(MUL_PROGRESS_KEY, &fresh.encode());
            fresh
        }),
        None => MulProgress::new(),
    }
}

/// Instante en milisegundos para el tiempo de respuesta (0 fuera del navegador).
fn now_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0.0
    }
}

/// Bloque de práctica por niveles con ejercicios generados.
#[component]
pub fn MultiplicationPractice(strings: UiStrings) -> impl IntoView {
    let UiStrings {
        mul_practice_title,
        mul_practice_intro,
        mul_practice_level,
        mul_practice_answer,
        mul_practice_check,
        mul_practice_new,
        mul_correct,
        mul_wrong,
        mul_practice_stats,
        mul_practice_elapsed,
        mul_mental,
        ..
    } = strings;

    let initial_level = initial_level();
    let level = RwSignal::new(initial_level);
    let seed = RwSignal::new(1u64);
    let drill = RwSignal::new(MultiplicationDrill::generate(initial_level, 1));
    let answer = RwSignal::new(String::new());
    let feedback = RwSignal::new(None::<bool>);
    let progress = RwSignal::new(initial_progress());
    let elapsed = RwSignal::new(0u64);
    let started = RwSignal::new(now_ms());

    let reset_round = move || {
        answer.set(String::new());
        feedback.set(None);
        elapsed.set(0);
        started.set(now_ms());
    };

    let next = move || {
        seed.update(|value| *value = value.wrapping_add(1));
        drill.set(MultiplicationDrill::generate(
            level.get_untracked(),
            seed.get_untracked(),
        ));
        reset_round();
    };

    let level_group_label = mul_practice_level.clone();
    let level_caption = mul_practice_level;
    let mental_label = mul_mental;
    let level_buttons = (MIN_MUL_LEVEL..=MAX_MUL_LEVEL)
        .map(|value| {
            let spec = multiplication_level(value);
            let label = level_label(value, spec, &mental_label);
            let spec_text = format!("{}×{}", spec.multiplicand_digits, spec.multiplier_digits);
            let mental = spec.mental.then(|| mental_label.clone());
            let select = move |_| {
                level.set(value);
                write_key(MUL_LEVEL_KEY, &value.to_string());
                drill.set(MultiplicationDrill::generate(value, seed.get_untracked()));
                reset_round();
            };
            view! {
                <button
                    type="button"
                    class="anzan-mul-level-card"
                    class:is-active=move || level.get() == value
                    aria-label=label
                    aria-pressed=move || if level.get() == value { "true" } else { "false" }
                    on:click=select
                >
                    <span class="anzan-mul-level-id">{format!("M{value}")}</span>
                    <span class="anzan-mul-level-spec">{spec_text}</span>
                    {mental.map(|text| view! {
                        <span class="anzan-mul-level-mental">{text}</span>
                    })}
                </button>
            }
        })
        .collect_view();

    let question = move || {
        let current = drill.get();
        format!(
            "{} × {}",
            current.task().multiplicand(),
            current.task().multiplier()
        )
    };

    let on_submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if feedback.get_untracked().is_some() {
            return;
        }
        let Some(is_correct) = parse_answer(&drill.get_untracked(), &answer.get_untracked()) else {
            return;
        };
        let current_level = level.get_untracked();
        progress.update(|state| state.record(current_level, is_correct));
        write_key(MUL_PROGRESS_KEY, &progress.get_untracked().encode());
        let end = now_ms();
        let start = started.get_untracked();
        elapsed.set(if start > 0.0 && end >= start {
            (end - start).round() as u64
        } else {
            0
        });
        feedback.set(Some(is_correct));
    };

    let wrong_text = mul_wrong;
    let feedback_view = move || match feedback.get() {
        Some(true) => view! {
            <p class="anzan-mul-feedback is-correct" role="status">
                {mul_correct.clone()}
            </p>
        }
        .into_any(),
        Some(false) => {
            let text = wrong_text
                .clone()
                .replace("{product}", &drill.get().task().product().to_string());
            view! {
                <p class="anzan-mul-feedback is-wrong" role="status">
                    {text}
                </p>
            }
            .into_any()
        }
        None => view! { <p class="anzan-mul-feedback" hidden=true></p> }.into_any(),
    };

    let stats_text = move || {
        let state = progress.get();
        let current_level = level.get();
        mul_practice_stats
            .clone()
            .replace("{correct}", &state.correct(current_level).to_string())
            .replace("{attempts}", &state.attempts(current_level).to_string())
    };
    let elapsed_text = move || {
        mul_practice_elapsed
            .clone()
            .replace("{ms}", &elapsed.get().to_string())
    };
    let meter_width = move || {
        let state = progress.get();
        let current_level = level.get();
        let attempts = state.attempts(current_level);
        if attempts == 0 {
            "0%".to_owned()
        } else {
            let ratio = f64::from(state.correct(current_level)) / f64::from(attempts) * 100.0;
            format!("{ratio:.1}%")
        }
    };

    view! {
        <section class="anzan-mul-practice" aria-labelledby="anzan-mul-practice-title">
            <header class="anzan-mul-practice-head">
                <div>
                    <h3 id="anzan-mul-practice-title">{mul_practice_title}</h3>
                    <p>{mul_practice_intro}</p>
                </div>
                <p class="anzan-mul-stats">
                    <span>{stats_text}</span>
                    <span>{elapsed_text}</span>
                </p>
            </header>

            <div class="anzan-mul-levels" role="group" aria-label=level_group_label>
                <span class="anzan-control-label">{level_caption}</span>
                <div class="anzan-mul-level-grid">{level_buttons}</div>
            </div>

            <div class="anzan-mul-question-card">
                <p class="anzan-mul-question" aria-live="polite">
                    {question}
                </p>
                <form class="anzan-answer" on:submit=on_submit>
                    <label class="anzan-field">
                        <span>{mul_practice_answer}</span>
                        <input
                            type="text"
                            inputmode="numeric"
                            autocomplete="off"
                            prop:value=move || answer.get()
                            readonly=move || feedback.get().is_some()
                            on:input=move |event| answer.set(event_target_value(&event))
                        />
                    </label>
                    <button
                        type="submit"
                        class="anzan-submit"
                        disabled=move || feedback.get().is_some()
                    >
                        {mul_practice_check}
                    </button>
                    <button type="button" class="anzan-reset" on:click=move |_| next()>
                        {mul_practice_new}
                    </button>
                </form>
                <div class="anzan-mul-meter" aria-hidden="true">
                    <span style:width=meter_width></span>
                </div>
                {feedback_view}
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anzan_content::{Locale, load_multiplication};

    #[test]
    fn every_lesson_example_matches_the_engine() {
        for locale in [Locale::Es, Locale::En] {
            let content = load_multiplication(locale);
            assert!(
                content.is_ok(),
                "el contenido de multiplicación debe cargar en {}",
                locale.code()
            );
            if let Ok(content) = content {
                assert!(!content.lessons.is_empty());
                for lesson in content.lessons {
                    let result =
                        Multiplication::new(lesson.example.multiplicand, lesson.example.multiplier);
                    assert!(result.is_ok(), "el ejemplo {} debe ser válido", lesson.id);
                    if let Ok(task) = result {
                        assert_eq!(
                            task.product(),
                            lesson.example.expected,
                            "el ejemplo {} debe dar el producto del contenido",
                            lesson.id
                        );
                        assert_eq!(
                            task.state_after(usize::MAX).value(),
                            lesson.example.expected,
                            "el ábaco del ejemplo {} debe terminar en el producto",
                            lesson.id
                        );
                        assert!(!task.steps().is_empty());
                        assert_eq!(
                            task.multiplicand_digits().len(),
                            task.multiplicand().to_string().len()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn practice_labels_name_the_digit_counts_and_mark_mental() {
        for value in MIN_MUL_LEVEL..=MAX_MUL_LEVEL {
            let spec = multiplication_level(value);
            let label = level_label(value, spec, "mental");
            assert!(label.starts_with(&format!("M{value} · ")));
            assert!(label.contains(&format!(
                "{}×{}",
                spec.multiplicand_digits, spec.multiplier_digits
            )));
            assert_eq!(label.ends_with(" · mental"), spec.mental);
        }
    }

    #[test]
    fn practice_storage_defaults_without_a_host() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            assert_eq!(initial_level(), MIN_MUL_LEVEL);
            assert_eq!(initial_progress(), MulProgress::new());
        }
    }

    #[test]
    fn practice_answers_are_parsed_and_checked() {
        let drill = MultiplicationDrill::generate(4, 2026);
        let product = drill.task().product();
        assert_eq!(parse_answer(&drill, ""), None);
        assert_eq!(parse_answer(&drill, "   "), None);
        assert_eq!(parse_answer(&drill, "doce"), None);
        assert_eq!(parse_answer(&drill, "-3"), None);
        assert_eq!(parse_answer(&drill, &product.to_string()), Some(true));
        assert_eq!(parse_answer(&drill, &format!(" {} ", product)), Some(true));
        assert_eq!(
            parse_answer(&drill, &(product + 1).to_string()),
            Some(false)
        );
    }
}
