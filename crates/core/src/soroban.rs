// SPDX-License-Identifier: AGPL-3.0-or-later
//! Soroban completo: una colección de varillas que representa un entero decimal.
//!
//! La varilla 0 es la de menor peso (unidades). El valor máximo depende de la cantidad de
//! varillas; con 38 varillas el máximo es `10^38 - 1`, que cabe holgadamente en `u128`.

use crate::{Digit, Rod, Slot, Touch};

/// Cantidad máxima de varillas soportada.
///
/// `10^38 - 1 < u128::MAX`, así que `value()` nunca desborda con este límite.
pub const MAX_RODS: usize = 38;

/// Cantidad de varillas que usa la UI por defecto.
pub const DEFAULT_RODS: usize = 8;

/// Error de construcción o mutación de un [`Soroban`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SorobanError {
    /// La cantidad de varillas no está en 1..=38.
    #[error("cantidad de varillas inválida: {0} (se esperaba 1..={MAX_RODS})")]
    InvalidRodCount(usize),
    /// La varilla pedida no existe.
    #[error("varilla fuera de rango: {rod} (el soroban tiene {rods})")]
    RodOutOfRange {
        /// Índice pedido.
        rod: usize,
        /// Cantidad de varillas del soroban.
        rods: usize,
    },
    /// El valor no cabe en la cantidad de varillas.
    #[error("el valor {value} no cabe en {rods} varillas")]
    ValueOverflow {
        /// Valor pedido.
        value: u128,
        /// Cantidad de varillas del soroban.
        rods: usize,
    },
}

/// Un soroban de `n` varillas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Soroban {
    rods: Vec<Rod>,
}

impl Default for Soroban {
    fn default() -> Self {
        Self {
            rods: vec![Rod::ZERO; DEFAULT_RODS],
        }
    }
}

impl Soroban {
    /// Crea un soroban en cero con `rods` varillas.
    ///
    /// # Errors
    /// Devuelve [`SorobanError::InvalidRodCount`] si `rods` no está en 1..=38.
    pub fn new(rods: usize) -> Result<Self, SorobanError> {
        if !(1..=MAX_RODS).contains(&rods) {
            return Err(SorobanError::InvalidRodCount(rods));
        }
        Ok(Self {
            rods: vec![Rod::ZERO; rods],
        })
    }

    /// Crea un soroban con `value`.
    ///
    /// # Errors
    /// Devuelve [`SorobanError::ValueOverflow`] si `value` no cabe en `rods` varillas.
    pub fn from_value(value: u128, rods: usize) -> Result<Self, SorobanError> {
        let mut soroban = Self::new(rods)?;
        soroban.set_value(value)?;
        Ok(soroban)
    }

    /// Cantidad de varillas.
    #[must_use]
    pub fn rods(&self) -> usize {
        self.rods.len()
    }

    /// Devuelve la varilla `index`, si existe.
    #[must_use]
    pub fn rod(&self, index: usize) -> Option<Rod> {
        self.rods.get(index).copied()
    }

    /// Valor decimal representado.
    #[must_use]
    pub fn value(&self) -> u128 {
        self.rods
            .iter()
            .rev()
            .fold(0u128, |acc, rod| acc * 10 + u128::from(rod.value()))
    }

