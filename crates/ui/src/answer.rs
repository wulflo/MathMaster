// SPDX-License-Identifier: AGPL-3.0-or-later
//! Campo de respuesta ultrarrápido compartido por todos los modos de práctica.
//!
//! El valor se evalúa en cada pulsación: en cuanto coincide con la respuesta esperada se
//! marca como correcto y, si el avance automático está activo, se pide el siguiente
//! ejercicio. Si el usuario se pasa de longitud o pulsa Enter con un valor erróneo, se
//! registra el fallo y se ofrece la resolución en el soroban.

#![allow(missing_docs)]

use anzan_content::UiStrings;
use leptos::prelude::*;
use std::rc::Rc;

/// Acción local (no `Send`): los modos de práctica comparten `Rc` con el bucle de audio.
#[derive(Clone)]
pub struct Action(Rc<dyn Fn()>);

impl Action {
    /// Envuelve una closure que no necesita ser enviable entre hilos.
    #[must_use]
    pub fn new(callback: impl Fn() + 'static) -> Self {
        Self(Rc::new(callback))
    }

    /// Ejecuta la acción.
    pub fn run(&self) {
        (self.0)();
    }
}

/// Resultado de evaluar el texto escrito.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerOutcome {
    /// Todavía no se puede decidir; el usuario sigue escribiendo.
    Pending,
    /// El valor coincide exactamente con la respuesta esperada.
    Correct,
    /// Se ha pasado de longitud o se ha confirmado un valor erróneo.
    Wrong,
}

/// Evalúa una respuesta. `forced` representa una confirmación explícita (Enter o botón).
#[must_use]
pub fn classify_answer(
    raw: &str,
    expected: Option<u128>,
    already_resolved: bool,
    forced: bool,
) -> AnswerOutcome {
    if already_resolved {
        return AnswerOutcome::Pending;
    }
    let Some(expected) = expected else {
        return AnswerOutcome::Pending;
    };
    let digits = expected.to_string().len();
    match raw.trim().parse::<u128>() {
        Ok(value) if value == expected => AnswerOutcome::Correct,
        Ok(_) if forced || raw.trim().len() > digits => AnswerOutcome::Wrong,
        Ok(_) => AnswerOutcome::Pending,
        Err(_) if forced && !raw.trim().is_empty() => AnswerOutcome::Wrong,
        Err(_) => AnswerOutcome::Pending,
    }
}

/// Enfoca el campo al aparecer un ejercicio; en nativo no hay documento.
#[cfg(target_arch = "wasm32")]
fn focus_input(input: NodeRef<leptos::html::Input>) {
    if let Some(element) = input.get() {
        let _ = element.focus();
    }
}

/// Enfoca el campo al aparecer un ejercicio; en nativo no hay documento.
#[cfg(not(target_arch = "wasm32"))]
fn focus_input(_input: NodeRef<leptos::html::Input>) {}

