// SPDX-License-Identifier: AGPL-3.0-or-later
//! Mitori Anzan (見取暗算): columnas de números estilo examen oficial.
//!
//! La generación es determinista y la representación alinea las unidades a la derecha,
//! con un separador cada tres cifras para que la lectura guiada sea posible.

use crate::drill::{next_u64, normalize_seed};

/// Dígitos mínimos por fila.
pub const MIN_DIGITS: u8 = 2;
/// Dígitos máximos por fila.
pub const MAX_DIGITS: u8 = 6;
/// Filas mínimas.
pub const MIN_ROWS: u8 = 3;
/// Filas máximas.
pub const MAX_ROWS: u8 = 15;
/// Tiempo límite máximo en segundos.
pub const MAX_TIME_S: u16 = 600;

/// Configuración de una columna de Mitori Anzan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MitoriConfig {
    /// Dígitos de cada término (2..=6).
    pub digits: u8,
    /// Cantidad de filas (3..=15).
    pub rows: u8,
    /// Tiempo límite en segundos; `0` significa sin límite.
    pub time_limit_s: u16,
}

impl MitoriConfig {
    /// Configuración por defecto: 3 dígitos, 8 filas y 60 s de límite.
    pub const DEFAULT: Self = Self {
        digits: 3,
        rows: 8,
        time_limit_s: 60,
    };

    /// Valida la configuración.
    ///
    /// # Errors
    /// Devuelve [`MitoriError`] si algún parámetro está fuera de rango.
    pub const fn validate(self) -> Result<Self, MitoriError> {
        if self.digits < MIN_DIGITS || self.digits > MAX_DIGITS {
            return Err(MitoriError::Digits(self.digits));
        }
        if self.rows < MIN_ROWS || self.rows > MAX_ROWS {
            return Err(MitoriError::Rows(self.rows));
        }
        if self.time_limit_s > MAX_TIME_S {
            return Err(MitoriError::TimeLimit(self.time_limit_s));
        }
        Ok(self)
    }

    /// Mayor valor posible de una fila.
    #[must_use]
    pub fn max_term(self) -> u32 {
        10u32.pow(u32::from(self.digits)) - 1
    }
}

/// Error de validación de [`MitoriConfig`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MitoriError {
    /// Dígitos fuera de `2..=6`.
    #[error("dígitos inválidos: {0} (se esperaba 2..=6)")]
    Digits(u8),
    /// Filas fuera de `3..=15`.
    #[error("filas inválidas: {0} (se esperaba 3..=15)")]
    Rows(u8),
    /// Tiempo límite por encima de 600 s.
    #[error("tiempo límite inválido: {0} s")]
    TimeLimit(u16),
}

/// Columna ya generada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MitoriDrill {
    config: MitoriConfig,
    seed: u64,
    terms: Vec<u32>,
    answer: u128,
}

impl MitoriDrill {
    /// Genera una columna determinista.
    ///
    /// # Errors
    /// Devuelve [`MitoriError`] si la configuración no es válida.
    pub fn generate(config: MitoriConfig, seed: u64) -> Result<Self, MitoriError> {
        let config = config.validate()?;
        let max_term = config.max_term();
        let mut state = normalize_seed(seed);
        let mut terms = Vec::with_capacity(usize::from(config.rows));
        let mut answer = 0u128;
        for _ in 0..config.rows {
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
    pub const fn config(&self) -> MitoriConfig {
        self.config
    }

    /// Semilla original.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Términos de la columna, en orden.
    #[must_use]
    pub fn terms(&self) -> &[u32] {
        &self.terms
    }

    /// Suma correcta de la columna.
    #[must_use]
    pub const fn answer(&self) -> u128 {
        self.answer
    }

    /// Comprueba una respuesta.
    #[must_use]
    pub const fn is_correct(&self, answer: u128) -> bool {
        answer == self.answer
    }

    /// Términos formateados, alineados a la derecha y con separador de millares.
    #[must_use]
    pub fn columns(&self) -> Vec<String> {
        let width = usize::from(self.config.digits);
        self.terms
            .iter()
            .map(|term| format_column_term(*term, width))
            .collect()
    }
}

/// Formatea un término con ceros a la izquierda y separador cada tres cifras.
///
/// Por ejemplo, con una anchura de 6 el término `1234` se representa como `001,234`.
#[must_use]
pub fn format_column_term(term: u32, width: usize) -> String {
    let padded = format!("{term:0width$}");
    let mut grouped = String::with_capacity(padded.len() + padded.len() / 3);
    for (index, digit) in padded.chars().enumerate() {
        if index > 0 && (padded.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const CASES: u32 = 10_000;

    #[test]
    fn rejects_invalid_configurations() {
        assert!(
            MitoriConfig {
                digits: 1,
                rows: 5,
                time_limit_s: 0
            }
            .validate()
            .is_err()
        );
        assert!(
            MitoriConfig {
                digits: 7,
                rows: 5,
                time_limit_s: 0
            }
            .validate()
            .is_err()
        );
        assert!(
            MitoriConfig {
                digits: 3,
                rows: 2,
                time_limit_s: 0
            }
            .validate()
            .is_err()
        );
        assert!(
            MitoriConfig {
                digits: 3,
                rows: 16,
                time_limit_s: 0
            }
            .validate()
            .is_err()
        );
        assert!(
            MitoriConfig {
                digits: 3,
                rows: 5,
                time_limit_s: 601
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn column_terms_are_right_aligned_with_group_separators() {
        assert_eq!(format_column_term(1234, 6), "001,234");
        assert_eq!(format_column_term(7, 3), "007");
        assert_eq!(format_column_term(123456, 6), "123,456");
        assert_eq!(format_column_term(9, 2), "09");
    }

    #[test]
    fn generation_is_deterministic_and_the_answer_is_the_sum() {
        let first = MitoriDrill::generate(MitoriConfig::DEFAULT, 42);
        let second = MitoriDrill::generate(MitoriConfig::DEFAULT, 42);
        assert_eq!(first, second);
        if let Ok(drill) = first {
            assert_eq!(drill.terms().len(), usize::from(MitoriConfig::DEFAULT.rows));
            let sum: u128 = drill.terms().iter().copied().map(u128::from).sum();
            assert_eq!(drill.answer(), sum);
            assert_eq!(drill.columns().len(), drill.terms().len());
            let width = usize::from(MitoriConfig::DEFAULT.digits);
            for column in drill.columns() {
                assert_eq!(column.len(), width + (width - 1) / 3);
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: CASES, ..ProptestConfig::default() })]

        #[test]
        fn columns_have_the_requested_shape(digits in MIN_DIGITS..=MAX_DIGITS, rows in MIN_ROWS..=MAX_ROWS, limit in 0u16..=MAX_TIME_S, seed in any::<u64>()) {
            let config = MitoriConfig { digits, rows, time_limit_s: limit };
            let drill = MitoriDrill::generate(config, seed);
            prop_assert!(drill.is_ok());
            if let Ok(drill) = drill {
                prop_assert_eq!(drill.terms().len(), usize::from(rows));
                let sum: u128 = drill.terms().iter().copied().map(u128::from).sum();
                prop_assert_eq!(drill.answer(), sum);
                for term in drill.terms() {
                    prop_assert!(*term >= 1 && *term <= config.max_term());
                }
                let expected_len = usize::from(digits) + (usize::from(digits) - 1) / 3;
                for column in drill.columns() {
                    prop_assert_eq!(column.len(), expected_len);
                    prop_assert!(column.chars().all(|character| character.is_ascii_digit() || character == ','));
                }
            }
        }
    }
}
