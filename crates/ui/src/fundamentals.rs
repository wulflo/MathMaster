// SPDX-License-Identifier: AGPL-3.0-or-later
//! Manual práctico de fundamentos del soroban.
//!
//! Los capítulos, párrafos, fórmulas y pasos vienen del RON. La única lógica aquí es
//! aplicar una fórmula elegida al ábaco con los planes de `anzan_core` para que el ejemplo
//! sea interactivo y no una imagen estática.

#![allow(missing_docs)]

use crate::multiplication::abacus_view;
use anzan_content::{FormulaRow, FundamentalsContent, FundamentalsSection, UiStrings};
use anzan_core::Soroban;
use leptos::prelude::*;

/// Interpreta una etiqueta de fórmula (`+4`, `−4`) como (¿suma?, cantidad).
fn parse_formula(label: &str) -> Option<(bool, u32)> {
    let mut characters = label.trim().chars();
    let add = match characters.next()? {
        '+' => true,
        '-' | '\u{2212}' => false,
        _ => return None,
    };
    let amount: String = characters.collect();
    amount
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .map(|value| (add, value))
}

/// Aplica una fórmula a un valor inicial y devuelve el resultado del ábaco.
fn apply_formula(start: u32, label: &str) -> Option<u128> {
    let (add, amount) = parse_formula(label)?;
    let mut soroban = Soroban::from_value(u128::from(start), 6).ok()?;
    let plan = if add {
        soroban.plan_add(u128::from(amount)).ok()?
    } else {
        soroban.plan_sub(u128::from(amount)).ok()?
    };
    for step in plan.steps() {
        let _ = soroban.touch(step.touch);
    }
    Some(soroban.value())
}

/// Dígitos de un valor para dibujarlos en el ábaco de sólo lectura.
fn digits_of(value: u128) -> Vec<u8> {
    Soroban::from_value(value, 6)
        .map(|soroban| soroban.digits().iter().map(|digit| digit.value()).collect())
        .unwrap_or_default()
}

/// Resultado de la fórmula seleccionada sobre el valor inicial.
fn chosen_result(options: &[FormulaRow], start: u32, choice: usize) -> Option<u128> {
    options
        .get(choice)
        .and_then(|formula| apply_formula(start, &formula.label))
}

/// Demostración interactiva de una tabla de fórmulas.
#[component]
fn FormulaDemo(section: FundamentalsSection, strings: UiStrings) -> impl IntoView {
    let start = RwSignal::new(0u32);
    let choice = RwSignal::new(0usize);
    let formulas = section.formulas.clone();
    let for_after = formulas.clone();
    let for_outcome = formulas.clone();
    let before_label = strings.soroban_label.clone();
    let after_label = strings.soroban_label.clone();
    let before_abacus = move || abacus_view(&digits_of(u128::from(start.get())), &before_label);
    let after_abacus = move || {
        let value =
            chosen_result(&for_after, start.get(), choice.get()).unwrap_or(u128::from(start.get()));
        abacus_view(&digits_of(value), &after_label)
    };
    let outcome = move || match chosen_result(&for_outcome, start.get(), choice.get()) {
        Some(value) => strings.demo_result.replace("{value}", &value.to_string()),
        None => strings.demo_invalid.clone(),
    };
    let formula_buttons = section
        .formulas
        .iter()
        .enumerate()
        .map(|(index, formula)| {
            let label = formula.label.clone();
            view! {
                <button
                    type="button"
                    class="anzan-stage"
                    class:is-active=move || choice.get() == index
                    aria-pressed=move || if choice.get() == index { "true" } else { "false" }
                    on:click=move |_| choice.set(index)
                >
                    {label}
                </button>
            }
            .into_any()
        })
        .collect_view();

    view! {
        <div class="anzan-formula-demo">
            <div class="anzan-formula-options" role="group" aria-label=strings.demo_rule.clone()>
                {formula_buttons}
            </div>
            <label class="anzan-field">
                <span>{strings.demo_start.clone()}</span>
                <input
                    type="number"
                    min="0"
                    max="999"
                    prop:value=move || start.get()
                    on:input=move |event| {
                        if let Ok(value) = event_target_value(&event).parse::<u32>()
                            && value <= 999
                        {
                            start.set(value);
                        }
                    }
                />
            </label>
            <div class="anzan-formula-abaci">
                {before_abacus}
                <span class="anzan-formula-arrow" aria-hidden="true">"→"</span>
                {after_abacus}
            </div>
            <output class="anzan-formula-result" role="status">{outcome}</output>
        </div>
    }
}

