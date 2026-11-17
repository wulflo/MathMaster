// SPDX-License-Identifier: AGPL-3.0-or-later
//! Clasificación pedagógica de las sumas.
//!
//! Cada suma parcial de un drill se clasifica por la regla del soroban que exige como
//! mínimo: movimiento directo, complemento a 5, complemento a 10 o acarreo en cadena.
//! La clasificación es una aproximación explícita y documentada, no una simulación del
//! ábaco: sirve para agrupar errores por tipo de dificultad.

use core::fmt;

/// Caso pedagógico de una suma, de menos a más difícil.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PedagogicalCase {
    /// Movimiento directo, sin complementos ni acarreo.
    #[default]
    Direct,
    /// Complemento a 5 (amigo pequeño).
    Complement5,
    /// Complemento a 10 (amigo grande) con un solo acarreo.
    Complement10,
    /// Acarreo en cadena: dos o más posiciones generan acarreo.
    Carry,
}

impl PedagogicalCase {
    /// Todos los casos en orden de dificultad.
    pub const ALL: [Self; 4] = [
        Self::Direct,
        Self::Complement5,
        Self::Complement10,
        Self::Carry,
    ];

    /// Código estable para persistencia y CSV.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Complement5 => "c5",
            Self::Complement10 => "c10",
            Self::Carry => "carry",
        }
    }

    /// Reconstruye un caso desde [`Self::code`].
    #[must_use]
    pub const fn from_code(code: &str) -> Option<Self> {
        match code.as_bytes() {
            b"direct" => Some(Self::Direct),
            b"c5" => Some(Self::Complement5),
            b"c10" => Some(Self::Complement10),
            b"carry" => Some(Self::Carry),
            _ => None,
        }
    }

    /// Dificultad relativa: 0 para directo, 3 para acarreo en cadena.
    #[must_use]
    pub const fn severity(self) -> u8 {
        match self {
            Self::Direct => 0,
            Self::Complement5 => 1,
            Self::Complement10 => 2,
            Self::Carry => 3,
        }
    }
}

impl fmt::Display for PedagogicalCase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

/// Clasifica una suma `from + addend` por su caso pedagógico dominante.
///
/// Regla:
/// - dos o más posiciones con acarreo → [`PedagogicalCase::Carry`];
/// - exactamente una posición con acarreo → [`PedagogicalCase::Complement10`];
/// - sin acarreo pero con desbordamiento de las fichas terrestres (necesita el 5) →
///   [`PedagogicalCase::Complement5`];
/// - en otro caso → [`PedagogicalCase::Direct`].
///
/// Las fichas celestiales se ignoran a propósito: añadir un 5 es un movimiento directo.
#[must_use]
pub fn classify_addition(from: u128, addend: u128) -> PedagogicalCase {
    let mut left = from;
    let mut right = addend;
    let mut carry = 0u128;
    let mut carries = 0u32;
    let mut complement5 = false;

    loop {
        if left == 0 && right == 0 && carry == 0 {
            break;
        }
        let digit_left = left % 10;
        let digit_right = right % 10;
        let sum = digit_left + digit_right + carry;
        if sum >= 10 {
            carries = carries.saturating_add(1);
        }
        if (digit_left % 5) + (digit_right % 5) > 4 && digit_left + digit_right < 10 {
            complement5 = true;
        }
        carry = sum / 10;
        left /= 10;
        right /= 10;
    }

    if carries >= 2 {
        PedagogicalCase::Carry
    } else if carries == 1 {
        PedagogicalCase::Complement10
    } else if complement5 {
        PedagogicalCase::Complement5
    } else {
        PedagogicalCase::Direct
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const CASES: u32 = 10_000;

    #[test]
    fn direct_movements_are_classified_as_direct() {
        assert_eq!(classify_addition(0, 1), PedagogicalCase::Direct);
        assert_eq!(classify_addition(6, 1), PedagogicalCase::Direct);
        assert_eq!(classify_addition(5, 4), PedagogicalCase::Direct);
        assert_eq!(classify_addition(4, 5), PedagogicalCase::Direct);
        assert_eq!(classify_addition(123, 321), PedagogicalCase::Direct);
        assert_eq!(classify_addition(123, 456), PedagogicalCase::Complement5);
    }

    #[test]
    fn five_complements_are_detected() {
        assert_eq!(classify_addition(3, 4), PedagogicalCase::Complement5);
        assert_eq!(classify_addition(2, 3), PedagogicalCase::Complement5);
        assert_eq!(classify_addition(4, 1), PedagogicalCase::Complement5);
        assert_eq!(classify_addition(12, 4), PedagogicalCase::Complement5);
    }

    #[test]
    fn single_carries_need_the_ten_complement() {
        assert_eq!(classify_addition(8, 7), PedagogicalCase::Complement10);
        assert_eq!(classify_addition(9, 9), PedagogicalCase::Complement10);
        assert_eq!(classify_addition(6, 5), PedagogicalCase::Complement10);
    }

    #[test]
    fn chained_carries_are_the_hardest_case() {
        assert_eq!(classify_addition(99, 1), PedagogicalCase::Carry);
        assert_eq!(classify_addition(95, 7), PedagogicalCase::Carry);
        assert_eq!(classify_addition(999, 1), PedagogicalCase::Carry);
    }

    #[test]
    fn codes_round_trip_and_are_ordered_by_severity() {
        for (index, case) in PedagogicalCase::ALL.iter().enumerate() {
            assert_eq!(PedagogicalCase::from_code(case.code()), Some(*case));
            assert_eq!(usize::from(case.severity()), index);
        }
        assert_eq!(PedagogicalCase::from_code("nope"), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: CASES, ..ProptestConfig::default() })]
        #[test]
        fn classification_is_deterministic_and_valid(from in 0u128..1_000_000_000, addend in 0u128..1_000_000_000) {
            let first = classify_addition(from, addend);
            let second = classify_addition(from, addend);
            prop_assert_eq!(first, second);
            prop_assert!(PedagogicalCase::ALL.contains(&first));
        }

        #[test]
        fn adding_zero_never_adds_difficulty(from in 0u128..1_000_000_000) {
            prop_assert_eq!(classify_addition(from, 0), PedagogicalCase::Direct);
        }
    }
}
