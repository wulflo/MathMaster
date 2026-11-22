// SPDX-License-Identifier: AGPL-3.0-or-later
//! Drills deterministas de sumas y restas intercaladas.
//!
//! Invariante central: el acumulado parcial nunca es negativo. Un término sólo se marca
//! como sustractivo si el acumulado actual puede absorberlo; así el plan del ábaco siempre
//! es ejecutable y la respuesta final es un entero no negativo.

use crate::drill::{DrillError, next_u64, normalize_seed};

/// Signo de un término de un drill mixto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TermSign {
    /// El término se suma.
    Add,
    /// El término se resta.
    Subtract,
}

/// Término con signo, listo para mostrar con el símbolo correspondiente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignedTerm {
    /// Signo del término.
    pub sign: TermSign,
    /// Valor absoluto del término.
    pub value: u32,
}

/// Configuración de un drill mixto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MixedConfig {
    /// Dígitos de cada término (1..=6).
    pub digits: u8,
    /// Cantidad de términos (2..=20).
    pub terms: u8,
}

impl MixedConfig {
    /// Configuración por defecto: 2 dígitos y 6 términos.
    pub const DEFAULT: Self = Self {
        digits: 2,
        terms: 6,
    };

    /// Valida la configuración con los mismos límites que los drills Flash.
    ///
    /// # Errors
    /// Devuelve [`DrillError`] si los dígitos o los términos están fuera de rango.
    pub const fn validate(self) -> Result<Self, DrillError> {
        if self.digits < 1 || self.digits > crate::drill::MAX_DIGITS {
            return Err(DrillError::InvalidDigits(self.digits));
        }
        if self.terms < crate::drill::MIN_TERMS || self.terms > 20 {
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

/// Error al generar un drill mixto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MixedError {
    /// La configuración no es válida.
    #[error("configuración de drill mixto inválida: {0}")]
    Config(#[from] DrillError),
    /// No se pudo construir una secuencia con el invariante pedido.
    #[error("no se pudo generar una secuencia mixta con acumulado no negativo")]
    Generation,
}

/// Drill mixto ya generado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixedDrill {
    config: MixedConfig,
    seed: u64,
    terms: Vec<SignedTerm>,
    partials: Vec<u128>,
    answer: u128,
}

impl MixedDrill {
    /// Genera un drill mixto determinista.
    ///
    /// El primer término siempre es positivo. Cada término posterior se resta sólo si el
    /// acumulado lo permite, de modo que todos los parciales son `>= 0`.
    ///
    /// # Errors
    /// Devuelve [`MixedError`] si la configuración no es válida o si la generación no
    /// consigue satisfacer el invariante.
    pub fn generate(config: MixedConfig, seed: u64) -> Result<Self, MixedError> {
        let config = config.validate()?;
        let max_term = config.max_term();
        let mut state = normalize_seed(seed);
        let mut terms = Vec::with_capacity(usize::from(config.terms));
        let mut partials = Vec::with_capacity(usize::from(config.terms));
        let mut running = 0u128;

        for index in 0..config.terms {
            let magnitude = (next_u64(&mut state) % u64::from(max_term)) as u32 + 1;
            let subtract = index > 0
                && next_u64(&mut state).is_multiple_of(2)
                && running >= u128::from(magnitude);
            let sign = if subtract {
                TermSign::Subtract
            } else {
                TermSign::Add
            };
            running = match sign {
                TermSign::Add => running + u128::from(magnitude),
                TermSign::Subtract => running - u128::from(magnitude),
            };
            terms.push(SignedTerm {
                sign,
                value: magnitude,
            });
            partials.push(running);
        }

        if terms.len() != usize::from(config.terms) || partials.len() != usize::from(config.terms) {
            return Err(MixedError::Generation);
        }

        Ok(Self {
            config,
            seed,
            terms,
            partials,
            answer: running,
        })
    }

    /// Configuración usada.
    #[must_use]
    pub const fn config(&self) -> MixedConfig {
        self.config
    }

    /// Semilla original.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Términos con signo, en orden de presentación.
    #[must_use]
    pub fn terms(&self) -> &[SignedTerm] {
        &self.terms
    }

    /// Acumulado después de cada término; ninguno es negativo por construcción.
    #[must_use]
    pub fn partials(&self) -> &[u128] {
        &self.partials
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

    /// Reconstruye el acumulado a partir de los términos con signo.
    #[must_use]
    pub fn recomputed_answer(&self) -> u128 {
        self.terms
            .iter()
            .fold(0u128, |total, term| match term.sign {
                TermSign::Add => total + u128::from(term.value),
                TermSign::Subtract => total.saturating_sub(u128::from(term.value)),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drill::{MAX_DIGITS, MIN_TERMS};
    use proptest::prelude::*;

    const CASES: u32 = 10_000;

    fn any_config() -> impl Strategy<Value = MixedConfig> {
        (1u8..=MAX_DIGITS, MIN_TERMS..=20).prop_map(|(digits, terms)| MixedConfig { digits, terms })
    }

    fn test_drill(config: MixedConfig, seed: u64) -> Result<MixedDrill, TestCaseError> {
        MixedDrill::generate(config, seed).map_err(|error| TestCaseError::fail(error.to_string()))
    }

    #[test]
    fn rejects_invalid_configurations() {
        assert!(
            MixedConfig {
                digits: 0,
                terms: 5
            }
            .validate()
            .is_err()
        );
        assert!(
            MixedConfig {
                digits: 7,
                terms: 5
            }
            .validate()
            .is_err()
        );
        assert!(
            MixedConfig {
                digits: 2,
                terms: 1
            }
            .validate()
            .is_err()
        );
        assert!(
            MixedConfig {
                digits: 2,
                terms: 21
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn first_term_is_positive_and_partials_never_go_negative() {
        let drill = MixedDrill::generate(MixedConfig::DEFAULT, 17);
        assert!(drill.is_ok());
        if let Ok(drill) = drill {
            assert_eq!(drill.terms()[0].sign, TermSign::Add);
            assert!(
                drill
                    .partials()
                    .iter()
                    .all(|value| *value <= u128::from(u32::MAX))
            );
        }
    }

    #[test]
    fn subtraction_only_happens_when_the_running_total_allows_it() {
        let drill = MixedDrill::generate(MixedConfig::DEFAULT, 99);
        assert!(drill.is_ok());
        if let Ok(drill) = drill {
            let mut running = 0u128;
            for (index, term) in drill.terms().iter().enumerate() {
                match term.sign {
                    TermSign::Add => running += u128::from(term.value),
                    TermSign::Subtract => {
                        assert!(running >= u128::from(term.value));
                        running -= u128::from(term.value);
                    }
                }
                assert_eq!(drill.partials()[index], running);
            }
            assert_eq!(drill.answer(), running);
            assert_eq!(drill.recomputed_answer(), running);
        }
    }

    #[test]
    fn same_seed_reproduces_the_same_sequence() {
        let first = MixedDrill::generate(MixedConfig::DEFAULT, 3);
        let second = MixedDrill::generate(MixedConfig::DEFAULT, 3);
        assert_eq!(first, second);
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: CASES, ..ProptestConfig::default() })]

        #[test]
        fn generation_is_deterministic_and_bounded(config in any_config(), seed in any::<u64>()) {
            let first = test_drill(config, seed)?;
            let second = test_drill(config, seed)?;
            prop_assert_eq!(&first, &second);
            prop_assert_eq!(first.terms().len(), usize::from(config.terms));
            prop_assert_eq!(first.partials().len(), usize::from(config.terms));
            prop_assert_eq!(first.answer(), first.recomputed_answer());
            prop_assert_eq!(first.partials().last().copied().unwrap_or(0), first.answer());
            prop_assert_eq!(first.terms()[0].sign, TermSign::Add);
            for term in first.terms() {
                prop_assert!(term.value >= 1 && term.value <= config.max_term());
            }
        }

        #[test]
        fn every_partial_respects_the_absolute_limit(config in any_config(), seed in any::<u64>()) {
            let drill = test_drill(config, seed)?;
            let limit = u128::from(config.max_term()) * u128::from(config.terms);
            for partial in drill.partials() {
                prop_assert!(*partial <= limit);
            }
            for (index, term) in drill.terms().iter().enumerate() {
                if term.sign == TermSign::Subtract {
                    prop_assert!(index > 0);
                }
            }
        }
    }
}
