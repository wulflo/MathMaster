// SPDX-License-Identifier: AGPL-3.0-or-later
//! Progresión por niveles y estadísticas de sesión.
//!
//! El modelo es deliberadamente simple y auditable: dos aciertos consecutivos suben de
//! nivel y dos fallos consecutivos bajan uno. No es un algoritmo de repaso espaciado
//! completo; es el equivalente propio de esta fase, sin dependencias opacas.

/// Nivel mínimo.
pub const MIN_LEVEL: u8 = 1;
/// Nivel máximo.
pub const MAX_LEVEL: u8 = 6;
/// Aciertos consecutivos necesarios para subir de nivel.
pub const PROMOTE_STREAK: u32 = 2;
/// Fallos consecutivos que hacen bajar de nivel.
pub const DEMOTE_MISSES: u32 = 2;

/// Parámetros pedagógicos de un nivel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LevelSpec {
    /// Dígitos por término.
    pub digits: u8,
    /// Cantidad de términos.
    pub terms: u8,
    /// Frames que permanece visible cada término.
    pub stimulus_frames: u32,
    /// Frames de hueco entre términos.
    pub gap_frames: u32,
}

impl LevelSpec {
    /// Frames totales de un término (estímulo + hueco).
    #[must_use]
    pub const fn frames_per_term(self) -> u32 {
        self.stimulus_frames + self.gap_frames
    }
}

/// Tabla de niveles, de más fácil a más difícil.
pub const LEVELS: [LevelSpec; 6] = [
    LevelSpec {
        digits: 1,
        terms: 3,
        stimulus_frames: 36,
        gap_frames: 9,
    },
    LevelSpec {
        digits: 1,
        terms: 5,
        stimulus_frames: 30,
        gap_frames: 6,
    },
    LevelSpec {
        digits: 2,
        terms: 4,
        stimulus_frames: 30,
        gap_frames: 6,
    },
    LevelSpec {
        digits: 2,
        terms: 6,
        stimulus_frames: 24,
        gap_frames: 6,
    },
    LevelSpec {
        digits: 3,
        terms: 5,
        stimulus_frames: 24,
        gap_frames: 6,
    },
    LevelSpec {
        digits: 3,
        terms: 7,
        stimulus_frames: 18,
        gap_frames: 6,
    },
];

/// Especificación de un nivel, con saturación en 1..=6.
#[must_use]
pub const fn level_spec(level: u8) -> LevelSpec {
    let index = if level < MIN_LEVEL {
        0
    } else if level > MAX_LEVEL {
        MAX_LEVEL as usize - 1
    } else {
        level as usize - 1
    };
    LEVELS[index]
}

/// Cambio de nivel producido por un resultado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LevelChange {
    /// El nivel no cambió.
    None,
    /// Subió al nivel indicado.
    Promoted(u8),
    /// Bajó al nivel indicado.
    Demoted(u8),
}

/// Estado de progreso y estadísticas acumuladas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProgressState {
    /// Nivel actual (1..=6).
    pub level: u8,
    /// Aciertos consecutivos.
    pub streak: u32,
    /// Fallos consecutivos.
    pub misses: u32,
    /// Intentos totales.
    pub attempts: u64,
    /// Aciertos totales.
    pub correct: u64,
    /// Mejor racha histórica.
    pub best_streak: u32,
}

impl Default for ProgressState {
    fn default() -> Self {
        Self {
            level: MIN_LEVEL,
            streak: 0,
            misses: 0,
            attempts: 0,
            correct: 0,
            best_streak: 0,
        }
    }
}

impl ProgressState {
    /// Registra un resultado y devuelve el cambio de nivel.
    pub fn record(&mut self, correct: bool) -> LevelChange {
        self.attempts = self.attempts.saturating_add(1);
        if correct {
            self.correct = self.correct.saturating_add(1);
            self.streak = self.streak.saturating_add(1);
            self.misses = 0;
            self.best_streak = self.best_streak.max(self.streak);
            if self.streak >= PROMOTE_STREAK && self.level < MAX_LEVEL {
                self.level += 1;
                self.streak = 0;
                LevelChange::Promoted(self.level)
            } else {
                LevelChange::None
            }
        } else {
            self.streak = 0;
            self.misses = self.misses.saturating_add(1);
            if self.misses >= DEMOTE_MISSES && self.level > MIN_LEVEL {
                self.level -= 1;
                self.misses = 0;
                LevelChange::Demoted(self.level)
            } else {
                LevelChange::None
            }
        }
    }

    /// Precisión acumulada en `0..=1`.
    #[must_use]
    pub fn accuracy(&self) -> f32 {
        if self.attempts == 0 {
            0.0
        } else {
            self.correct as f32 / self.attempts as f32
        }
    }

    /// Parámetros del nivel actual.
    #[must_use]
    pub const fn spec(&self) -> LevelSpec {
        level_spec(self.level)
    }

    /// Serializa el estado en un formato estable de una línea.
    #[must_use]
    pub fn encode(&self) -> String {
        format!(
            "v1,{},{},{},{},{},{}",
            self.level, self.streak, self.misses, self.attempts, self.correct, self.best_streak
        )
    }

