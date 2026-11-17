// SPDX-License-Identifier: AGPL-3.0-or-later
//! Repaso espaciado propio, inspirado en SM-2/FSRS.
//!
//! No pretende ser FSRS: es un planificador transparente de intervalos con facilidad
//! acotada, sin dependencias opacas. Cada tarjeta es un nivel de progresión y el estado
//! se serializa en una línea. La UI lo usa como aviso de repaso; no bloquea la práctica.

/// Milisegundos de un día.
pub const DAY_MS: u64 = 86_400_000;
/// Intervalo máximo: un año.
pub const MAX_INTERVAL_MS: u64 = 365 * DAY_MS;
/// Facilidad mínima (1,30).
pub const MIN_EASE: u16 = 1_300;
/// Facilidad máxima (3,00).
pub const MAX_EASE: u16 = 3_000;
/// Facilidad inicial (2,50).
pub const DEFAULT_EASE: u16 = 2_500;

/// Resultado de una sesión para el planificador.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReviewOutcome {
    /// Falló: vuelve a empezar en minutos.
    Again,
    /// Acertó con esfuerzo.
    Hard,
    /// Acertó.
    Good,
    /// Acertó con facilidad.
    Easy,
}

/// Estado de repaso de una tarjeta (un nivel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReviewState {
    /// Repeticiones exitosas consecutivas.
    pub reps: u16,
    /// Fallos acumulados.
    pub lapses: u16,
    /// Intervalo actual en milisegundos.
    pub interval_ms: u64,
    /// Facilidad en milésimas (1,30..=3,00).
    pub ease_permille: u16,
    /// Próxima fecha de repaso, Unix en milisegundos.
    pub due_ms: u64,
}

impl Default for ReviewState {
    fn default() -> Self {
        Self {
            reps: 0,
            lapses: 0,
            interval_ms: 0,
            ease_permille: DEFAULT_EASE,
            due_ms: 0,
        }
    }
}

impl ReviewState {
    /// Indica si la tarjeta toca repasar.
    #[must_use]
    pub const fn is_due(self, now_ms: u64) -> bool {
        self.due_ms <= now_ms
    }

    /// Aplica un resultado y devuelve el estado siguiente.
    #[must_use]
    pub fn review(self, outcome: ReviewOutcome, now_ms: u64) -> Self {
        let (reps, lapses, interval_ms, ease_permille) = match outcome {
            ReviewOutcome::Again => (
                0,
                self.lapses.saturating_add(1),
                10 * 60 * 1_000,
                self.ease_permille.saturating_sub(200).max(MIN_EASE),
            ),
            ReviewOutcome::Hard => (
                self.reps.saturating_add(1),
                self.lapses,
                grow(self.interval_ms, 1_200, DAY_MS),
                self.ease_permille.saturating_sub(150).max(MIN_EASE),
            ),
            ReviewOutcome::Good => (
                self.reps.saturating_add(1),
                self.lapses,
                match self.reps {
                    0 => DAY_MS,
                    1 => 3 * DAY_MS,
                    _ => grow(self.interval_ms, self.ease_permille, 3 * DAY_MS),
                },
                self.ease_permille,
            ),
            ReviewOutcome::Easy => (
                self.reps.saturating_add(1),
                self.lapses,
                grow(
                    self.interval_ms,
                    self.ease_permille.saturating_add(150),
                    4 * DAY_MS,
                )
                .max(4 * DAY_MS),
                self.ease_permille.saturating_add(150).min(MAX_EASE),
            ),
        };
        let interval_ms = interval_ms.min(MAX_INTERVAL_MS);
        Self {
            reps,
            lapses,
            interval_ms,
            ease_permille,
            due_ms: now_ms.saturating_add(interval_ms),
        }
    }

    /// Serializa el estado en una línea.
    #[must_use]
    pub fn encode(self) -> String {
        format!(
            "v1,{},{},{},{},{}",
            self.reps, self.lapses, self.interval_ms, self.ease_permille, self.due_ms
        )
    }

    /// Reconstruye el estado desde [`Self::encode`].
    #[must_use]
    pub fn decode(raw: &str) -> Option<Self> {
        let mut parts = raw.split(',');
        if parts.next() != Some("v1") {
            return None;
        }
        let reps = parts.next()?.parse::<u16>().ok()?;
        let lapses = parts.next()?.parse::<u16>().ok()?;
        let interval_ms = parts.next()?.parse::<u64>().ok()?;
        let ease_permille = parts.next()?.parse::<u16>().ok()?;
        let due_ms = parts.next()?.parse::<u64>().ok()?;
        if parts.next().is_some()
            || interval_ms > MAX_INTERVAL_MS
            || !(MIN_EASE..=MAX_EASE).contains(&ease_permille)
        {
            return None;
        }
        Some(Self {
            reps,
            lapses,
            interval_ms,
            ease_permille,
            due_ms,
        })
    }
}

