// SPDX-License-Identifier: AGPL-3.0-or-later
//! Punto de entrada de MathMaster.
//!
//! La web (wasm) arranca en [`web::start`]. El host Tauri se añadirá en una tarea
//! posterior; por ahora este crate compila en host y en wasm para validar el workspace.

#![forbid(unsafe_code)]
#![allow(missing_docs)] // El entrypoint wasm lo documenta el módulo; no hay API pública amplia.

pub mod web;

/// Cápsula de ejecución actual (`web` o `desktop`).
#[must_use]
pub const fn platform() -> &'static str {
    anzan_engine::platform::name()
}
