// SPDX-License-Identifier: AGPL-3.0-or-later
//! Dígitos decimales del soroban (0..=9).

use std::fmt;

/// Error al construir un [`Digit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DigitError {
    /// El valor recibido no está en el rango 0..=9.
    #[error("dígito fuera de rango: {0} (se esperaba 0..=9)")]
    OutOfRange(u8),
}

/// Un dígito decimal válido, en el rango 0..=9.
///
/// La invariante se impone en la construcción: no existe un `Digit` inválido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digit(u8);

impl Digit {
    /// Dígito cero.
    pub const ZERO: Self = Self(0);
    /// Dígito nueve.
    pub const MAX: Self = Self(9);

    /// Construye un dígito válido.
    ///
    /// # Errors
    /// Devuelve [`DigitError::OutOfRange`] si `value > 9`.
    pub const fn new(value: u8) -> Result<Self, DigitError> {
        if value <= 9 {
            Ok(Self(value))
        } else {
            Err(DigitError::OutOfRange(value))
        }
    }

    /// Valor numérico del dígito.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }

    /// Construye un dígito a partir de un valor que el llamador ya validó.
    ///
    /// Es `pub(crate)` para que los módulos del core puedan materializar la invariante sin
    /// volver a comprobar el rango. El módulo `% 10` es una red de seguridad: nunca puede
    /// producir un valor fuera de 0..=9.
    pub(crate) const fn from_valid(value: u8) -> Self {
        Self(value % 10)
    }
}

impl TryFrom<u8> for Digit {
    type Error = DigitError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Digit> for u8 {
    fn from(digit: Digit) -> Self {
        digit.value()
    }
}

impl fmt::Display for Digit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_ten_valid_values() {
        for value in 0..=9 {
            let digit = Digit::new(value);
            assert!(digit.is_ok(), "el valor {value} debería ser válido");
        }
    }

    #[test]
    fn rejects_values_above_nine() {
        for value in 10..=u8::MAX {
            assert_eq!(Digit::new(value), Err(DigitError::OutOfRange(value)));
        }
    }

    #[test]
    fn round_trips_through_u8() {
        for value in 0..=9 {
            let digit = Digit::new(value);
            assert!(digit.is_ok());
            if let Ok(digit) = digit {
                assert_eq!(u8::from(digit), value);
            }
        }
    }
}