/// Multiplica `interval_ms` por `factor_permille / 1000`, con saturación y mínimo.
fn grow(interval_ms: u64, factor_permille: u16, minimum_ms: u64) -> u64 {
    let scaled = u128::from(interval_ms) * u128::from(factor_permille) / 1_000;
    let value = u64::try_from(scaled).unwrap_or(u64::MAX);
    value.max(minimum_ms).min(MAX_INTERVAL_MS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const CASES: u32 = 10_000;
    const NOW: u64 = 1_700_000_000_000;

    #[test]
    fn again_restarts_in_ten_minutes() {
        let state = ReviewState::default().review(ReviewOutcome::Good, NOW);
        let next = state.review(ReviewOutcome::Again, NOW);
        assert_eq!(next.reps, 0);
        assert_eq!(next.lapses, 1);
        assert_eq!(next.interval_ms, 10 * 60 * 1_000);
        assert_eq!(next.due_ms, NOW + 10 * 60 * 1_000);
        assert_eq!(next.ease_permille, DEFAULT_EASE - 200);
    }

    #[test]
    fn good_intervals_grow() {
        let first = ReviewState::default().review(ReviewOutcome::Good, NOW);
        let second = first.review(ReviewOutcome::Good, NOW);
        let third = second.review(ReviewOutcome::Good, NOW);
        assert_eq!(first.interval_ms, DAY_MS);
        assert_eq!(second.interval_ms, 3 * DAY_MS);
        assert!(third.interval_ms > second.interval_ms);
    }

    #[test]
    fn intervals_and_ease_stay_bounded() {
        let mut state = ReviewState::default();
        for _ in 0..200 {
            state = state.review(ReviewOutcome::Easy, NOW);
        }
        assert!(state.interval_ms <= MAX_INTERVAL_MS);
        assert_eq!(state.ease_permille, MAX_EASE);

        let mut state = ReviewState::default();
        for _ in 0..200 {
            state = state.review(ReviewOutcome::Again, NOW);
        }
        assert_eq!(state.ease_permille, MIN_EASE);
        assert!(state.lapses >= 200);
    }

    #[test]
    fn due_moves_forward_unless_clock_goes_back() {
        for outcome in [
            ReviewOutcome::Again,
            ReviewOutcome::Hard,
            ReviewOutcome::Good,
            ReviewOutcome::Easy,
        ] {
            let state = ReviewState::default().review(outcome, NOW);
            assert!(state.due_ms >= NOW);
            assert!(state.interval_ms > 0);
        }
    }

    #[test]
    fn encoding_round_trips_and_rejects_corruption() {
        let state = ReviewState::default().review(ReviewOutcome::Hard, NOW);
        assert_eq!(ReviewState::decode(&state.encode()), Some(state));
        assert_eq!(ReviewState::decode("v1,0,0,0,2500,0,extra"), None);
        assert_eq!(ReviewState::decode("v1,0,0,0,999,0"), None);
        assert_eq!(ReviewState::decode("v1,0,0,999999999999,2500,0"), None);
        assert_eq!(ReviewState::decode(""), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: CASES, ..ProptestConfig::default() })]
        #[test]
        fn review_advances_due_and_keeps_bounds(
            reps in 0u16..100,
            lapses in 0u16..100,
            interval_ms in 0u64..=MAX_INTERVAL_MS,
            ease in MIN_EASE..=MAX_EASE,
            due_ms in any::<u64>(),
            now_ms in 0u64..4_000_000_000_000,
            outcome in prop::sample::select(vec![
                ReviewOutcome::Again,
                ReviewOutcome::Hard,
                ReviewOutcome::Good,
                ReviewOutcome::Easy,
            ]),
        ) {
            let state = ReviewState { reps, lapses, interval_ms, ease_permille: ease, due_ms };
            let next = state.review(outcome, now_ms);
            prop_assert!(next.due_ms >= now_ms);
            prop_assert!(next.interval_ms > 0);
            prop_assert!(next.interval_ms <= MAX_INTERVAL_MS);
            prop_assert!((MIN_EASE..=MAX_EASE).contains(&next.ease_permille));
            prop_assert_eq!(ReviewState::decode(&next.encode()), Some(next));
            if outcome == ReviewOutcome::Again {
                prop_assert_eq!(next.reps, 0);
                prop_assert_eq!(next.interval_ms, 10 * 60 * 1_000);
            }
        }
    }
}
