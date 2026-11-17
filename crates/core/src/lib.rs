// SPDX-License-Identifier: AGPL-3.0-or-later
//! Núcleo puro de MathMaster.
//!
//! Este crate no conoce la UI, ni el sistema operativo, ni el reloj. Contiene las
//! estructuras matemáticas del soroban, el modelo de fading y la geometría normalizada.
//!
//! Regla de oro: nada aquí puede depender de `engine`, `content` o `ui`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod digit;
pub mod fade;
pub mod geometry;
pub mod plan;
pub mod rod;
pub mod soroban;

pub use digit::{Digit, DigitError};
pub use fade::{FadeLayer, FadeProfile, FadeStage, lerp, lerp_profile};
pub use geometry::{GeometryError, Rect, SorobanGeometry};
pub use plan::{Plan, PlanError, Step, StepKind};
pub use rod::{Rod, RodError, Slot, Touch};
pub use soroban::{DEFAULT_RODS, MAX_RODS, Soroban, SorobanError};
