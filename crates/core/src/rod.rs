// SPDX-License-Identifier: AGPL-3.0-or-later
//! Una varilla del soroban: una ficha celestial y cuatro terrestres.
//!
//! El estado de una varilla es exactamente un dígito decimal 0..=9. La invariante se
//! impone en la construcción y en cada mutación: nunca existe una varilla con más de
//! cuatro fichas terrestres activas.

use crate::Digit;

/// Error al construir o mutar una [`Rod`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RodError {
    /// La cantidad de fichas terrestres activas no está en 0..=4.
    #[error("cantidad de fichas terrestres fuera de rango: {0} (se esperaba 0..=4)")]
    EarthOutOfRange(u8),
    /// El índice de una ficha terrestre no está en 1..=4.
    #[error("posición terrestre fuera de rango: {0} (se esperaba 1..=4)")]
    EarthSlotOutOfRange(u8),
}

/// Posición de una ficha dentro de una varilla.
///
/// `Earth(1)` es la ficha terrestre más cercana al travesaño; `Earth(4)` la más lejana.
/// `Heaven` es la única ficha celestial (vale 5 cuando está activa).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Slot {
    /// Ficha celestial (5).
    Heaven,
    /// Ficha terrestre 1..=4 (cada una vale 1).
    Earth(u8),
}

impl Slot {
    /// Menor índice terrestre válido.
    pub const EARTH_MIN: u8 = 1;
    /// Mayor índice terrestre válido.
    pub const EARTH_MAX: u8 = 4;

    /// Construye la posición de una ficha terrestre validando el índice.
    ///
    /// # Errors
    /// Devuelve [`RodError::EarthSlotOutOfRange`] si `index` no está en 1..=4.
    pub const fn earth(index: u8) -> Result<Self, RodError> {
        if index >= Self::EARTH_MIN && index <= Self::EARTH_MAX {
            Ok(Self::Earth(index))
        } else {
            Err(RodError::EarthSlotOutOfRange(index))
        }
    }

    /// Índice lineal de la ficha: 0 para `Heaven`, 1..=4 para `Earth`.
    #[must_use]
    pub const fn index(self) -> u8 {
        match self {
            Self::Heaven => 0,
            Self::Earth(index) => index,
        }
    }
}

/// Toque del usuario sobre una ficha de una varilla concreta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Touch {
    /// Índice de varilla; 0 es la de menor peso (unidades).
    pub rod: usize,
    /// Ficha tocada dentro de la varilla.
    pub slot: Slot,
}

impl Touch {
    /// Construye un toque.
    #[must_use]
    pub const fn new(rod: usize, slot: Slot) -> Self {
        Self { rod, slot }
    }
}

/// Estado de una varilla del soroban (un dígito decimal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rod {
    heaven: bool,
    earth: u8,
}

impl Rod {
    /// Varilla en cero.
    pub const ZERO: Self = Self {
        heaven: false,
        earth: 0,
    };

    /// Construye una varilla a partir de sus fichas.
    ///
    /// # Errors
    /// Devuelve [`RodError::EarthOutOfRange`] si `earth > 4`.
    pub const fn from_parts(heaven: bool, earth: u8) -> Result<Self, RodError> {
        if earth <= 4 {
            Ok(Self { heaven, earth })
        } else {
            Err(RodError::EarthOutOfRange(earth))
        }
    }

    /// Construye la varilla que representa `digit`.
    #[must_use]
    pub const fn from_digit(digit: Digit) -> Self {
        let value = digit.value();
        Self {
            heaven: value >= 5,
            earth: value % 5,
        }
    }

    /// Dígito decimal que representa la varilla.
    #[must_use]
    pub const fn digit(self) -> Digit {
        // La invariante `earth <= 4` garantiza que el valor está en 0..=9.
        Digit::from_valid(self.value())
    }

    /// Valor numérico de la varilla (0..=9).
    #[must_use]
    pub const fn value(self) -> u8 {
        (if self.heaven { 5 } else { 0 }) + self.earth
    }

    /// Indica si la ficha celestial está activa.
    #[must_use]
    pub const fn heaven(self) -> bool {
        self.heaven
    }

    /// Cantidad de fichas terrestres activas (0..=4).
    #[must_use]
    pub const fn earth(self) -> u8 {
        self.earth
    }

    /// Activa o desactiva la ficha celestial.
    pub const fn set_heaven(&mut self, active: bool) {
        self.heaven = active;
    }

