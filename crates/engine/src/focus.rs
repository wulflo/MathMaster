// SPDX-License-Identifier: AGPL-3.0-or-later
//! Generación de drills focalizados en un caso pedagógico concreto.
//!
//! La secuencia se abre con una pareja que garantiza una suma parcial del caso pedido
//! (complemento a 5, complemento a 10 o acarreo en cadena) y se completa con términos
//! aleatorios. Si el presupuesto de dígitos no permite ese caso, por ejemplo un acarreo
//! en cadena con una sola cifra, la función devuelve `None` en vez de generar un ejercicio
//! que no entrena lo prometido.

use crate::drill::{DrillConfig, next_u64, normalize_seed};
use crate::pedagogy::{PedagogicalCase, classify_addition};

/// Drill que contiene al menos una suma parcial del caso pedagógico solicitado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedDrill {
    target: PedagogicalCase,
    terms: Vec<u32>,
    cases: Vec<PedagogicalCase>,
    answer: u128,
}

impl FocusedDrill {
    /// Genera un drill focalizado determinista.
    ///
    /// Devuelve `None` si con los dígitos disponibles no es posible producir el caso
    /// solicitado (por ejemplo, un acarreo en cadena con una sola cifra).
    #[must_use]
    pub fn generate(target: PedagogicalCase, digits: u8, terms: u8, seed: u64) -> Option<Self> {
        let config = DrillConfig { digits, terms }.validate().ok()?;
        let max_term = config.max_term();
        let opening: &[u32] = match target {
            PedagogicalCase::Direct => &[1, 1],
            PedagogicalCase::Complement5 => &[1, 4],
            PedagogicalCase::Complement10 => &[9, 1],
            PedagogicalCase::Carry => &[99, 1],
        };
        if opening.iter().any(|value| *value > max_term) {
            return None;
        }

        let mut state = normalize_seed(seed);
        let mut built_terms = Vec::with_capacity(usize::from(config.terms));
        let mut built_cases = Vec::with_capacity(usize::from(config.terms));
        let mut running = 0u128;
        let mut forced = 0usize;

        while built_terms.len() < usize::from(config.terms) {
            let magnitude = if forced < opening.len() {
                forced += 1;
                opening[forced - 1]
            } else {
                (next_u64(&mut state) % u64::from(max_term)) as u32 + 1
            };
            let case = classify_addition(running, u128::from(magnitude));
            running += u128::from(magnitude);
            built_terms.push(magnitude);
            built_cases.push(case);
        }

        if !built_cases.contains(&target) {
            return None;
        }

        Some(Self {
            target,
            terms: built_terms,
            cases: built_cases,
            answer: running,
        })
    }

    /// Caso pedagógico solicitado.
    #[must_use]
    pub const fn target(&self) -> PedagogicalCase {
        self.target
    }

    /// Términos en orden de presentación.
    #[must_use]
    pub fn terms(&self) -> &[u32] {
        &self.terms
    }

    /// Caso de cada suma parcial; al menos uno coincide con [`Self::target`].
    #[must_use]
    pub fn cases(&self) -> &[PedagogicalCase] {
        &self.cases
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const CASES: u32 = 10_000;

    fn any_case() -> impl Strategy<Value = PedagogicalCase> {
        (0u8..4).prop_map(|index| PedagogicalCase::ALL[usize::from(index)])
    }

    #[test]
    fn every_requested_case_is_generated_when_possible() {
        for target in PedagogicalCase::ALL {
            let drill = FocusedDrill::generate(target, 3, 6, 1234);
            assert!(drill.is_some(), "caso {:?} no generado", target);
            if let Some(drill) = drill {
                assert_eq!(drill.target(), target);
                assert!(drill.cases().contains(&target));
                assert_eq!(drill.terms().len(), 6);
            }
        }
    }

    #[test]
    fn same_seed_reproduces_the_same_drill() {
        let first = FocusedDrill::generate(PedagogicalCase::Complement5, 2, 5, 77);
        let second = FocusedDrill::generate(PedagogicalCase::Complement5, 2, 5, 77);
        assert_eq!(first, second);
    }

    #[test]
    fn chained_carry_is_impossible_with_a_single_digit() {
        assert!(FocusedDrill::generate(PedagogicalCase::Carry, 1, 4, 9).is_none());
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: CASES, ..ProptestConfig::default() })]

        #[test]
        fn generated_drills_match_the_requested_case(target in any_case(), digits in 1u8..=6, terms in 2u8..=20, seed in any::<u64>()) {
            if let Some(drill) = FocusedDrill::generate(target, digits, terms, seed) {
                prop_assert_eq!(drill.target(), target);
                prop_assert_eq!(drill.terms().len(), usize::from(terms));
                prop_assert_eq!(drill.cases().len(), usize::from(terms));
                prop_assert!(drill.cases().contains(&target));
                let sum: u128 = drill.terms().iter().copied().map(u128::from).sum();
                prop_assert_eq!(drill.answer(), sum);
                prop_assert!(drill.is_correct(sum));
                for term in drill.terms() {
                    prop_assert!(*term >= 1 && *term < 10u32.pow(u32::from(digits)));
                }
            }
        }
    }
}