/// Un varillero del ábaco en SVG; los colores vienen de los tokens del tema.
fn rod_group(x: i32, digit: u8, frame: bool) -> AnyView {
    let heaven = digit >= 5;
    let earth = digit % 5;
    let earth_beads = (0..4)
        .map(|index| {
            let active = index < usize::from(earth);
            let cy = if active {
                100 - index * 11
            } else {
                120 + index * 17
            };
            view! {
                <circle
                    class=move || format!("diagram-bead{}", if active { " is-active" } else { "" })
                    cx="18"
                    cy=cy
                    r="7"
                />
            }
            .into_any()
        })
        .collect_view();
    view! {
        <g class="diagram-rod" transform=format!("translate({x},0)")>
            {frame.then(|| view! {
                <rect class="diagram-frame" x="0" y="8" width="36" height="172" rx="3" />
            })}
            <line class="diagram-guide" x1="18" y1="8" x2="18" y2="180" />
            <rect class="diagram-beam" x="0" y="60" width="36" height="4" />
            <circle
                class=format!("diagram-bead is-heaven{}", if heaven { " is-active" } else { "" })
                cx="18"
                cy=if heaven { 52 } else { 28 }
                r="7"
            />
            {earth_beads}
            <text class="diagram-digit" x="18" y="198" text-anchor="middle">
                {digit.to_string()}
            </text>
        </g>
    }
    .into_any()
}

/// Flecha horizontal hacia la derecha.
fn arrow_right(x1: i32, y: i32, x2: i32) -> AnyView {
    view! {
        <g class="diagram-arrow">
            <line x1=x1 y1=y x2={x2 - 9} y2=y />
            <polygon points=format!("{x2},{y} {},{} {},{}", x2 - 10, y - 6, x2 - 10, y + 6) />
        </g>
    }
    .into_any()
}

/// Flecha vertical hacia arriba (pulgar).
fn arrow_up(x: i32, y1: i32, y2: i32) -> AnyView {
    view! {
        <g class="diagram-arrow">
            <line x1=x y1=y1 x2=x y2={y2 + 9} />
            <polygon points=format!("{x},{y2} {},{} {},{}", x - 6, y2 + 10, x + 6, y2 + 10) />
        </g>
    }
    .into_any()
}

/// Flecha vertical hacia abajo (índice).
fn arrow_down(x: i32, y1: i32, y2: i32) -> AnyView {
    view! {
        <g class="diagram-arrow">
            <line x1=x y1=y1 x2=x y2={y2 - 9} />
            <polygon points=format!("{x},{y2} {},{} {},{}", x - 6, y2 - 10, x + 6, y2 - 10) />
        </g>
    }
    .into_any()
}