    /// Reconstruye el estado desde [`Self::encode`].
    ///
    /// Acepta únicamente valores coherentes: nivel 1..=6, aciertos <= intentos y mejor
    /// racha <= intentos. Cualquier otro valor devuelve `None`.
    #[must_use]
    pub fn decode(raw: &str) -> Option<Self> {
        let mut parts = raw.split(',');
        if parts.next() != Some("v1") {
            return None;
        }
        let level = parts.next()?.parse::<u8>().ok()?;
        let streak = parts.next()?.parse::<u32>().ok()?;
        let misses = parts.next()?.parse::<u32>().ok()?;
        let attempts = parts.next()?.parse::<u64>().ok()?;
        let correct = parts.next()?.parse::<u64>().ok()?;
        let best_streak = parts.next()?.parse::<u32>().ok()?;
        if parts.next().is_some()
            || !(MIN_LEVEL..=MAX_LEVEL).contains(&level)
            || correct > attempts
            || u64::from(streak) > attempts
            || u64::from(best_streak) > attempts
        {
            return None;
        }
        Some(Self {
            level,
            streak,
            misses,
            attempts,
            correct,
            best_streak,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const CASES: u32 = 10_000;

    fn any_progress() -> impl Strategy<Value = ProgressState> {
        (
            MIN_LEVEL..=MAX_LEVEL,
            0u32..50,
            0u32..50,
            0u64..500,
            0u64..500,
            0u32..50,
        )
            .prop_map(|(level, streak, misses, attempts, correct, best_streak)| {
                let limit = u32::try_from(attempts).unwrap_or(u32::MAX);
                ProgressState {
                    level,
                    streak: streak.min(limit),
                    misses: misses.min(limit),
                    attempts,
                    correct: correct.min(attempts),
                    best_streak: best_streak.min(limit),
                }
            })
    }

    #[test]
    fn levels_are_ordered_by_difficulty() {
        for pair in LEVELS.windows(2) {
            assert!(pair[0].digits <= pair[1].digits);
            assert!(pair[0].stimulus_frames >= pair[1].stimulus_frames);
        }
        assert_eq!(level_spec(0), LEVELS[0]);
        assert_eq!(level_spec(7), LEVELS[5]);
    }

    #[test]
    fn two_hits_promote_and_two_misses_demote() {
        let mut progress = ProgressState::default();
        assert_eq!(progress.record(true), LevelChange::None);
        assert_eq!(progress.record(true), LevelChange::Promoted(2));
        assert_eq!(progress.level, 2);
        assert_eq!(progress.record(false), LevelChange::None);
        assert_eq!(progress.record(false), LevelChange::Demoted(1));
        assert_eq!(progress.level, 1);
        assert_eq!(progress.attempts, 4);
        assert_eq!(progress.correct, 2);
    }

    #[test]
    fn level_is_bounded_at_both_ends() {
        let mut progress = ProgressState {
            level: MAX_LEVEL,
            ..ProgressState::default()
        };
        for _ in 0..10 {
            assert_eq!(progress.record(true), LevelChange::None);
        }
        assert_eq!(progress.level, MAX_LEVEL);

        let mut progress = ProgressState::default();
        for _ in 0..10 {
            assert_eq!(progress.record(false), LevelChange::None);
        }
        assert_eq!(progress.level, MIN_LEVEL);
    }

    #[test]
    fn accuracy_and_best_streak_are_tracked() {
        let mut progress = ProgressState::default();
        for result in [true, true, false, true] {
            progress.record(result);
        }
        assert!((progress.accuracy() - 0.75).abs() < 1e-6);
        assert_eq!(progress.best_streak, 2);
        assert_eq!(progress.attempts, 4);
    }

    #[test]
    fn encoding_round_trips() {
        let progress = ProgressState {
            level: 4,
            streak: 1,
            misses: 0,
            attempts: 20,
            correct: 15,
            best_streak: 6,
        };
        assert_eq!(ProgressState::decode(&progress.encode()), Some(progress));
    }

    #[test]
    fn decoding_rejects_corrupt_values() {
        assert_eq!(ProgressState::decode("v2,1,0,0,0,0,0"), None);
        assert_eq!(ProgressState::decode("v1,0,0,0,0,0,0"), None);
        assert_eq!(ProgressState::decode("v1,7,0,0,0,0,0"), None);
        assert_eq!(ProgressState::decode("v1,1,0,0,2,3,0"), None);
        assert_eq!(ProgressState::decode("v1,1,0,0,0,0,0,extra"), None);
        assert_eq!(ProgressState::decode(""), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: CASES, ..ProptestConfig::default() })]

        #[test]
        fn level_stays_in_range(mut state in any_progress(), results in prop::collection::vec(any::<bool>(), 0..200)) {
            for result in results {
                state.record(result);
                prop_assert!((MIN_LEVEL..=MAX_LEVEL).contains(&state.level));
            }
        }

        #[test]
        fn totals_are_consistent(state in any_progress(), results in prop::collection::vec(any::<bool>(), 0..200)) {
            let mut next = state;
            let successes = results.iter().filter(|result| **result).count() as u64;
            let total = results.len() as u64;
            for result in results {
                next.record(result);
            }
            prop_assert_eq!(next.attempts, state.attempts + total);
            prop_assert_eq!(next.correct, state.correct + successes);
            prop_assert!(next.correct <= next.attempts);
            prop_assert!(u64::from(next.best_streak) <= next.attempts);
            prop_assert!((0.0..=1.0).contains(&next.accuracy()));
        }

        #[test]
        fn encode_decode_round_trips(state in any_progress()) {
            prop_assert_eq!(ProgressState::decode(&state.encode()), Some(state));
        }

        #[test]
        fn successes_never_demote(mut state in any_progress(), count in 1usize..50) {
            for _ in 0..count {
                let change = state.record(true);
                prop_assert!(!matches!(change, LevelChange::Demoted(_)));
            }
        }
    }
}