/// Campo de respuesta con teclado numérico opcional.
#[component]
pub fn AnswerBox(
    strings: UiStrings,
    /// Texto introducido por el usuario; lo posee el modo de práctica.
    answer: RwSignal<String>,
    /// Respuesta esperada del ejercicio actual.
    expected: RwSignal<Option<u128>>,
    /// Indica si hay un ejercicio esperando respuesta.
    active: RwSignal<bool>,
    /// Avanzar automáticamente tras un acierto.
    auto_advance: RwSignal<bool>,
    /// Se llama al acertar (el modo decide cómo registrarlo).
    on_correct: Action,
    /// Se llama al fallar.
    on_wrong: Action,
    /// Se llama para pedir el siguiente ejercicio.
    on_next: Action,
    /// Se llama al pulsar Espacio antes de resolver.
    #[prop(optional)]
    on_start: Option<Action>,
    /// Abre la resolución del ejercicio en el soroban.
    #[prop(optional)]
    on_show_solution: Option<Action>,
    /// Muestra el teclado numérico para pantallas táctiles.
    #[prop(default = true)]
    keypad: bool,
) -> impl IntoView {
    let resolved = RwSignal::new(false);
    let correct_flash = RwSignal::new(false);
    let input: NodeRef<leptos::html::Input> = NodeRef::new();

    Effect::new(move || {
        let _ = expected.get();
        if active.get() {
            resolved.set(false);
            correct_flash.set(false);
            focus_input(input);
        }
    });

    let next_for_resolve = on_next.clone();
    let resolve: Rc<dyn Fn(AnswerOutcome)> = Rc::new(move |outcome: AnswerOutcome| match outcome {
        AnswerOutcome::Correct => {
            resolved.set(true);
            correct_flash.set(true);
            on_correct.run();
            if auto_advance.get_untracked() {
                next_for_resolve.run();
            }
        }
        AnswerOutcome::Wrong => {
            resolved.set(true);
            on_wrong.run();
        }
        AnswerOutcome::Pending => {}
    });

    let evaluate: Rc<dyn Fn(bool)> = Rc::new(move |forced: bool| {
        let outcome = classify_answer(
            &answer.get_untracked(),
            expected.get_untracked(),
            resolved.get_untracked(),
            forced,
        );
        resolve(outcome);
    });

    let for_input = evaluate.clone();
    let on_input = move |event: leptos::ev::Event| {
        let value = event_target_value(&event);
        answer.set(value);
        for_input(false);
    };

    let for_key = evaluate.clone();
    let next_on_key = on_next.clone();
    let start_on_key = on_start.clone();
    let on_keydown = move |event: leptos::ev::KeyboardEvent| match event.key().as_str() {
        "Enter" => {
            event.prevent_default();
            for_key(true);
        }
        " " => {
            event.prevent_default();
            if resolved.get_untracked() {
                next_on_key.run();
            } else if let Some(start) = start_on_key.as_ref() {
                start.run();
            }
        }
        _ => {}
    };

    let for_digit = evaluate.clone();
    let push_digit: Rc<dyn Fn(char)> = Rc::new(move |digit: char| {
        if resolved.get_untracked() {
            return;
        }
        answer.update(|value| {
            if value.len() < 12 {
                value.push(digit);
            }
        });
        for_digit(false);
        focus_input(input);
    });
    let on_backspace = move |_| {
        if !resolved.get_untracked() {
            answer.update(|value| {
                value.pop();
            });
        }
        focus_input(input);
    };
    let on_clear = move |_| {
        if !resolved.get_untracked() {
            answer.set(String::new());
        }
        focus_input(input);
    };

    let digits = (0..=9)
        .map(|digit| {
            let character = char::from(b'0' + digit);
            let press = push_digit.clone();
            view! {
                <button
                    type="button"
                    class="anzan-keypad-key"
                    on:click=move |_| press(character)
                >
                    {character.to_string()}
                </button>
            }
            .into_any()
        })
        .collect_view();

    let expected_text = move || match expected.get() {
        Some(value) if resolved.get() => format!("{}: {value}", strings.drill_expected),
        _ => String::new(),
    };
    let has_solution = on_show_solution.is_some();
    let solution_action = on_show_solution.clone();
    let show_solution = move |_| {
        if let Some(callback) = solution_action.as_ref() {
            callback.run();
        }
    };
    let next_button = on_next.clone();

    view! {
        <div class="anzan-answer-box">
            <div class="anzan-answer-row" class:is-correct=correct_flash>
                <label class="anzan-field">
                    <span>{strings.drill_answer.clone()}</span>
                    <input
                        node_ref=input
                        type="text"
                        inputmode="numeric"
                        autocomplete="off"
                        disabled=move || !active.get() || resolved.get()
                        prop:value=move || answer.get()
                        on:input=on_input
                        on:keydown=on_keydown
                    />
                </label>
                <button
                    type="button"
                    class="anzan-submit"
                    disabled=move || !active.get() || resolved.get()
                    on:click=move |_| evaluate(true)
                >
                    {strings.drill_submit.clone()}
                </button>
            </div>
            <p class="anzan-answer-hint">{strings.answer_hint.clone()}</p>
            <div class="anzan-answer-feedback" role="status">
                <span>{expected_text}</span>
                <button
                    type="button"
                    class="anzan-see-solution"
                    hidden=move || !resolved.get() || !has_solution
                    on:click=show_solution
                >
                    {strings.answer_see_soroban.clone()}
                </button>
                <button
                    type="button"
                    class="anzan-next"
                    hidden=move || !resolved.get()
                    on:click=move |_| next_button.run()
                >
                    {strings.answer_next.clone()}
                </button>
            </div>
            <div class="anzan-keypad" role="group" aria-label=strings.answer_keypad.clone() hidden=!keypad>
                {digits}
                <button type="button" class="anzan-keypad-key" on:click=on_backspace>
                    "⌫"
                </button>
                <button type="button" class="anzan-keypad-key" on:click=on_clear>
                    {strings.answer_clear.clone()}
                </button>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_value_is_recognised_while_typing() {
        assert_eq!(
            classify_answer("12", Some(12), false, false),
            AnswerOutcome::Correct
        );
        assert_eq!(
            classify_answer("1", Some(12), false, false),
            AnswerOutcome::Pending
        );
    }

    #[test]
    fn extra_digits_or_forced_wrong_value_fail_immediately() {
        assert_eq!(
            classify_answer("123", Some(12), false, false),
            AnswerOutcome::Wrong
        );
        assert_eq!(
            classify_answer("11", Some(12), false, true),
            AnswerOutcome::Wrong
        );
        assert_eq!(
            classify_answer("11", Some(12), false, false),
            AnswerOutcome::Pending
        );
    }

    #[test]
    fn resolved_answers_are_not_evaluated_twice() {
        assert_eq!(
            classify_answer("12", Some(12), true, false),
            AnswerOutcome::Pending
        );
        assert_eq!(
            classify_answer("12", None, false, true),
            AnswerOutcome::Pending
        );
    }
}
