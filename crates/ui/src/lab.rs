// SPDX-License-Identifier: AGPL-3.0-or-later
//! Laboratorio de soroban libre y demostrador de movimientos.
//!
//! El demostrador usa los planes de `anzan_core` (no reimplementa las reglas) y muestra
//! para cada toque la regla aplicada: directo, complemento a 5, complemento a 10, acarreo
//! o préstamo.

#![allow(missing_docs)]

use crate::multiplication::abacus_view;
use crate::soroban::SorobanBoard;
use anzan_content::UiStrings;
use anzan_core::{DEFAULT_RODS, Soroban, Step, StepKind};
use leptos::prelude::*;

/// Varillas mínimas y máximas del laboratorio.
pub const MIN_RODS: usize = 4;
/// Varillas mínimas y máximas del laboratorio.
pub const MAX_RODS: usize = 13;

/// Dígitos de un valor para el ábaco de sólo lectura.
fn digits_of(value: u128) -> Vec<u8> {
    Soroban::from_value(value, MAX_RODS)
        .map(|soroban| soroban.digits().iter().map(|digit| digit.value()).collect())
        .unwrap_or_default()
}

/// Etiqueta de la regla de un paso.
fn rule_label(strings: &UiStrings, kind: StepKind) -> String {
    match kind {
        StepKind::Direct => strings.rule_direct.clone(),
        StepKind::FiveComplement => strings.rule_complement5.clone(),
        StepKind::TenComplement => strings.rule_complement10.clone(),
        StepKind::Carry => strings.rule_carry.clone(),
        StepKind::Borrow => strings.rule_borrow.clone(),
    }
}

/// Calcula los estados intermediates de una operación.
fn build_steps(start: u32, add: bool, amount: u32) -> Option<(Vec<(Step, u128)>, u128)> {
    let mut soroban = Soroban::from_value(u128::from(start), MAX_RODS).ok()?;
    let plan = if add {
        soroban.plan_add(u128::from(amount)).ok()?
    } else {
        soroban.plan_sub(u128::from(amount)).ok()?
    };
    let mut states = Vec::with_capacity(plan.len());
    for step in plan.steps() {
        let _ = soroban.touch(step.touch);
        states.push((*step, soroban.value()));
    }
    Some((states, soroban.value()))
}

/// Ábaco libre con selector de varillas.
#[component]
pub fn SorobanLab(strings: UiStrings) -> impl IntoView {
    let rods = RwSignal::new(DEFAULT_RODS);
    let buttons = (MIN_RODS..=MAX_RODS)
        .map(|value| {
            view! {
                <button
                    type="button"
                    class="anzan-stage"
                    class:is-active=move || rods.get() == value
                    aria-pressed=move || if rods.get() == value { "true" } else { "false" }
                    on:click=move |_| rods.set(value)
                >
                    {value.to_string()}
                </button>
            }
            .into_any()
        })
        .collect_view();
    let strings_for_board = strings.clone();
    let board = move || {
        let value = rods.get();
        view! { <SorobanBoard strings=strings_for_board.clone() rods=value /> }.into_any()
    };
    view! {
        <section class="anzan-sub-card" aria-labelledby="anzan-lab-title">
            <header class="anzan-lab-head">
                <h3 id="anzan-lab-title">{strings.soroban_label.clone()}</h3>
                <div class="anzan-lab-rods" role="group" aria-label=strings.lab_rods.clone()>
                    <span class="anzan-control-label">{strings.lab_rods.clone()}</span>
                    {buttons}
                </div>
            </header>
            {board}
        </section>
    }
}