    /// Fija el valor decimal.
    ///
    /// # Errors
    /// Devuelve [`SorobanError::ValueOverflow`] si `value` no cabe en las varillas actuales.
    pub fn set_value(&mut self, value: u128) -> Result<(), SorobanError> {
        let limit = 10u128.pow(self.rods.len() as u32);
        if value >= limit {
            return Err(SorobanError::ValueOverflow {
                value,
                rods: self.rods.len(),
            });
        }

        let mut rest = value;
        for rod in &mut self.rods {
            let digit_value = (rest % 10) as u8;
            rest /= 10;
            match Digit::new(digit_value) {
                Ok(digit) => *rod = Rod::from_digit(digit),
                Err(_) => {
                    return Err(SorobanError::ValueOverflow {
                        value,
                        rods: self.rods.len(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Fija el dígito de una varilla.
    ///
    /// # Errors
    /// Devuelve [`SorobanError::RodOutOfRange`] si la varilla no existe.
    pub fn set_digit(&mut self, rod: usize, digit: Digit) -> Result<(), SorobanError> {
        let rods = self.rods.len();
        let target = self
            .rods
            .get_mut(rod)
            .ok_or(SorobanError::RodOutOfRange { rod, rods })?;
        *target = Rod::from_digit(digit);
        Ok(())
    }

    /// Aplica un toque del usuario.
    ///
    /// Devuelve `true` si el estado cambió. El valor siempre queda dentro de 0..=10^n - 1
    /// porque cada varilla sigue siendo un dígito válido.
    ///
    /// # Errors
    /// Devuelve [`SorobanError::RodOutOfRange`] si la varilla del toque no existe.
    pub fn touch(&mut self, touch: Touch) -> Result<bool, SorobanError> {
        let rods = self.rods.len();
        let target = self
            .rods
            .get_mut(touch.rod)
            .ok_or(SorobanError::RodOutOfRange {
                rod: touch.rod,
                rods,
            })?;
        Ok(target.apply(touch.slot))
    }

    /// Pone todas las varillas en cero.
    pub fn reset(&mut self) {
        self.rods.fill(Rod::ZERO);
    }

    /// Dígitos de mayor a menor peso (para depuración y UI).
    #[must_use]
    pub fn digits(&self) -> Vec<Digit> {
        self.rods
            .iter()
            .rev()
            .map(|rod| rod.digit())
            .collect::<Vec<Digit>>()
    }

    /// Todas las posiciones válidas de una varilla.
    #[must_use]
    pub const fn slots() -> [Slot; 5] {
        [
            Slot::Heaven,
            Slot::Earth(1),
            Slot::Earth(2),
            Slot::Earth(3),
            Slot::Earth(4),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const CASES: u32 = 10_000;
    const TEST_RODS: usize = 6;
    const LIMIT: u128 = 1_000_000;

    fn slot_strategy() -> impl Strategy<Value = Slot> {
        prop_oneof![Just(Slot::Heaven), (1u8..=4).prop_map(Slot::Earth)]
    }

    fn touch_strategy() -> impl Strategy<Value = Touch> {
        (0usize..TEST_RODS, slot_strategy()).prop_map(|(rod, slot)| Touch::new(rod, slot))
    }

    fn arbitrary_slot_strategy() -> impl Strategy<Value = Slot> {
        prop_oneof![
            Just(Slot::Heaven),
            (1u8..=4).prop_map(Slot::Earth),
            Just(Slot::Earth(0)),
            Just(Slot::Earth(5)),
        ]
    }

    /// Modelo independiente sobre el dígito 0..=9, sin usar `Rod`.
    fn model_apply(digit: u8, slot: Slot) -> u8 {
        let heaven = digit >= 5;
        let earth = digit % 5;
        match slot {
            Slot::Heaven => {
                if heaven {
                    digit - 5
                } else {
                    digit + 5
                }
            }
            Slot::Earth(index) => {
                let next_earth = if earth >= index { index - 1 } else { index };
                (if heaven { 5 } else { 0 }) + next_earth
            }
        }
    }

    fn test_soroban(rods: usize) -> Result<Soroban, TestCaseError> {
        Soroban::new(rods).map_err(|error| TestCaseError::fail(error.to_string()))
    }

    fn test_set_value(soroban: &mut Soroban, value: u128) -> Result<(), TestCaseError> {
        soroban
            .set_value(value)
            .map_err(|error| TestCaseError::fail(error.to_string()))
    }

    fn test_touch(soroban: &mut Soroban, touch: Touch) -> Result<(), TestCaseError> {
        soroban
            .touch(touch)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        Ok(())
    }

    #[test]
    fn property_case_budget_is_ten_thousand() {
        let cases = std::hint::black_box(CASES);
        println!("propiedades de soroban: casos={cases}");
        assert!(cases >= 10_000);
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: CASES, ..ProptestConfig::default() })]

        #[test]
        fn set_value_round_trips(value in 0u128..LIMIT) {
            let mut soroban = test_soroban(TEST_RODS)?;
            test_set_value(&mut soroban, value)?;
            prop_assert_eq!(soroban.value(), value);
        }

        #[test]
        fn value_overflow_is_rejected(value in LIMIT..=u128::MAX) {
            let mut soroban = test_soroban(TEST_RODS)?;
            prop_assert_eq!(
                soroban.set_value(value),
                Err(SorobanError::ValueOverflow { value, rods: TEST_RODS })
            );
        }

        #[test]
        fn touch_sequences_match_the_u8_model(touches in prop::collection::vec(touch_strategy(), 0..200)) {
            let mut soroban = test_soroban(TEST_RODS)?;
            let mut model = [0u8; TEST_RODS];
            for touch in touches {
                test_touch(&mut soroban, touch)?;
                model[touch.rod] = model_apply(model[touch.rod], touch.slot);
            }
            let expected = model
                .iter()
                .rev()
                .fold(0u128, |acc, digit| acc * 10 + u128::from(*digit));
            prop_assert_eq!(soroban.value(), expected);
        }

        #[test]
        fn every_touch_keeps_all_rods_valid(touch in touch_strategy()) {
            let mut soroban = test_soroban(TEST_RODS)?;
            test_touch(&mut soroban, touch)?;
            for index in 0..TEST_RODS {
                if let Some(rod) = soroban.rod(index) {
                    prop_assert!(rod.earth() <= 4);
                    prop_assert_eq!(rod.value(), rod.digit().value());
                }
            }
        }

        #[test]
        fn fuzz_touch_sequences_never_corrupt_the_soroban(
            (rods, value) in (1usize..=MAX_RODS).prop_flat_map(|rods| {
                let limit = 10u128.pow(rods as u32);
                (Just(rods), 0u128..limit)
            }),
            raw in prop::collection::vec((0usize..MAX_RODS, arbitrary_slot_strategy()), 0..120),
        ) {
            let limit = 10u128.pow(rods as u32);
            let mut soroban = test_soroban(rods)?;
            test_set_value(&mut soroban, value)?;

            for (rod, slot) in raw {
                let touch = Touch::new(rod, slot);
                if rod < rods {
                    test_touch(&mut soroban, touch)?;
                } else {
                    let before = soroban.value();
                    prop_assert!(soroban.touch(touch).is_err());
                    prop_assert_eq!(soroban.value(), before);
                }
            }

            prop_assert!(soroban.value() < limit);
            for index in 0..rods {
                if let Some(rod) = soroban.rod(index) {
                    prop_assert!(rod.earth() <= 4);
                    prop_assert_eq!(rod.value(), rod.digit().value());
                }
            }
        }
    }

    #[test]
    fn rejects_invalid_rod_counts() {
        assert_eq!(Soroban::new(0), Err(SorobanError::InvalidRodCount(0)));
        assert_eq!(
            Soroban::new(MAX_RODS + 1),
            Err(SorobanError::InvalidRodCount(MAX_RODS + 1))
        );
        assert!(Soroban::new(1).is_ok());
        assert!(Soroban::new(MAX_RODS).is_ok());
    }

    #[test]
    fn zero_and_max_values_are_representable() {
        let mut soroban = Soroban::new(4);
        assert!(soroban.is_ok());
        if let Ok(soroban) = soroban.as_mut() {
            assert_eq!(soroban.value(), 0);
            assert_eq!(soroban.set_value(9_999), Ok(()));
            assert_eq!(soroban.value(), 9_999);
            assert_eq!(
                soroban.set_value(10_000),
                Err(SorobanError::ValueOverflow {
                    value: 10_000,
                    rods: 4
                })
            );
        }
    }

    #[test]
    fn touch_out_of_range_is_rejected() {
        let mut soroban = Soroban::new(2);
        assert!(soroban.is_ok());
        if let Ok(soroban) = soroban.as_mut() {
            assert_eq!(
                soroban.touch(Touch::new(2, Slot::Heaven)),
                Err(SorobanError::RodOutOfRange { rod: 2, rods: 2 })
            );
        }
    }
}
