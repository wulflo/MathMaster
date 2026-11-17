// SPDX-License-Identifier: AGPL-3.0-or-later
//! Soroban interactivo de la UI.
//!
//! La UI no calcula: sólo aplica los toques que produce la geometría y dibuja el estado.
//! No hay bucle de dibujo ni asignaciones por frame; el DOM se actualiza cuando cambia el
//! estado o la etapa de fading.

use anzan_content::UiStrings;
use anzan_core::{
    DEFAULT_RODS, FadeProfile, FadeStage, Slot, Soroban, SorobanGeometry, Touch,
    geometry::{BEAM_HALF_GAP, BEAM_Y, PADDING_Y},
};
use leptos::prelude::*;

const SLOTS: [Slot; 5] = [
    Slot::Heaven,
    Slot::Earth(1),
    Slot::Earth(2),
    Slot::Earth(3),
    Slot::Earth(4),
];

/// Relación alto/ancho del tablero (`aspect-ratio: 4 / 3` en el CSS).
const BOARD_ASPECT: f32 = 4.0 / 3.0;

/// Reduce el rectángulo de un hueco a una ficha cuadrada centrada.
///
/// El botón conserva un área de impacto razonable, pero la ficha dibujada tiene forma de
/// cuenta en vez de estirarse por todo el hueco (sobre todo la celestial).
fn bead_rect(rect: anzan_core::geometry::Rect) -> (f32, f32, f32, f32) {
    let width = rect.width.min(rect.height / BOARD_ASPECT);
    let height = width * BOARD_ASPECT;
    (
        rect.x + (rect.width - width) / 2.0,
        rect.y + (rect.height - height) / 2.0,
        width,
        height,
    )
}