/// Demostrador paso a paso de `plan_add` / `plan_sub`.
#[component]
pub fn MovementDemo(
    strings: UiStrings,
    /// Petición externa: (inicio, ¿suma?, cantidad).
    #[prop(optional)]
    demo_request: Option<RwSignal<Option<(u32, bool, u32)>>>,
) -> impl IntoView {
    let start = RwSignal::new(0u32);
    let amount = RwSignal::new(1u32);
    let add = RwSignal::new(true);
    let computed = RwSignal::new(None::<(Vec<(Step, u128)>, u128)>);
    let current = RwSignal::new(0usize);

    if let Some(request) = demo_request {
        Effect::new(move || {
            if let Some((initial, sum, quantity)) = request.get() {
                start.set(initial);
                add.set(sum);
                amount.set(quantity);
                computed.set(build_steps(initial, sum, quantity));
                current.set(0);
            }
        });
    }

    let on_run = move |_| {
        computed.set(build_steps(
            start.get_untracked(),
            add.get_untracked(),
            amount.get_untracked(),
        ));
        current.set(0);
    };
    let outcome_strings = strings.clone();
    let outcome = move || match computed.get() {
        Some((_, value)) => outcome_strings
            .demo_result
            .replace("{value}", &value.to_string()),
        None => outcome_strings.demo_invalid.clone(),
    };
    let step_strings = strings.clone();
    let steps = move || {
        computed
            .get()
            .map(|(states, _)| {
                states
                    .iter()
                    .enumerate()
                    .map(|(index, (step, value))| {
                        view! {
                            <li class:is-current=move || current.get() == index + 1>
                                <span class="anzan-demo-step-no">
                                    {step_strings.demo_step.replace("{n}", &(index + 1).to_string())
                                        .replace("{total}", &states.len().to_string())}
                                </span>
                                <span class="anzan-demo-rule">{rule_label(&step_strings, step.kind)}</span>
                                <span class="anzan-demo-touch">
                                    {format!("{} · {}", step.touch.rod, step.touch.slot.index())}
                                </span>
                                {abacus_view(&digits_of(*value), &step_strings.mul_rods_label)}
                            </li>
                        }
                        .into_any()
                    })
                    .collect_view()
            })
            .unwrap_or_default()
    };

    view! {
        <section class="anzan-sub-card" aria-labelledby="anzan-demo-title">
            <h3 id="anzan-demo-title">{strings.demo_title.clone()}</h3>
            <div class="anzan-demo-controls">
                <label class="anzan-field">
                    <span>{strings.demo_start.clone()}</span>
                    <input
                        type="number"
                        min="0"
                        max="99999"
                        prop:value=move || start.get()
                        on:input=move |event| {
                            if let Ok(value) = event_target_value(&event).parse::<u32>()
                                && value <= 99_999
                            {
                                start.set(value);
                            }
                        }
                    />
                </label>
                <div class="anzan-demo-ops" role="group" aria-label=strings.demo_rule.clone()>
                    <button
                        type="button"
                        class="anzan-stage"
                        class:is-active=move || add.get()
                        aria-pressed=move || if add.get() { "true" } else { "false" }
                        on:click=move |_| add.set(true)
                    >
                        {strings.demo_add.clone()}
                    </button>
                    <button
                        type="button"
                        class="anzan-stage"
                        class:is-active=move || !add.get()
                        aria-pressed=move || if add.get() { "false" } else { "true" }
                        on:click=move |_| add.set(false)
                    >
                        {strings.demo_sub.clone()}
                    </button>
                </div>
                <label class="anzan-field">
                    <span>{strings.demo_amount.clone()}</span>
                    <input
                        type="number"
                        min="0"
                        max="99999"
                        prop:value=move || amount.get()
                        on:input=move |event| {
                            if let Ok(value) = event_target_value(&event).parse::<u32>()
                                && value <= 99_999
                            {
                                amount.set(value);
                            }
                        }
                    />
                </label>
                <button type="button" class="anzan-demo-run" on:click=on_run>
                    {strings.demo_run.clone()}
                </button>
            </div>
            <output class="anzan-demo-result" role="status">{outcome}</output>
            <ol class="anzan-demo-steps">{steps}</ol>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rod_selector_stays_inside_the_documented_range() {
        assert_eq!(MIN_RODS, 4);
        assert_eq!(MAX_RODS, 13);
    }

    #[test]
    fn plans_include_a_rule_for_every_touch() {
        let result = build_steps(99, true, 1);
        assert!(result.is_some());
        if let Some((steps, value)) = result {
            assert!(!steps.is_empty());
            assert_eq!(steps.last().map(|(_, value)| *value), Some(value));
            assert!(steps.iter().any(|(step, _)| step.kind == StepKind::Carry));
        }
    }
}
