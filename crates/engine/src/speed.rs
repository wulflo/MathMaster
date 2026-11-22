// SPDX-License-Identifier: AGPL-3.0-or-later
//! Control granular de velocidad en modo libre.
//!
//! La exposición y el hueco se configuran en milisegundos y se convierten a frames con
//! [`crate::frames_for_duration`], la misma función que usa el planificador. Ningún valor
//! se redondea a ojo y las cadencias son siempre finitas.

use crate::frames_for_duration;

/// Exposición mínima por término, en milisegundos.
pub const MIN_EXPOSURE_MS: u16 = 100;
/// Exposición máxima por término, en milisegundos.
pub const MAX_EXPOSURE_MS: u16 = 1500;
/// Hueco máximo entre términos, en milisegundos.
pub const MAX_GAP_MS: u16 = 1500;

/// Velocidad configurable de un drill libre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DrillSpeed {
    /// Milisegundos que cada término permanece visible.
    pub exposure_ms: u16,
    /// Milisegundos de hueco entre términos.
    pub gap_ms: u16,
}

impl DrillSpeed {
    /// Configuración por defecto: 500 ms de exposición y 100 ms de hueco.
    pub const DEFAULT: Self = Self {
        exposure_ms: 500,
        gap_ms: 100,
    };

    /// Valida la configuración.
    ///
    /// # Errors
    /// Devuelve [`SpeedError`] si la exposición o el hueco están fuera de rango.
    pub const fn validate(self) -> Result<Self, SpeedError> {
        if self.exposure_ms < MIN_EXPOSURE_MS || self.exposure_ms > MAX_EXPOSURE_MS {
            return Err(SpeedError::Exposure(self.exposure_ms));
        }
        if self.gap_ms > MAX_GAP_MS {
            return Err(SpeedError::Gap(self.gap_ms));
        }
        Ok(self)
    }

    /// Frames que ocupa la exposición a la cadencia indicada.
    #[must_use]
    pub fn exposure_frames(self, frame_ms: f64) -> u32 {
        frames_for_duration(f64::from(self.exposure_ms), frame_ms).unwrap_or(1)
    }

    /// Frames que ocupa el hueco a la cadencia indicada.
    #[must_use]
    pub fn gap_frames(self, frame_ms: f64) -> u32 {
        if self.gap_ms == 0 {
            return 0;
        }
        frames_for_duration(f64::from(self.gap_ms), frame_ms).unwrap_or(1)
    }

    /// Cadencia en términos por segundo.
    #[must_use]
    pub fn terms_per_second(self) -> f64 {
        1000.0 / f64::from(u32::from(self.exposure_ms) + u32::from(self.gap_ms))
    }
}

/// Error de validación de [`DrillSpeed`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SpeedError {
    /// La exposición está fuera de `100..=1500` ms.
    #[error("exposición inválida: {0} ms (se esperaba 100..=1500)")]
    Exposure(u16),
    /// El hueco supera 1500 ms.
    #[error("hueco inválido: {0} ms (se esperaba 0..=1500)")]
    Gap(u16),
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const CASES: u32 = 10_000;
    const FRAME_MS: f64 = 1000.0 / 60.0;

    fn valid_speed(exposure: u16, gap: u16) -> Result<DrillSpeed, TestCaseError> {
        DrillSpeed {
            exposure_ms: exposure,
            gap_ms: gap,
        }
        .validate()
        .map_err(|error| TestCaseError::fail(error.to_string()))
    }

    #[test]
    fn default_is_valid() {
        assert_eq!(DrillSpeed::DEFAULT.validate(), Ok(DrillSpeed::DEFAULT));
    }

    #[test]
    fn rejects_out_of_range_values() {
        assert_eq!(
            DrillSpeed {
                exposure_ms: 99,
                gap_ms: 0
            }
            .validate(),
            Err(SpeedError::Exposure(99))
        );
        assert_eq!(
            DrillSpeed {
                exposure_ms: 1501,
                gap_ms: 0
            }
            .validate(),
            Err(SpeedError::Exposure(1501))
        );
        assert_eq!(
            DrillSpeed {
                exposure_ms: 100,
                gap_ms: 1501
            }
            .validate(),
            Err(SpeedError::Gap(1501))
        );
    }

    #[test]
    fn frames_and_cadence_match_the_reference_values() {
        let speed = DrillSpeed {
            exposure_ms: 100,
            gap_ms: 100,
        };
        assert_eq!(speed.exposure_frames(FRAME_MS), 6);
        assert_eq!(speed.gap_frames(FRAME_MS), 6);
        assert!((speed.terms_per_second() - 5.0).abs() < f64::EPSILON);
        let zero_gap = DrillSpeed {
            exposure_ms: 500,
            gap_ms: 0,
        };
        assert_eq!(zero_gap.gap_frames(FRAME_MS), 0);
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: CASES, ..ProptestConfig::default() })]

        #[test]
        fn valid_speeds_have_bounded_frames_and_cadence(exposure in MIN_EXPOSURE_MS..=MAX_EXPOSURE_MS, gap in 0u16..=MAX_GAP_MS) {
            let speed = valid_speed(exposure, gap)?;
            prop_assert!(speed.exposure_frames(FRAME_MS) >= 1);
            prop_assert!(speed.exposure_frames(FRAME_MS) <= 90);
            let cadence = speed.terms_per_second();
            prop_assert!(cadence.is_finite());
            prop_assert!(cadence > 0.33 && cadence <= 10.0);
            let total = speed.exposure_frames(FRAME_MS) + speed.gap_frames(FRAME_MS);
            prop_assert!(total >= 1);
        }
    }
}
