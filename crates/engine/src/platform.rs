// SPDX-License-Identifier: AGPL-3.0-or-later
//! Puertos de plataforma.
//!
//! Este es el único lugar del proyecto donde se permite `unsafe`, y sólo con un bloque
//! `// SAFETY:` que justifique la invariante. De momento no hay ninguna llamada insegura:
//! los backends concretos se añadirán con las tareas de audio y almacenamiento.

use core::time::Duration;

/// Reloj monótono abstraído por plataforma.
pub trait Clock {
    /// Instante actual desde un origen arbitrario pero monótono.
    fn now(&self) -> Duration;
}

/// Nombre de la cápsula de ejecución actual.
#[must_use]
pub const fn name() -> &'static str {
    if cfg!(target_arch = "wasm32") {
        "web"
    } else {
        "desktop"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_a_known_platform() {
        assert!(matches!(name(), "web" | "desktop"));
    }
}