/// Leyenda con muestras de color, legible también en móvil.
fn legend(entries: Vec<(String, &'static str)>) -> AnyView {
    let items = entries
        .into_iter()
        .map(|(label, kind)| {
            view! {
                <li>
                    <span class="diagram-swatch" data-kind=kind></span>
                    <span>{label}</span>
                </li>
            }
            .into_any()
        })
        .collect_view();
    view! { <ul class="anzan-diagram-legend">{items}</ul> }.into_any()
}

/// Esquema vectorial asociado a una sección del manual.
#[component]
pub fn SvgDiagram(kind: String, strings: UiStrings) -> impl IntoView {
    let content = match kind.as_str() {
        "anatomy" => view! {
            <svg class="anzan-diagram-svg" viewBox="0 0 560 230" role="img" aria-label=strings.diagram_hari.clone()>
                <rect class="diagram-frame" x="20" y="8" width="440" height="172" rx="3" />
                {rod_group(40, 0, false)}
                {rod_group(120, 8, false)}
                {rod_group(200, 5, false)}
                {rod_group(280, 3, false)}
                {rod_group(360, 0, false)}
                <line class="diagram-guide" x1="240" y1="62" x2="500" y2="40" />
                <circle class="diagram-point" cx="500" cy="40" r="5" />
                <line class="diagram-guide" x1="138" y1="52" x2="500" y2="85" />
                <circle class="diagram-point" cx="500" cy="85" r="5" />
                <line class="diagram-guide" x1="298" y1="100" x2="500" y2="130" />
                <circle class="diagram-point" cx="500" cy="130" r="5" />
                <line class="diagram-guide" x1="378" y1="182" x2="500" y2="175" />
                <circle class="diagram-point" cx="500" cy="175" r="5" />
            </svg>
            {legend(vec![
                (strings.diagram_hari, "hari"),
                (strings.diagram_go_dama, "heaven"),
                (strings.diagram_ichi_dama, "earth"),
                (strings.diagram_teii_ten, "unit"),
            ])}
        }
        .into_any(),
        "reading" => view! {
            <svg class="anzan-diagram-svg" viewBox="0 0 420 230" role="img" aria-label=strings.diagram_go_dama.clone()>
                <rect class="diagram-frame" x="14" y="8" width="230" height="172" rx="3" />
                {rod_group(24, 8, false)}
                {rod_group(104, 3, false)}
                {rod_group(184, 5, false)}
                <text class="diagram-caption" x="272" y="80">"8"</text>
                <text class="diagram-caption" x="272" y="120">"3"</text>
                <text class="diagram-caption" x="272" y="160">"5"</text>
            </svg>
            {legend(vec![
                (strings.diagram_go_dama, "heaven"),
                (strings.diagram_ichi_dama, "earth"),
            ])}
        }
        .into_any(),
        "fingering" => view! {
            <svg class="anzan-diagram-svg" viewBox="0 0 460 230" role="img" aria-label=strings.diagram_thumb.clone()>
                {rod_group(60, 4, true)}
                {arrow_up(140, 130, 88)}
                {arrow_down(150, 20, 56)}
                <text class="diagram-caption" x="190" y="100">{strings.diagram_thumb.clone()}</text>
                <text class="diagram-caption" x="190" y="150">{strings.diagram_index.clone()}</text>
            </svg>
        }
        .into_any(),
        "complement5" => view! {
            <svg class="anzan-diagram-svg" viewBox="0 0 520 230" role="img" aria-label=strings.rule_complement5.clone()>
                {rod_group(30, 4, true)}
                {arrow_right(110, 100, 190)}
                {rod_group(210, 5, true)}
                <text class="diagram-caption" x="90" y="220">{strings.diagram_before.clone()}</text>
                <text class="diagram-caption" x="270" y="220">{strings.diagram_after.clone()}</text>
            </svg>
        }
        .into_any(),
        "complement10" => view! {
            <svg class="anzan-diagram-svg" viewBox="0 0 620 230" role="img" aria-label=strings.rule_complement10.clone()>
                {rod_group(20, 9, true)}
                {arrow_right(100, 100, 170)}
                {rod_group(190, 0, true)}
                {arrow_right(270, 60, 340)}
                {rod_group(360, 1, true)}
                <text class="diagram-caption" x="80" y="220">{strings.diagram_before.clone()}</text>
                <text class="diagram-caption" x="250" y="220">{strings.diagram_after.clone()}</text>
                <text class="diagram-caption" x="430" y="220">{strings.diagram_carry.clone()}</text>
            </svg>
        }
        .into_any(),
        "mixed" => view! {
            <svg class="anzan-diagram-svg" viewBox="0 0 620 230" role="img" aria-label=strings.diagram_mixed.clone()>
                {rod_group(20, 9, true)}
                {arrow_right(100, 100, 170)}
                {rod_group(190, 5, true)}
                {arrow_right(270, 60, 340)}
                {rod_group(360, 1, true)}
                <text class="diagram-caption" x="80" y="220">{strings.diagram_before.clone()}</text>
                <text class="diagram-caption" x="250" y="220">{strings.diagram_after.clone()}</text>
                <text class="diagram-caption" x="430" y="220">{strings.diagram_carry.clone()}</text>
            </svg>
            {legend(vec![(strings.diagram_mixed, "mixed")])}
        }
        .into_any(),
        "fading" => {
            let stages = [
                ("is-full", strings.stage_full.clone()),
                ("is-ghost", strings.stage_ghost.clone()),
                ("is-rods", strings.stage_rods.clone()),
                ("is-unit", strings.stage_unit_marks.clone()),
                ("is-blank", strings.stage_blank.clone()),
            ];
            let marks = stages
                .iter()
                .enumerate()
                .map(|(index, (class, label))| {
                    view! {
                        <g transform=format!("translate({},0)", 20 + index * 118)>
                            <g class=format!("diagram-rod {class}")>
                                {rod_group(0, 8, true)}
                                <circle class="diagram-unit-mark" cx="18" cy="182" r="2.5" />
                            </g>
                            <text class="diagram-caption" x="18" y="220" text-anchor="middle">
                                {label.clone()}
                            </text>
                        </g>
                    }
                    .into_any()
                })
                .collect_view();
            view! {
                <svg class="anzan-diagram-svg" viewBox="0 0 620 240" role="img" aria-label=strings.diagram_before.clone()>
                    {marks}
                </svg>
            }
            .into_any()
        }
        _ => view! { <span></span> }.into_any(),
    };

    view! { <figure class="anzan-diagram">{content}</figure> }
}

/// Manual completo de fundamentos del soroban.
#[component]
pub fn FundamentalsView(strings: UiStrings, content: FundamentalsContent) -> impl IntoView {
    let operation_label = strings.formula_operation.clone();
    let equation_label = strings.formula_equation.clone();
    let note_label = strings.formula_note.clone();
    let chapters = content
        .chapters
        .into_iter()
        .enumerate()
        .map(|(index, chapter)| {
            let strings_for_demo = strings.clone();
            let operation_for_demo = operation_label.clone();
            let equation_for_demo = equation_label.clone();
            let note_for_demo = note_label.clone();
            let sections = chapter
                .sections
                .into_iter()
                .map(|section| {
                    let heading = section.heading.clone();
                    let paragraphs = section
                        .paragraphs
                        .iter()
                        .map(|text| view! { <p>{text.clone()}</p> }.into_any())
                        .collect_view();
                    let formulas = section
                        .formulas
                        .iter()
                        .map(|row| {
                            view! {
                                <tr>
                                    <th scope="row">{row.label.clone()}</th>
                                    <td>{row.equation.clone()}</td>
                                    <td>{row.note.clone()}</td>
                                </tr>
                            }
                            .into_any()
                        })
                        .collect_view();
                    let steps = section
                        .steps
                        .iter()
                        .map(|text| view! { <li>{text.clone()}</li> }.into_any())
                        .collect_view();
                    let tips = section
                        .tips
                        .iter()
                        .map(|text| view! { <li>{text.clone()}</li> }.into_any())
                        .collect_view();
                    let has_formulas = !section.formulas.is_empty();
                    let has_steps = !section.steps.is_empty();
                    let has_tips = !section.tips.is_empty();
                    let diagram = section.diagram.clone();
                    let demo = has_formulas.then(|| {
                        view! {
                            <FormulaDemo section=section strings=strings_for_demo.clone() />
                        }
                    });
                    view! {
                        <section class="anzan-chapter-section">
                            <h4>{heading}</h4>
                            <div class="anzan-chapter-prose">{paragraphs}</div>
                            {diagram.map(|kind| view! {
                                <SvgDiagram kind=kind strings=strings_for_demo.clone() />
                            })}
                            {demo}
                            <table class="anzan-formula-table" hidden=!has_formulas>
                                <thead>
                                    <tr>
                                        <th scope="col">{operation_for_demo.clone()}</th>
                                        <th scope="col">{equation_for_demo.clone()}</th>
                                        <th scope="col">{note_for_demo.clone()}</th>
                                    </tr>
                                </thead>
                                <tbody>{formulas}</tbody>
                            </table>
                            <ol class="anzan-chapter-steps" hidden=!has_steps>{steps}</ol>
                            <ul class="anzan-chapter-tips" hidden=!has_tips>{tips}</ul>
                        </section>
                    }
                    .into_any()
                })
                .collect_view();
            view! {
                <details class="anzan-chapter" open=index == 0>
                    <summary>
                        <strong>{chapter.id.clone()}</strong>
                        {chapter.title.clone()}
                    </summary>
                    <p class="anzan-chapter-goal">{chapter.goal.clone()}</p>
                    {sections}
                </details>
            }
            .into_any()
        })
        .collect_view();

    view! {
        <section class="anzan-fundamentals" aria-labelledby="anzan-fundamentals-title">
            <header class="anzan-fundamentals-head">
                <h3 id="anzan-fundamentals-title">{content.title}</h3>
                <p>{content.intro}</p>
            </header>
            {chapters}
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formula_labels_are_parsed_with_both_minus_signs() {
        assert_eq!(parse_formula("+4"), Some((true, 4)));
        assert_eq!(parse_formula("−6"), Some((false, 6)));
        assert_eq!(parse_formula("-9"), Some((false, 9)));
        assert_eq!(parse_formula("0"), None);
        assert_eq!(parse_formula("x1"), None);
    }

    #[test]
    fn formulas_reproduce_the_expected_abacus_values() {
        assert_eq!(apply_formula(1, "+4"), Some(5));
        assert_eq!(apply_formula(0, "+4"), Some(4));
        assert_eq!(apply_formula(9, "+1"), Some(10));
        assert_eq!(apply_formula(5, "−1"), Some(4));
        assert_eq!(apply_formula(0, "−1"), None);
    }
}