    /// Fija la cantidad de fichas terrestres activas.
    ///
    /// # Errors
    /// Devuelve [`RodError::EarthOutOfRange`] si `earth > 4`.
    pub const fn set_earth(&mut self, earth: u8) -> Result<(), RodError> {
        if earth <= 4 {
            self.earth = earth;
            Ok(())
        } else {
            Err(RodError::EarthOutOfRange(earth))
        }
    }

    /// Aplica un toque sobre una ficha y devuelve si el estado cambió.
    ///
    /// La ficha celestial se alterna. Tocar una ficha terrestre activa la desactiva a ella
    /// y a las más lejanas (deja `index - 1` activas); tocar una inactiva activa esa ficha
    /// y las más cercanas al travesaño (deja `index` activas). El resultado siempre tiene
    /// entre 0 y 4 fichas terrestres activas.
    pub const fn apply(&mut self, slot: Slot) -> bool {
        match slot {
            Slot::Heaven => {
                self.heaven = !self.heaven;
                true
            }
            Slot::Earth(index) => {
                if index < Slot::EARTH_MIN || index > Slot::EARTH_MAX {
                    return false;
                }
                let next = if self.earth >= index {
                    index - 1
                } else {
                    index
                };
                if next == self.earth {
                    false
                } else {
                    self.earth = next;
                    true
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn all_ten_digits_round_trip() {
        for value in 0..=9 {
            let digit = Digit::new(value);
            assert!(digit.is_ok(), "el dígito {value} debe ser válido");
            if let Ok(digit) = digit {
                let rod = Rod::from_digit(digit);
                assert_eq!(rod.digit(), digit, "la biyección falla en {value}");
                assert_eq!(rod.value(), value);
            }
        }
    }

    #[test]
    fn ten_digits_map_to_ten_distinct_states() {
        let states: BTreeSet<(bool, u8)> = (0..=9)
            .filter_map(|value| Digit::new(value).ok())
            .map(|digit| {
                let rod = Rod::from_digit(digit);
                (rod.heaven(), rod.earth())
            })
            .collect();
        assert_eq!(states.len(), 10, "debe haber exactamente 10 estados");
    }

    #[test]
    fn earth_above_four_is_rejected() {
        assert_eq!(Rod::from_parts(true, 5), Err(RodError::EarthOutOfRange(5)));
        let mut rod = Rod::ZERO;
        assert_eq!(rod.set_earth(5), Err(RodError::EarthOutOfRange(5)));
        assert_eq!(rod.earth(), 0, "un error no debe mutar la varilla");
    }

    #[test]
    fn earth_slot_indices_are_validated() {
        assert!(Slot::earth(1).is_ok());
        assert!(Slot::earth(4).is_ok());
        assert_eq!(Slot::earth(0), Err(RodError::EarthSlotOutOfRange(0)));
        assert_eq!(Slot::earth(5), Err(RodError::EarthSlotOutOfRange(5)));
    }

    #[test]
    fn heaven_toggles_between_zero_and_five() {
        let mut rod = Rod::ZERO;
        assert!(rod.apply(Slot::Heaven));
        assert_eq!(rod.value(), 5);
        assert!(rod.apply(Slot::Heaven));
        assert_eq!(rod.value(), 0);
    }

    #[test]
    fn tapping_earth_beads_sets_the_active_count() {
        let mut rod = Rod::ZERO;
        assert!(rod.apply(Slot::Earth(3)));
        assert_eq!(rod.value(), 3);
        assert!(rod.apply(Slot::Earth(2)));
        assert_eq!(rod.value(), 1);
        assert!(rod.apply(Slot::Earth(4)));
        assert_eq!(rod.value(), 4);
        assert!(rod.apply(Slot::Earth(4)));
        assert_eq!(rod.value(), 3);
    }

    #[test]
    fn invalid_earth_slots_are_ignored() {
        let mut rod = Rod::from_parts(true, 2).unwrap_or(Rod::ZERO);
        let before = rod;
        assert!(!rod.apply(Slot::Earth(0)));
        assert!(!rod.apply(Slot::Earth(5)));
        assert_eq!(rod, before);
    }

    #[test]
    fn every_touch_keeps_the_rod_valid() {
        for value in 0..=9 {
            if let Ok(digit) = Digit::new(value) {
                let rod = Rod::from_digit(digit);
                for slot in [
                    Slot::Heaven,
                    Slot::Earth(1),
                    Slot::Earth(2),
                    Slot::Earth(3),
                    Slot::Earth(4),
                ] {
                    let mut next = rod;
                    next.apply(slot);
                    assert!(next.earth() <= 4, "estado inválido tras {slot:?}");
                    assert_eq!(next.value(), next.digit().value());
                }
            }
        }
    }
}
