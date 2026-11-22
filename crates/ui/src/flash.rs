// SPDX-License-Identifier: AGPL-3.0-or-later
//! Modo Flash Anzan y sus controles de entrenamiento.
//!
//! El componente genera un drill determinista, construye un plan en frames y lo reproduce
//! con `requestAnimationFrame`. En el callback sólo se consulta el plan, se escribe el
//! `textContent` del display y se alimenta la telemetría de tamaño fijo: no hay
//! asignaciones por frame. El audio web se dispara una vez por estímulo, no por frame.

#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code, unused_imports))]

use crate::answer::{Action, AnswerBox};
use crate::storage::read_key;
#[cfg(target_arch = "wasm32")]
use crate::storage::{tauri_invoke, write_key};
use anzan_content::UiStrings;
use anzan_engine::{
    DAY_MS, DrillSpeed, FramePlan, FrameTelemetry, LevelChange, LevelSpec, MixedConfig, MixedDrill,
    PedagogicalCase, ProgressState, ReviewOutcome, ReviewState, SessionRecord, TermSign,
    case_stats, classify_addition, decode_log, encode_log, to_csv,
};
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Frames que permanece visible cada término en práctica libre (500 ms a 60 Hz).
const STIMULUS_FRAMES: u32 = 30;
/// Frames de hueco entre términos (100 ms a 60 Hz).
const GAP_FRAMES: u32 = 6;
/// Frames de la cuenta regresiva de fijación (300 ms por número).
const COUNTDOWN_FRAMES: u32 = 18;
/// Frames de hueco durante la cuenta regresiva.
const COUNTDOWN_GAP_FRAMES: u32 = 6;
/// Cantidad de números de la cuenta regresiva.
const COUNTDOWN_STEPS: usize = 3;
/// Cadencia nominal usada para planificar; la telemetría mide la real.
const NOMINAL_FRAME_MS: f64 = 1000.0 / 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Running,
    Answer,
    Result,
}

#[derive(Debug, Clone)]
struct CaseLabels {
    direct: String,
    complement5: String,
    complement10: String,
    carry: String,
}

