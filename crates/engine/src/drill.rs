// SPDX-License-Identifier: AGPL-3.0-or-later
//! Generación determinista de drills Flash Anzan.
//!
//! Un drill es una secuencia de sumandos positivos que el usuario debe sumar de memoria.
//! La generación depende sólo de [`DrillConfig`] y de una semilla, de modo que una sesión
//! se puede reproducir exactamente y los tests de propiedades son posibles.

use crate::pedagogy::{PedagogicalCase, classify_addition};

/// Dígitos máximos por término.
pub const MAX_DIGITS: u8 = 6;
/// Términos mínimos de un drill.
pub const MIN_TERMS: u8 = 2;
/// Términos máximos de un drill.
pub const MAX_TERMS: u8 = 20;

/// Configuración de un drill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DrillConfig {
    /// Dígitos de cada término (1..=6).
    pub digits: u8,
    /// Cantidad de términos (2..=20).
    pub terms: u8,
}

impl DrillConfig {
    /// Configuración por defecto: 2 dígitos y 5 términos.
    pub const DEFAULT: Self = Self {
        digits: 2,
        terms: 5,
    };

    /// Valida la configuración.
    ///
    /// # Errors
    /// Devuelve [`DrillError`] si los dígitos o los términos están fuera de rango.
    pub const fn validate(self) -> Result<Self, DrillError> {
        if self.digits < 1 || self.digits > MAX_DIGITS {
            return Err(DrillError::InvalidDigits(self.digits));
        }
        if self.terms < MIN_TERMS || self.terms > MAX_TERMS {
            return Err(DrillError::InvalidTerms(self.terms));
        }
        Ok(self)
    }

    /// Mayor valor posible de un término.
    #[must_use]
    pub fn max_term(self) -> u32 {
        10u32.pow(u32::from(self.digits)) - 1
    }
}

/// Error de configuración o generación de un drill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DrillError {
    /// Los dígitos están fuera de 1..=6.
    #[error("dígitos inválidos: {0} (se esperaba 1..=6)")]
    InvalidDigits(u8),
    /// Los términos están fuera de 2..=20.
    #[error("cantidad de términos inválida: {0} (se esperaba 2..=20)")]
    InvalidTerms(u8),
}

/// Drill Flash Anzan ya generado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drill {
    config: DrillConfig,
    seed: u64,
    terms: Vec<u32>,
    answer: u128,
}

impl Drill {
    /// Genera un drill determinista a partir de una semilla.
    ///
    /// # Errors
    /// Devuelve [`DrillError`] si la configuración no es válida.
    pub fn generate(config: DrillConfig, seed: u64) -> Result<Self, DrillError> {
        let config = config.validate()?;
        let max_term = config.max_term();
        let mut state = normalize_seed(seed);
        let mut terms = Vec::with_capacity(usize::from(config.terms));
        let mut answer = 0u128;
        for _ in 0..config.terms {
            let term = (next_u64(&mut state) % u64::from(max_term)) as u32 + 1;
            answer += u128::from(term);
            terms.push(term);
        }
        Ok(Self {
            config,
            seed,
            terms,
            answer,
        })
    }

    /// Configuración usada.
    #[must_use]
    pub const fn config(&self) -> DrillConfig {
        self.config
    }

    /// Semilla original.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Sumandos en orden de presentación.
    #[must_use]
    pub fn terms(&self) -> &[u32] {
        &self.terms
    }

    /// Respuesta correcta.
    #[must_use]
    pub const fn answer(&self) -> u128 {
        self.answer
    }

    /// Comprueba una respuesta.
    #[must_use]
    pub const fn is_correct(&self, answer: u128) -> bool {
        answer == self.answer
    }

    /// Caso pedagógico de cada suma parcial, en orden de presentación.
    #[must_use]
    pub fn cases(&self) -> Vec<PedagogicalCase> {
        let mut running = 0u128;
        self.terms
            .iter()
            .map(|term| {
                let case = classify_addition(running, u128::from(*term));
                running += u128::from(*term);
                case
            })
            .collect()
    }

    /// Caso más difícil presente en el drill.
    #[must_use]
    pub fn hardest_case(&self) -> PedagogicalCase {
        self.cases()
            .into_iter()
            .max()
            .unwrap_or(PedagogicalCase::Direct)
    }
}

/// Normaliza la semilla para que el generador nunca se quede en cero.
pub(crate) const fn normalize_seed(seed: u64) -> u64 {
    if seed == 0 {
        0x9E37_79B9_7F4A_7C15
    } else {
        seed
    }
}

