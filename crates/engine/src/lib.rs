// SPDX-License-Identifier: AGPL-3.0-or-later
//! Motor de MathMaster: planes temporales en frames, drills y audio precalculado.
//!
//! El motor no dibuja ni abre dispositivos. Produce planes, secuencias y buffers; la UI
//! sólo los reproduce.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod audio;
pub mod drill;
pub mod frames;
pub mod multiplication;
pub mod pedagogy;
pub mod platform;
pub mod progress;
pub mod review;
pub mod schedule;
pub mod session;

pub use audio::{
    AudioError, ClickBuffer, DEFAULT_SAMPLE_RATE, ToneSpec, click_offsets, frame_to_sample,
};
pub use drill::{Drill, DrillConfig, DrillError};
pub use frames::{FrameError, frames_for_duration};
pub use multiplication::{
    MAX_FACTOR_DIGITS, MAX_MUL_LEVEL, MAX_RODS, MIN_MUL_LEVEL, MUL_LEVELS, MulProgress,
    Multiplication, MultiplicationDrill, MultiplicationError, MultiplicationLevel, PartialProduct,
    RodState, multiplication_level,
};
pub use pedagogy::{PedagogicalCase, classify_addition};
pub use progress::{LevelChange, LevelSpec, MAX_LEVEL, MIN_LEVEL, ProgressState, level_spec};
pub use review::{DAY_MS, MAX_INTERVAL_MS, ReviewOutcome, ReviewState};
pub use schedule::{
    FramePlan, FramePlanBuilder, FrameTelemetry, PlanError, Segment, SegmentKind, TelemetryError,
    TelemetrySummary,
};
pub use session::{
    CaseStat, MAX_SESSIONS, SessionRecord, case_stats, decode_log, encode_log, to_csv,
};