impl CaseLabels {
    fn label(&self, case: PedagogicalCase) -> String {
        match case {
            PedagogicalCase::Direct => self.direct.clone(),
            PedagogicalCase::Complement5 => self.complement5.clone(),
            PedagogicalCase::Complement10 => self.complement10.clone(),
            PedagogicalCase::Carry => self.carry.clone(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
type FrameClosure = wasm_bindgen::closure::Closure<dyn FnMut()>;
#[cfg(not(target_arch = "wasm32"))]
type FrameClosure = ();

#[cfg(target_arch = "wasm32")]
struct WebAudio {
    context: web_sys::AudioContext,
    click: web_sys::AudioBuffer,
}

#[cfg(target_arch = "wasm32")]
impl WebAudio {
    fn new() -> Option<Self> {
        use wasm_bindgen::JsCast;

        let context = web_sys::AudioContext::new().ok()?;
        let _ = web_sys::AudioContext::resume(&context);
        let base: &web_sys::BaseAudioContext = context.unchecked_ref();
        let sample_rate = base.sample_rate();
        let rate = if sample_rate.is_finite() && sample_rate > 0.0 {
            sample_rate.round() as u32
        } else {
            anzan_engine::DEFAULT_SAMPLE_RATE
        };
        let click = anzan_engine::audio::ClickBuffer::new(rate).ok()?;
        let buffer = base
            .create_buffer(1, click.samples().len() as u32, sample_rate)
            .ok()?;
        buffer.copy_to_channel(click.samples(), 0).ok()?;
        Some(Self {
            context,
            click: buffer,
        })
    }

    /// Dispara un clic ahora mismo. Asigna un nodo por estímulo, nunca por frame.
    fn click(&mut self) {
        use wasm_bindgen::JsCast;

        let base: &web_sys::BaseAudioContext = self.context.unchecked_ref();
        let destination = base.destination();
        if let Ok(source) = base.create_buffer_source() {
            source.set_buffer(Some(&self.click));
            let connected = web_sys::AudioNode::connect_with_audio_node(
                source.unchecked_ref(),
                destination.unchecked_ref(),
            );
            if connected.is_ok() {
                let _ = source.start_with_when(base.current_time());
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct WebAudio;

/// Estado mutable de una reproducción en curso.
struct Run {
    plan: FramePlan,
    terms: Vec<String>,
    telemetry: FrameTelemetry,
    frame: u32,
    last_stimulus: Option<u32>,
    started_ms: Option<f64>,
    countdown: bool,
}

/// Ajustes persistentes del modo Flash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DrillSettings {
    digits: u8,
    terms: u8,
    sound: bool,
    mixed: bool,
    exposure_ms: u16,
    gap_ms: u16,
    countdown: bool,
}

impl Default for DrillSettings {
    fn default() -> Self {
        Self {
            digits: 2,
            terms: 5,
            sound: true,
            mixed: false,
            exposure_ms: 500,
            gap_ms: 100,
            countdown: true,
        }
    }
}

impl DrillSettings {
    fn decode(raw: &str) -> Self {
        let defaults = Self::default();
        let parts: Vec<&str> = raw.split(',').collect();
        let number = |index: usize, fallback: u16| -> u16 {
            parts
                .get(index)
                .and_then(|value| value.trim().parse::<u16>().ok())
                .unwrap_or(fallback)
        };
        let digits = number(0, u16::from(defaults.digits));
        let terms = number(1, u16::from(defaults.terms));
        let sound = parts.get(2).is_none_or(|value| *value != "0");
        let mixed = parts.get(3).is_some_and(|value| *value == "1");
        let exposure_ms = number(4, defaults.exposure_ms);
        let gap_ms = number(5, defaults.gap_ms);
        let countdown = parts.get(6).is_none_or(|value| *value != "0");
        Self {
            digits: u8::try_from(digits).unwrap_or(defaults.digits).clamp(1, 3),
            terms: u8::try_from(terms).unwrap_or(defaults.terms).clamp(2, 10),
            sound,
            mixed,
            exposure_ms: exposure_ms.clamp(100, 1500),
            gap_ms: gap_ms.min(1500),
            countdown,
        }
    }

    fn encode(self) -> String {
        format!(
            "{},{},{},{},{},{},{}",
            self.digits,
            self.terms,
            u8::from(self.sound),
            u8::from(self.mixed),
            self.exposure_ms,
            self.gap_ms,
            u8::from(self.countdown)
        )
    }
}

/// Argumentos compartidos por los distintos lanzamientos de un drill.
#[derive(Clone)]
struct LaunchArgs {
    digits: RwSignal<u8>,
    terms: RwSignal<u8>,
    sound: RwSignal<bool>,
    mixed: RwSignal<bool>,
    countdown: RwSignal<bool>,
    exposure_ms: RwSignal<u16>,
    gap_ms: RwSignal<u16>,
    progressive: RwSignal<bool>,
    progress: RwSignal<ProgressState>,
    active_level: RwSignal<u8>,
    active_cases: RwSignal<Vec<PedagogicalCase>>,
    last_seed: RwSignal<u64>,
    launched_ms: RwSignal<f64>,
    phase: RwSignal<Phase>,
    expected: RwSignal<Option<u128>>,
    answer: RwSignal<String>,
    result: RwSignal<Option<bool>>,
    display: NodeRef<leptos::html::Div>,
    audio: Rc<RefCell<Option<WebAudio>>>,
    handle: Rc<RefCell<Option<FrameClosure>>>,
}

const SETTINGS_KEY: &str = "anzan.drill";
const PROGRESS_KEY: &str = "anzan.progress";
const PROGRESSIVE_KEY: &str = "anzan.progressive";
const SESSIONS_KEY: &str = "anzan.sessions";
const REVIEW_KEY_PREFIX: &str = "anzan.review.";

#[cfg(target_arch = "wasm32")]
fn load_settings() -> DrillSettings {
    read_key(SETTINGS_KEY).map_or_else(DrillSettings::default, |raw| DrillSettings::decode(&raw))
}

#[cfg(target_arch = "wasm32")]
fn save_settings(settings: DrillSettings) {
    write_key(SETTINGS_KEY, &settings.encode());
}

#[cfg(not(target_arch = "wasm32"))]
fn load_settings() -> DrillSettings {
    DrillSettings::default()
}

#[cfg(not(target_arch = "wasm32"))]
fn save_settings(_settings: DrillSettings) {}

#[cfg(target_arch = "wasm32")]
fn initial_progress() -> ProgressState {
    read_key(PROGRESS_KEY)
        .and_then(|raw| ProgressState::decode(&raw))
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
fn save_progress(progress: ProgressState) {
    write_key(PROGRESS_KEY, &progress.encode());
}

#[cfg(target_arch = "wasm32")]
fn initial_progressive() -> bool {
    read_key(PROGRESSIVE_KEY).is_none_or(|value| value != "0")
}

#[cfg(target_arch = "wasm32")]
fn save_progressive(enabled: bool) {
    write_key(PROGRESSIVE_KEY, if enabled { "1" } else { "0" });
}

#[cfg(not(target_arch = "wasm32"))]
fn initial_progress() -> ProgressState {
    ProgressState::default()
}

#[cfg(not(target_arch = "wasm32"))]
fn save_progress(_progress: ProgressState) {}

#[cfg(not(target_arch = "wasm32"))]
fn initial_progressive() -> bool {
    true
}

#[cfg(not(target_arch = "wasm32"))]
fn save_progressive(_enabled: bool) {}

#[cfg(target_arch = "wasm32")]
fn load_sessions() -> Vec<SessionRecord> {
    read_key(SESSIONS_KEY).map_or_else(Vec::new, |raw| decode_log(&raw))
}

#[cfg(target_arch = "wasm32")]
fn save_sessions(sessions: &[SessionRecord]) {
    write_key(SESSIONS_KEY, &encode_log(sessions));
}

#[cfg(target_arch = "wasm32")]
fn load_review(level: u8) -> ReviewState {
    read_key(&format!("{REVIEW_KEY_PREFIX}{level}"))
        .and_then(|raw| ReviewState::decode(&raw))
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
fn save_review(level: u8, state: ReviewState) {
    write_key(&format!("{REVIEW_KEY_PREFIX}{level}"), &state.encode());
}

#[cfg(not(target_arch = "wasm32"))]
fn load_sessions() -> Vec<SessionRecord> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
fn save_sessions(_sessions: &[SessionRecord]) {}

#[cfg(not(target_arch = "wasm32"))]
fn load_review(_level: u8) -> ReviewState {
    ReviewState::default()
}

#[cfg(not(target_arch = "wasm32"))]
fn save_review(_level: u8, _state: ReviewState) {}

#[cfg(target_arch = "wasm32")]
fn now_ms() -> u64 {
    js_sys::Date::now().max(0.0) as u64
}

#[cfg(target_arch = "wasm32")]
fn performance_now() -> f64 {
    web_sys::window()
        .and_then(|window| window.performance())
        .map_or(0.0, |performance| performance.now())
}

#[cfg(target_arch = "wasm32")]
fn format_timestamp(timestamp_ms: u64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(timestamp_ms as f64));
    let iso = String::from(date.to_iso_string());
    if iso.len() >= 19 {
        format!("{} {}", &iso[..10], &iso[11..19])
    } else {
        iso
    }
}

#[cfg(target_arch = "wasm32")]
fn download_csv(csv: &str) {
    use wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let parts = js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(csv));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("text/csv;charset=utf-8");
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options) else {
        return;
    };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
        return;
    };
    if let Ok(element) = document.create_element("a")
        && let Ok(anchor) = element.dyn_into::<web_sys::HtmlAnchorElement>()
    {
        anchor.set_href(&url);
        anchor.set_download("mathmaster-sessions.csv");
        anchor.click();
    }
    let _ = web_sys::Url::revoke_object_url(&url);
}

#[cfg(not(target_arch = "wasm32"))]
fn now_ms() -> u64 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
fn performance_now() -> f64 {
    0.0
}

#[cfg(not(target_arch = "wasm32"))]
fn format_timestamp(timestamp_ms: u64) -> String {
    timestamp_ms.to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn download_csv(_csv: &str) {}

fn fill(template: &str, value: u64) -> String {
    template.replace("{n}", &value.to_string())
}

/// Indica si la UI corre dentro del host Tauri.
#[cfg(target_arch = "wasm32")]
fn native_desktop() -> bool {
    use wasm_bindgen::JsValue;

    web_sys::window()
        .and_then(|window| {
            js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
                .ok()
                .map(|value| !value.is_undefined() && !value.is_null())
        })
        .unwrap_or(false)
}

#[cfg(not(target_arch = "wasm32"))]
fn native_desktop() -> bool {
    false
}

/// Encola un clic nativo sin esperar la respuesta (una vez por estímulo).
#[cfg(target_arch = "wasm32")]
fn native_click() {
    let _ = tauri_invoke("audio_click");
}

#[cfg(not(target_arch = "wasm32"))]
fn native_click() {}

/// Construye el plan de frames: cuenta regresiva opcional y términos del drill.
fn build_plan(spec: LevelSpec, countdown: bool, speed: Option<DrillSpeed>) -> Option<FramePlan> {
    let mut builder = FramePlan::builder();
    if countdown {
        for _ in 0..COUNTDOWN_STEPS {
            builder = builder
                .stimulus(COUNTDOWN_FRAMES)
                .and_then(|next| next.gap(COUNTDOWN_GAP_FRAMES))
                .ok()?;
        }
    }
    let exposure = speed.map_or(spec.stimulus_frames, |value| {
        value.exposure_frames(NOMINAL_FRAME_MS)
    });
    let gap = speed.map_or(spec.gap_frames, |value| value.gap_frames(NOMINAL_FRAME_MS));
    for _ in 0..spec.terms {
        builder = builder
            .stimulus(exposure.max(1))
            .and_then(|next| next.gap(gap))
            .ok()?;
    }
    builder.build().ok()
}

fn spec_for(progressive: bool, progress: ProgressState, digits: u8, terms: u8) -> LevelSpec {
    if progressive {
        progress.spec()
    } else {
        LevelSpec {
            digits,
            terms,
            stimulus_frames: STIMULUS_FRAMES,
            gap_frames: GAP_FRAMES,
        }
    }
}

/// Lanza un drill; `seed_override` permite repetir exactamente la misma secuencia.
#[cfg(target_arch = "wasm32")]
fn launch(args: LaunchArgs, seed_override: Option<u64>) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(performance) = window.performance() else {
        return;
    };
    let Some(display) = args.display.get() else {
        return;
    };

    let progressive = args.progressive.get_untracked();
    let progress = args.progress.get_untracked();
    let spec = spec_for(
        progressive,
        progress,
        args.digits.get_untracked().clamp(1, 3),
        args.terms.get_untracked().clamp(2, 10),
    );
    let speed = if progressive {
        None
    } else {
        DrillSpeed {
            exposure_ms: args.exposure_ms.get_untracked(),
            gap_ms: args.gap_ms.get_untracked(),
        }
        .validate()
        .ok()
    };
    let countdown = args.countdown.get_untracked();
    let mixed = args.mixed.get_untracked();
    let seed = seed_override.unwrap_or_else(|| performance.now().to_bits());
    let config = anzan_engine::DrillConfig {
        digits: spec.digits.clamp(1, 6),
        terms: spec.terms.clamp(2, 20),
    };

    let mut terms: Vec<String> = Vec::with_capacity(usize::from(config.terms) + COUNTDOWN_STEPS);
    let mut cases = Vec::new();
    let answer;
    if mixed {
        let config_mixed = MixedConfig {
            digits: config.digits,
            terms: config.terms,
        };
        let Ok(drill) = MixedDrill::generate(config_mixed, seed) else {
            return;
        };
        let mut previous = 0u128;
        for term in drill.terms() {
            match term.sign {
                TermSign::Add => terms.push(term.value.to_string()),
                TermSign::Subtract => {
                    terms.push(format!("−{}", term.value));
                }
            }
            let partial = previous;
            previous = match term.sign {
                TermSign::Add => previous + u128::from(term.value),
                TermSign::Subtract => previous.saturating_sub(u128::from(term.value)),
            };
            cases.push(if previous >= partial {
                classify_addition(partial, previous - partial)
            } else {
                PedagogicalCase::Direct
            });
        }
        answer = drill.answer();
    } else {
        let Ok(drill) = anzan_engine::Drill::generate(config, seed) else {
            return;
        };
        terms.extend(drill.terms().iter().map(u32::to_string));
        cases = drill.cases();
        answer = drill.answer();
    }

    let Some(plan) = build_plan(spec, countdown, speed) else {
        return;
    };
    if countdown {
        for step in (1..=COUNTDOWN_STEPS).rev() {
            terms.insert(0, step.to_string());
        }
    }

    args.active_level.set(progress.level);
    args.active_cases.set(cases);
    args.last_seed.set(seed);
    args.launched_ms.set(performance.now());
    args.expected.set(Some(answer));
    args.answer.set(String::new());
    args.result.set(None);
    args.phase.set(Phase::Running);

    save_settings(DrillSettings {
        digits: config.digits.clamp(1, 3),
        terms: config.terms.clamp(2, 10),
        sound: args.sound.get_untracked(),
        mixed,
        exposure_ms: args.exposure_ms.get_untracked(),
        gap_ms: args.gap_ms.get_untracked(),
        countdown,
    });

    let native = native_desktop();
    if native {
        let _ = tauri_invoke("audio_start");
    }
    if args.sound.get_untracked() && !native && args.audio.borrow().is_none() {
        *args.audio.borrow_mut() = WebAudio::new();
    }

    if countdown {
        display.set_class_name("anzan-flash is-fixation");
    } else {
        display.set_class_name("anzan-flash");
    }

    let run = Rc::new(RefCell::new(Run {
        plan,
        terms,
        telemetry: FrameTelemetry::new(),
        frame: 0,
        last_stimulus: None,
        started_ms: None,
        countdown,
    }));

    let holder = args.handle.clone();
    let next = args.handle.clone();
    let run_for_closure = run.clone();
    let audio = args.audio.clone();
    let sound = args.sound;
    let phase = args.phase;
    let display_for_closure = display.clone();
    let window_for_closure = window.clone();

    *holder.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let audio_enabled = sound.get_untracked();
        let now = window_for_closure
            .performance()
            .map_or(0.0, |performance| performance.now());
        let finished = {
            let mut state = run_for_closure.borrow_mut();
            let _ = state.telemetry.record_frame(now);
            if state.started_ms.is_none() {
                state.started_ms = Some(now);
            }
            let index = state.plan.stimulus_at(state.frame);
            if index != state.last_stimulus {
                let text = index
                    .and_then(|value| state.terms.get(value as usize))
                    .map_or("", String::as_str);
                display_for_closure.set_text_content(Some(text));
                if state.countdown
                    && index.is_some_and(|value| value < COUNTDOWN_STEPS as u32)
                    && index != Some(0)
                {
                    display_for_closure.set_class_name("anzan-flash");
                }
                if let Some(segment) = state.plan.segment_at(state.frame) {
                    let frame_ms = state.telemetry.mean_frame_ms().unwrap_or(NOMINAL_FRAME_MS);
                    let elapsed = state.started_ms.map_or(0.0, |started| now - started);
                    let _ = state
                        .telemetry
                        .record_onset(f64::from(segment.start) * frame_ms, elapsed);
                }
                if index.is_some()
                    && audio_enabled
                    && index >= Some(u32::try_from(COUNTDOWN_STEPS).unwrap_or(0))
                {
                    if native {
                        native_click();
                    } else if let Some(audio) = audio.borrow_mut().as_mut() {
                        audio.click();
                    }
                }
                state.last_stimulus = index;
            }
            state.frame = state.frame.saturating_add(1);
            let finished = state.frame >= state.plan.total_frames();
            if finished {
                display_for_closure.set_text_content(Some(""));
                display_for_closure.set_class_name("anzan-flash");
                phase.set(Phase::Answer);
            }
            finished
        };

        if !finished && let Some(callback) = next.borrow().as_ref() {
            let _ = window_for_closure.request_animation_frame(callback.as_ref().unchecked_ref());
        }
    }) as Box<dyn FnMut()>));

    if let Some(callback) = holder.borrow().as_ref() {
        let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn launch(_args: LaunchArgs, _seed_override: Option<u64>) {}

/// Panel Flash Anzan completo.
#[component]
pub fn FlashAnzan(
    strings: UiStrings,
    /// Señal compartida con la cabecera: (nivel, racha).
    #[prop(optional)]
    header_stats: Option<RwSignal<(u8, u32)>>,
    /// Se incrementa al terminar un drill para refrescar el panel de progreso.
    #[prop(optional)]
    refresh: Option<RwSignal<u32>>,
    /// Petición de resolución en el soroban: (inicio, ¿suma?, cantidad).
    #[prop(optional)]
    demo_request: Option<RwSignal<Option<(u32, bool, u32)>>>,
) -> impl IntoView {
    let UiStrings {
        drill_title,
        drill_intro,
        drill_digits,
        drill_terms,
        drill_sound,
        drill_start,
        drill_answer: _,
        drill_submit: _,
        drill_correct,
        drill_wrong,
        drill_running,
        drill_ready,
        drill_countdown: _,
        drill_fixation,
        drill_mixed,
        drill_exposure,
        drill_gap,
        drill_frames,
        drill_cadence,
        minus_sign: _,
        progress_title,
        progress_level,
        progress_accuracy,
        progress_streak,
        progress_attempts,
        progress_best,
        progress_reset,
        progress_promoted,
        progress_demoted,
        progress_mode,
        progress_free,
        session_title,
        session_empty,
        session_export,
        session_elapsed,
        case_direct,
        case_complement5,
        case_complement10,
        case_carry,
        case_stats_title,
        case_stats_accuracy,
        review_title,
        review_now,
        review_hours,
        review_days,
        answer_auto_advance,
        answer_repeat,
        ..
    } = strings.clone();

    let initial = load_settings();
    let progress = RwSignal::new(initial_progress());
    let progressive = RwSignal::new(initial_progressive());
    let initial_spec = progress.get_untracked().spec();
    let digits = RwSignal::new(if progressive.get_untracked() {
        initial_spec.digits
    } else {
        initial.digits
    });
    let terms = RwSignal::new(if progressive.get_untracked() {
        initial_spec.terms
    } else {
        initial.terms
    });
    let sound = RwSignal::new(initial.sound);
    let mixed = RwSignal::new(initial.mixed);
    let countdown = RwSignal::new(initial.countdown);
    let exposure_ms = RwSignal::new(initial.exposure_ms);
    let gap_ms = RwSignal::new(initial.gap_ms);
    let auto_advance = RwSignal::new(true);
    let sessions = RwSignal::new(load_sessions());
    let reviews = RwSignal::new((1..=6).map(load_review).collect::<Vec<_>>());
    let active_level = RwSignal::new(progress.get_untracked().level);
    let active_cases = RwSignal::new(Vec::<PedagogicalCase>::new());
    let last_seed = RwSignal::new(0u64);
    let launched_ms = RwSignal::new(0.0f64);
    let level_change = RwSignal::new(None::<LevelChange>);
    let phase = RwSignal::new(Phase::Idle);
    let expected = RwSignal::new(None::<u128>);
    let answer = RwSignal::new(String::new());
    let result = RwSignal::new(None::<bool>);
    let display = NodeRef::<leptos::html::Div>::new();

    let audio: Rc<RefCell<Option<WebAudio>>> = Rc::new(RefCell::new(None));
    let handle: Rc<RefCell<Option<FrameClosure>>> = Rc::new(RefCell::new(None));

    let launch_args = LaunchArgs {
        digits,
        terms,
        sound,
        mixed,
        countdown,
        exposure_ms,
        gap_ms,
        progressive,
        progress,
        active_level,
        active_cases,
        last_seed,
        launched_ms,
        phase,
        expected,
        answer,
        result,
        display,
        audio: audio.clone(),
        handle: handle.clone(),
    };

    let args_start = launch_args.clone();
    let on_start = move |_| launch(args_start.clone(), None);
    let args_repeat = launch_args.clone();
    let on_repeat = move |_| launch(args_repeat.clone(), Some(last_seed.get_untracked()));
    let args_answer_next = launch_args.clone();

    let complete: Rc<dyn Fn(bool)> = Rc::new(move |correct: bool| {
        let level = active_level.get_untracked();
        let elapsed = performance_now() - launched_ms.get_untracked();
        let elapsed_ms = if elapsed.is_finite() && elapsed > 0.0 {
            elapsed.min(f64::from(u32::MAX)) as u32
        } else {
            0
        };
        let case = active_cases
            .get_untracked()
            .into_iter()
            .max()
            .unwrap_or(PedagogicalCase::Direct);
        let record = SessionRecord {
            timestamp_ms: now_ms(),
            level,
            digits: digits.get_untracked(),
            terms: terms.get_untracked(),
            correct,
            elapsed_ms,
            case,
        };
        let mut history = sessions.get_untracked();
        history.push(record);
        if history.len() > anzan_engine::MAX_SESSIONS {
            let excess = history.len() - anzan_engine::MAX_SESSIONS;
            history.drain(..excess);
        }
        save_sessions(&history);
        sessions.set(history);

        let mut book = reviews.get_untracked();
        if let Some(slot) = book.get_mut(usize::from(level).saturating_sub(1)) {
            let outcome = if correct {
                ReviewOutcome::Good
            } else {
                ReviewOutcome::Again
            };
            *slot = slot.review(outcome, now_ms());
            save_review(level, *slot);
        }
        reviews.set(book);

        let mut next_progress = progress.get_untracked();
        let change = next_progress.record(correct);
        progress.set(next_progress);
        save_progress(next_progress);
        if let Some(stats) = header_stats {
            stats.set((next_progress.level, next_progress.streak));
        }
        if let Some(refresh) = refresh {
            refresh.update(|value| *value = value.wrapping_add(1));
        }
        if progressive.get_untracked() {
            let spec = next_progress.spec();
            digits.set(spec.digits);
            terms.set(spec.terms);
        }
        level_change.set(Some(change));
        result.set(Some(correct));
        phase.set(Phase::Result);
    });

    let correct_cb = complete.clone();
    let on_correct = move || correct_cb(true);
    let wrong_cb = complete.clone();
    let on_wrong = move || wrong_cb(false);
    let next_action = move || launch(args_answer_next.clone(), None);
    let show_solution = move || {
        if let Some(request) = demo_request {
            let value = expected.get_untracked().unwrap_or(0);
            request.set(Some((0, true, u32::try_from(value).unwrap_or(u32::MAX))));
        }
    };

    let on_progressive = move |event: leptos::ev::Event| {
        let enabled = event_target_checked(&event);
        progressive.set(enabled);
        save_progressive(enabled);
        if enabled {
            let spec = progress.get_untracked().spec();
            digits.set(spec.digits);
            terms.set(spec.terms);
        }
    };

    let on_reset_progress = move |_| {
        let fresh = ProgressState::default();
        progress.set(fresh);
        save_progress(fresh);
        level_change.set(None);
        result.set(None);
        if progressive.get_untracked() {
            let spec = fresh.spec();
            digits.set(spec.digits);
            terms.set(spec.terms);
        }
    };

    let status = move || match phase.get() {
        Phase::Running => drill_running.clone(),
        Phase::Idle | Phase::Answer | Phase::Result => drill_ready.clone(),
    };
    let running = move || phase.get() == Phase::Running;
    let answered = move || matches!(phase.get(), Phase::Answer | Phase::Result);
    let text_correct = drill_correct.clone();
    let text_wrong = drill_wrong.clone();
    let result_text = move || match result.get() {
        Some(true) => text_correct.clone(),
        Some(false) => text_wrong.clone(),
        None => String::new(),
    };

    let speed_text = move || {
        let speed = DrillSpeed {
            exposure_ms: exposure_ms.get(),
            gap_ms: gap_ms.get(),
        }
        .validate()
        .unwrap_or(DrillSpeed::DEFAULT);
        let frames = speed.exposure_frames(NOMINAL_FRAME_MS) + speed.gap_frames(NOMINAL_FRAME_MS);
        format!(
            "{} · {}",
            drill_frames.replace("{n}", &frames.to_string()),
            drill_cadence.replace("{n}", &format!("{:.2}", speed.terms_per_second()))
        )
    };

    let level_label = progress_level.clone();
    let accuracy_label = progress_accuracy.clone();
    let streak_label = progress_streak.clone();
    let attempts_label = progress_attempts.clone();
    let best_label = progress_best.clone();
    let mode_label = progress_mode.clone();
    let level_text = move || format!("{} {}", level_label, progress.get().level);
    let accuracy_text = move || {
        format!(
            "{} {:.0}%",
            accuracy_label,
            progress.get().accuracy() * 100.0
        )
    };
    let streak_text = move || format!("{} {}", streak_label, progress.get().streak);
    let attempts_text = move || format!("{} {}", attempts_label, progress.get().attempts);
    let best_text = move || format!("{} {}", best_label, progress.get().best_streak);
    let mode_text = move || {
        if progressive.get() {
            mode_label.clone()
        } else {
            progress_free.clone()
        }
    };
    let change_text = move || match level_change.get() {
        Some(LevelChange::Promoted(level)) => format!("{} {level}", progress_promoted),
        Some(LevelChange::Demoted(level)) => format!("{} {level}", progress_demoted),
        Some(LevelChange::None) | None => String::new(),
    };

    let case_labels = CaseLabels {
        direct: case_direct,
        complement5: case_complement5,
        complement10: case_complement10,
        carry: case_carry,
    };
    let row_correct = drill_correct.clone();
    let row_wrong = drill_wrong.clone();
    let row_elapsed = session_elapsed.clone();
    let stats_accuracy = case_stats_accuracy.clone();
    let history_labels = case_labels.clone();
    let stats_labels = case_labels.clone();

    let review_text = move || {
        let level = progress.get().level;
        let state = reviews
            .get()
            .get(usize::from(level).saturating_sub(1))
            .copied()
            .unwrap_or_default();
        let now = now_ms();
        if state.is_due(now) {
            review_now.clone()
        } else {
            let remaining = state.due_ms.saturating_sub(now);
            if remaining < DAY_MS {
                fill(&review_hours, remaining.div_ceil(3_600_000).max(1))
            } else {
                fill(&review_days, remaining.div_ceil(DAY_MS))
            }
        }
    };

    let history_rows = move || {
        let labels = history_labels.clone();
        sessions
            .get()
            .iter()
            .rev()
            .take(6)
            .map(|session| {
                let outcome = if session.correct {
                    row_correct.clone()
                } else {
                    row_wrong.clone()
                };
                view! {
                    <li>
                        <span class="anzan-session-when">{format_timestamp(session.timestamp_ms)}</span>
                        <span class="anzan-session-level">{format!("L{}", session.level)}</span>
                        <span>{format!("{} {} ms", row_elapsed, session.elapsed_ms)}</span>
                        <span>{labels.label(session.case)}</span>
                        <span class="anzan-session-outcome">{outcome}</span>
                    </li>
                }
                .into_any()
            })
            .collect_view()
    };

    let stats_rows = move || {
        let labels = stats_labels.clone();
        case_stats(&sessions.get())
            .into_iter()
            .map(|stat| {
                let accuracy = if stat.attempts == 0 {
                    0
                } else {
                    (stat.accuracy() * 100.0).round() as u32
                };
                view! {
                    <div>
                        <dt>{labels.label(stat.case)}</dt>
                        <dd>{format!("{} {}/{} ({}%)", stats_accuracy, stat.correct, stat.attempts, accuracy)}</dd>
                    </div>
                }
                .into_any()
            })
            .collect_view()
    };

    let on_export = move |_| download_csv(&to_csv(&sessions.get_untracked()));

    view! {
        <section class="anzan-drill" aria-labelledby="anzan-drill-title">
            <header class="anzan-soroban-head">
                <h2 id="anzan-drill-title">{drill_title}</h2>
                <output class="anzan-drill-status" aria-live="polite">{status}</output>
            </header>
            <p class="anzan-drill-intro">{drill_intro}</p>

            <div class="anzan-flash" node_ref=display aria-hidden="true"></div>

            <div class="anzan-drill-config">
                <label class="anzan-field">
                    <span>{drill_digits}</span>
                    <input type="number" min="1" max="3"
                        prop:value=move || digits.get()
                        disabled=move || running() || progressive.get()
                        on:input=move |event| {
                            if let Ok(value) = event_target_value(&event).parse::<u8>()
                                && (1..=3).contains(&value) { digits.set(value); }
                        }
                    />
                </label>
                <label class="anzan-field">
                    <span>{drill_terms}</span>
                    <input type="number" min="2" max="10"
                        prop:value=move || terms.get()
                        disabled=move || running() || progressive.get()
                        on:input=move |event| {
                            if let Ok(value) = event_target_value(&event).parse::<u8>()
                                && (2..=10).contains(&value) { terms.set(value); }
                        }
                    />
                </label>
                <label class="anzan-field">
                    <span>{drill_exposure}</span>
                    <input type="number" min="100" max="1500" step="50"
                        prop:value=move || exposure_ms.get()
                        disabled=move || running() || progressive.get()
                        on:input=move |event| {
                            if let Ok(value) = event_target_value(&event).parse::<u16>()
                                && (100..=1500).contains(&value) { exposure_ms.set(value); }
                        }
                    />
                </label>
                <label class="anzan-field">
                    <span>{drill_gap}</span>
                    <input type="number" min="0" max="1500" step="50"
                        prop:value=move || gap_ms.get()
                        disabled=move || running() || progressive.get()
                        on:input=move |event| {
                            if let Ok(value) = event_target_value(&event).parse::<u16>()
                                && value <= 1500 { gap_ms.set(value); }
                        }
                    />
                </label>
                <p class="anzan-speed-readout">{speed_text}</p>
                <label class="anzan-field anzan-check">
                    <input type="checkbox" prop:checked=move || progressive.get()
                        disabled=running on:change=on_progressive />
                    <span>{progress_mode}</span>
                </label>
                <label class="anzan-field anzan-check">
                    <input type="checkbox" prop:checked=move || mixed.get()
                        disabled=running
                        on:change=move |event| mixed.set(event_target_checked(&event)) />
                    <span>{drill_mixed}</span>
                </label>
                <label class="anzan-field anzan-check">
                    <input type="checkbox" prop:checked=move || countdown.get()
                        disabled=running
                        on:change=move |event| countdown.set(event_target_checked(&event)) />
                    <span>{drill_fixation}</span>
                </label>
                <label class="anzan-field anzan-check">
                    <input type="checkbox" prop:checked=move || sound.get()
                        disabled=running
                        on:change=move |event| sound.set(event_target_checked(&event)) />
                    <span>{drill_sound}</span>
                </label>
                <label class="anzan-field anzan-check">
                    <input type="checkbox" prop:checked=move || auto_advance.get()
                        on:change=move |event| auto_advance.set(event_target_checked(&event)) />
                    <span>{answer_auto_advance}</span>
                </label>
                <button type="button" class="anzan-start" disabled=running on:click=on_start>
                    {drill_start}
                </button>
            </div>

            <div class="anzan-answer" hidden=move || !answered()>
                <AnswerBox
                    strings=strings.clone()
                    answer=answer
                    expected=expected
                    active=RwSignal::new(true)
                    auto_advance=auto_advance
                    on_correct=Action::new(on_correct)
                    on_wrong=Action::new(on_wrong)
                    on_next=Action::new(next_action)
                    on_show_solution=Action::new(show_solution)
                />
            </div>

            <div class="anzan-drill-result" role="status" hidden=move || result.get().is_none()>
                <strong>{result_text}</strong>
                <span class="anzan-level-change">{change_text}</span>
                <button type="button" class="anzan-repeat" on:click=on_repeat>
                    {answer_repeat}
                </button>
            </div>

            <section class="anzan-progress" aria-labelledby="anzan-progress-title">
                <header class="anzan-progress-head">
                    <h3 id="anzan-progress-title">{progress_title}</h3>
                    <span class="anzan-progress-mode">{mode_text}</span>
                </header>
                <dl class="anzan-stats">
                    <div><dt>{progress_level}</dt><dd>{level_text}</dd></div>
                    <div><dt>{progress_accuracy}</dt><dd>{accuracy_text}</dd></div>
                    <div><dt>{progress_streak}</dt><dd>{streak_text}</dd></div>
                    <div><dt>{progress_attempts}</dt><dd>{attempts_text}</dd></div>
                    <div><dt>{progress_best}</dt><dd>{best_text}</dd></div>
                </dl>
                <button type="button" class="anzan-reset" on:click=on_reset_progress>
                    {progress_reset}
                </button>
            </section>

            <section class="anzan-review" aria-labelledby="anzan-review-title">
                <h3 id="anzan-review-title">{review_title}</h3>
                <p>{review_text}</p>
            </section>

            <section class="anzan-case-stats" aria-labelledby="anzan-case-stats-title">
                <h3 id="anzan-case-stats-title">{case_stats_title}</h3>
                <dl class="anzan-case-grid">{stats_rows}</dl>
            </section>

            <section class="anzan-history" aria-labelledby="anzan-history-title">
                <header class="anzan-history-head">
                    <h3 id="anzan-history-title">{session_title}</h3>
                    <button type="button" class="anzan-export" on:click=on_export>
                        {session_export}
                    </button>
                </header>
                <p class="anzan-empty" hidden=move || !sessions.get().is_empty()>
                    {session_empty}
                </p>
                <ul class="anzan-session-list">{history_rows}</ul>
            </section>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_including_new_fields() {
        let settings = DrillSettings {
            digits: 3,
            terms: 7,
            sound: false,
            mixed: true,
            exposure_ms: 250,
            gap_ms: 80,
            countdown: false,
        };
        assert_eq!(DrillSettings::decode(&settings.encode()), settings);
    }

    #[test]
    fn old_settings_keep_working_defaults() {
        let settings = DrillSettings::decode("2,5,1");
        assert_eq!(settings.digits, 2);
        assert_eq!(settings.terms, 5);
        assert!(!settings.mixed);
        assert!(settings.countdown);
        assert_eq!(settings.exposure_ms, 500);
    }

    #[test]
    fn plan_has_countdown_and_one_stimulus_per_term() {
        let spec = LevelSpec {
            digits: 2,
            terms: 5,
            stimulus_frames: STIMULUS_FRAMES,
            gap_frames: GAP_FRAMES,
        };
        let plain = build_plan(spec, false, None);
        let with_countdown = build_plan(spec, true, None);
        assert!(plain.is_some() && with_countdown.is_some());
        if let (Some(plain), Some(with_countdown)) = (plain, with_countdown) {
            assert_eq!(plain.stimulus_count(), 5);
            assert_eq!(with_countdown.stimulus_count(), 5 + COUNTDOWN_STEPS as u32);
        }
    }

    #[test]
    fn speed_controls_change_the_plan() {
        let spec = LevelSpec {
            digits: 2,
            terms: 4,
            stimulus_frames: STIMULUS_FRAMES,
            gap_frames: GAP_FRAMES,
        };
        let slow = build_plan(
            spec,
            false,
            Some(DrillSpeed {
                exposure_ms: 1000,
                gap_ms: 500,
            }),
        );
        let fast = build_plan(
            spec,
            false,
            Some(DrillSpeed {
                exposure_ms: 100,
                gap_ms: 100,
            }),
        );
        assert!(slow.is_some() && fast.is_some());
        if let (Some(slow), Some(fast)) = (slow, fast) {
            assert!(slow.total_frames() > fast.total_frames());
        }
    }
}