/// Xorshift64*: rápido, determinista y suficiente para generar ejercicios.
pub(crate) const fn next_u64(state: &mut u64) -> u64 {
    let mut value = *state;
    value ^= value >> 12;
    value ^= value << 25;
    value ^= value >> 27;
    *state = value;
    value.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const CASES: u32 = 10_000;

    fn any_config() -> impl Strategy<Value = DrillConfig> {
        (1u8..=MAX_DIGITS, MIN_TERMS..=MAX_TERMS)
            .prop_map(|(digits, terms)| DrillConfig { digits, terms })
    }

    fn test_drill(config: DrillConfig, seed: u64) -> Result<Drill, TestCaseError> {
        Drill::generate(config, seed).map_err(|error| TestCaseError::fail(error.to_string()))
    }

    #[test]
    fn rejects_invalid_configurations() {
        assert_eq!(
            DrillConfig {
                digits: 0,
                terms: 5
            }
            .validate(),
            Err(DrillError::InvalidDigits(0))
        );
        assert_eq!(
            DrillConfig {
                digits: 7,
                terms: 5
            }
            .validate(),
            Err(DrillError::InvalidDigits(7))
        );
        assert_eq!(
            DrillConfig {
                digits: 2,
                terms: 1
            }
            .validate(),
            Err(DrillError::InvalidTerms(1))
        );
        assert_eq!(
            DrillConfig {
                digits: 2,
                terms: 21
            }
            .validate(),
            Err(DrillError::InvalidTerms(21))
        );
    }

    #[test]
    fn same_seed_reproduces_the_same_drill() {
        let config = DrillConfig {
            digits: 3,
            terms: 8,
        };
        let first = Drill::generate(config, 42);
        let second = Drill::generate(config, 42);
        assert_eq!(first, second);
    }

    #[test]
    fn different_seeds_give_different_terms() {
        let config = DrillConfig {
            digits: 3,
            terms: 8,
        };
        let first = Drill::generate(config, 1);
        let second = Drill::generate(config, 2);
        assert!(first.is_ok() && second.is_ok());
        if let (Ok(first), Ok(second)) = (first, second) {
            assert_ne!(first.terms(), second.terms());
        }
    }

    #[test]
    fn cases_cover_every_term_and_hardest_is_the_maximum() {
        let drill = Drill::generate(
            DrillConfig {
                digits: 2,
                terms: 8,
            },
            9,
        );
        assert!(drill.is_ok());
        if let Ok(drill) = drill {
            let cases = drill.cases();
            assert_eq!(cases.len(), usize::from(drill.config().terms));
            let hardest = drill.hardest_case();
            assert!(cases.iter().all(|case| *case <= hardest));
        }
    }

    #[test]
    fn answer_is_the_sum_of_terms() {
        let config = DrillConfig {
            digits: 4,
            terms: 12,
        };
        let drill = Drill::generate(config, 7);
        assert!(drill.is_ok());
        if let Ok(drill) = drill {
            let sum: u128 = drill.terms().iter().copied().map(u128::from).sum();
            assert_eq!(drill.answer(), sum);
            assert!(drill.is_correct(sum));
            assert!(!drill.is_correct(sum + 1));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: CASES, ..ProptestConfig::default() })]

        #[test]
        fn generation_is_deterministic(config in any_config(), seed in any::<u64>()) {
            let first = test_drill(config, seed)?;
            let second = test_drill(config, seed)?;
            prop_assert_eq!(first, second);
        }

        #[test]
        fn terms_respect_the_digit_budget(config in any_config(), seed in any::<u64>()) {
            let drill = test_drill(config, seed)?;
            prop_assert_eq!(drill.terms().len(), usize::from(config.terms));
            let maximum = config.max_term();
            for term in drill.terms() {
                prop_assert!(*term >= 1);
                prop_assert!(*term <= maximum);
            }
        }

        #[test]
        fn answer_matches_the_terms_and_fits(config in any_config(), seed in any::<u64>()) {
            let drill = test_drill(config, seed)?;
            let sum: u128 = drill.terms().iter().copied().map(u128::from).sum();
            prop_assert_eq!(drill.answer(), sum);
            prop_assert!(drill.answer() <= u128::from(config.max_term()) * u128::from(config.terms));
        }

        #[test]
        fn only_the_exact_answer_is_accepted(config in any_config(), seed in any::<u64>(), delta in 1u128..1_000) {
            let drill = test_drill(config, seed)?;
            prop_assert!(drill.is_correct(drill.answer()));
            prop_assert!(!drill.is_correct(drill.answer() + delta));
        }
    }
}