/// Panel interactivo del soroban.
#[component]
pub fn SorobanBoard(strings: UiStrings) -> impl IntoView {
    let UiStrings {
        soroban_label,
        value_label,
        value_hidden,
        show_value,
        hide_value,
        reset_label,
        fade_label,
        stage_full,
        stage_ghost,
        stage_rods,
        stage_unit_marks,
        stage_blank,
        ..
    } = strings;

    let soroban = RwSignal::new(Soroban::default());
    let stage = RwSignal::new(FadeStage::Full);
    let show = RwSignal::new(true);
    let pressed = RwSignal::new(None::<Touch>);
    let geometry = SorobanGeometry::for_rods(DEFAULT_RODS);

    let apply_touch = Callback::new(move |touch: Touch| {
        pressed.set(Some(touch));
        soroban.update(|value| {
            let _ = value.touch(touch);
        });
    });

    let title = StoredValue::new(soroban_label);
    let bead_label = title;

    let rod_lines = (0..DEFAULT_RODS)
        .rev()
        .filter_map(|rod| {
            let rect = geometry.rod_rect(rod)?;
            // Varilla fina centrada en la columna, no una columna entera.
            let width = (rect.width * 0.09).max(0.004);
            Some(view! {
                <span
                    class="anzan-rod-line"
                    style=format!(
                        "left:{:.3}%;top:{:.3}%;width:{:.3}%;height:{:.3}%;",
                        (rect.x + rect.width / 2.0 - width / 2.0) * 100.0,
                        rect.y * 100.0,
                        width * 100.0,
                        rect.height * 100.0,
                    )
                ></span>
            })
        })
        .collect_view();

    let beam_style = format!(
        "top:{:.3}%;height:{:.3}%;",
        (BEAM_Y - BEAM_HALF_GAP) * 100.0,
        (BEAM_HALF_GAP * 2.0) * 100.0,
    );

    let unit_marks = (0..DEFAULT_RODS)
        .rev()
        .filter_map(|rod| {
            let rect = geometry.rod_rect(rod)?;
            Some(view! {
                <span
                    class="anzan-unit-mark"
                    style=format!(
                        "left:{:.3}%;top:{:.3}%;width:{:.3}%;height:1.2%;",
                        rect.x * 100.0,
                        (1.0 - PADDING_Y - 0.012) * 100.0,
                        rect.width * 100.0,
                    )
                ></span>
            })
        })
        .collect_view();

    let beads = (0..DEFAULT_RODS)
        .rev()
        .flat_map(|rod| {
            SLOTS.into_iter().filter_map(move |slot| {
                let (x, y, width, height) = bead_rect(geometry.slot_rect(rod, slot)?);
                let touch = Touch::new(rod, slot);
                let is_active = move || {
                    soroban.get().rod(rod).is_some_and(|rod| match slot {
                        Slot::Heaven => rod.heaven(),
                        Slot::Earth(index) => rod.earth() >= index,
                    })
                };
                let is_pressed = move || pressed.get() == Some(touch);

                Some(view! {
                    <button
                        type="button"
                        class=move || {
                            let mut class = String::from("anzan-bead");
                            if is_active() {
                                class.push_str(" is-active");
                            }
                            if is_pressed() {
                                class.push_str(" is-pressed");
                            }
                            class
                        }
                        style=format!(
                            "left:{:.3}%;top:{:.3}%;width:{:.3}%;height:{:.3}%;",
                            x * 100.0,
                            y * 100.0,
                            width * 100.0,
                            height * 100.0,
                        )
                        aria-pressed=move || if is_active() { "true" } else { "false" }
                        aria-label=move || {
                            format!(
                                "{} {} {}",
                                bead_label.get_value(),
                                rod,
                                slot.index(),
                            )
                        }
                        on:pointerdown=move |_| apply_touch.run(touch)
                        on:pointerup=move |_| pressed.set(None)
                        on:pointerleave=move |_| pressed.set(None)
                        on:keydown=move |event: leptos::ev::KeyboardEvent| {
                            let key = event.key();
                            if key == "Enter" || key == " " {
                                event.prevent_default();
                                apply_touch.run(touch);
                            }
                        }
                        on:keyup=move |_| pressed.set(None)
                    ></button>
                })
            })
        })
        .collect_view();

    let stages = [
        (FadeStage::Full, stage_full),
        (FadeStage::Ghost, stage_ghost),
        (FadeStage::Rods, stage_rods),
        (FadeStage::UnitMarks, stage_unit_marks),
        (FadeStage::Blank, stage_blank),
    ];

    let stage_buttons = stages
        .into_iter()
        .map(|(value, label)| {
            let is_active = move || stage.get() == value;
            view! {
                <button
                    type="button"
                    class="anzan-stage"
                    class:is-active=is_active
                    aria-pressed=move || if is_active() { "true" } else { "false" }
                    on:click=move |_| stage.set(value)
                >
                    {label}
                </button>
            }
        })
        .collect_view();

    let container_style = move || {
        let profile = FadeProfile::for_stage(stage.get());
        format!(
            "--anzan-beads-opacity:{:.2};--anzan-rods-opacity:{:.2};--anzan-unit-opacity:{:.2};",
            profile.beads, profile.rods, profile.unit_marks,
        )
    };

    let show_label = show_value;
    let hide_label = hide_value;
    let value_caption = value_label;
    let hidden_caption = value_hidden;
    let fade_aria = fade_label.clone();

    view! {
        <section class="anzan-soroban-panel" aria-labelledby="anzan-soroban-title">
            <header class="anzan-soroban-head">
                <h2 id="anzan-soroban-title">{title.get_value()}</h2>
                <output class="anzan-value" aria-live="polite">
                    {move || {
                        if show.get() {
                            format!("{} {}", value_caption, soroban.get().value())
                        } else {
                            hidden_caption.clone()
                        }
                    }}
                </output>
            </header>

            <div class="anzan-soroban" style=container_style>
                {rod_lines}
                <span class="anzan-beam" style=beam_style></span>
                {beads}
                {unit_marks}
            </div>

            <div class="anzan-controls">
                <span class="anzan-control-label">{fade_label}</span>
                <div class="anzan-stage-buttons" role="group" aria-label=fade_aria>
                    {stage_buttons}
                </div>
                <button
                    type="button"
                    class="anzan-toggle"
                    on:click=move |_| show.update(|visible| *visible = !*visible)
                >
                    {move || if show.get() { hide_label.clone() } else { show_label.clone() }}
                </button>
                <button
                    type="button"
                    class="anzan-reset"
                    on:click=move |_| soroban.set(Soroban::default())
                >
                    {reset_label}
                </button>
            </div>
        </section>
    }
}
