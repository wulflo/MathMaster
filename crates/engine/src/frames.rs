// SPDX-License-Identifier: AGPL-3.0-or-later
//! Conversión explícita de duraciones objetivo a frames enteros.
//!
//! La planificación es en frames (ADR-002). Nunca se redondea “a ojo”: se calcula con la
//! cadencia medida y se devuelve el número de frames que más se acerca al objetivo.

/// Error de planificación temporal.
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum FrameError {
    /// La duración objetivo no es un número positivo y finito.
    #[error("duración objetivo inválida: {0} ms")]
    InvalidTarget(f64),
    /// La duración de frame no es un número positivo y finito.
    #[error("duración de frame inválida: {0} ms")]
    InvalidFrame(f64),
    /// El número de frames calculado no cabe en `u32`.
    #[error("el plan excede el máximo de frames representable")]
    TooManyFrames,
}

/// Calcula cuántos frames enteros cubren `target_ms` a una cadencia de `frame_ms`.
///
/// # Errors
/// Devuelve [`FrameError`] si alguna duración no es positiva/finita o si el resultado
/// no cabe en `u32`.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "el valor se valida contra 1.0..=u32::MAX antes de convertirlo"
)]
pub fn frames_for_duration(target_ms: f64, frame_ms: f64) -> Result<u32, FrameError> {
    if !target_ms.is_finite() || target_ms <= 0.0 {
        return Err(FrameError::InvalidTarget(target_ms));
    }
    if !frame_ms.is_finite() || frame_ms <= 0.0 {
        return Err(FrameError::InvalidFrame(frame_ms));
    }

    let frames = (target_ms / frame_ms).round();
    if !(1.0..=f64::from(u32::MAX)).contains(&frames) {
        return Err(FrameError::TooManyFrames);
    }

    Ok(frames as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_frames_at_sixty_hertz() {
        let frames = frames_for_duration(100.0, 16.666_666_666_666_668);
        assert_eq!(frames, Ok(6));
    }

    #[test]
    fn rounds_to_the_nearest_frame() {
        assert_eq!(frames_for_duration(90.0, 16.666_7), Ok(5));
        assert_eq!(frames_for_duration(110.0, 16.666_7), Ok(7));
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert_eq!(
            frames_for_duration(0.0, 16.0),
            Err(FrameError::InvalidTarget(0.0))
        );
        assert_eq!(
            frames_for_duration(100.0, 0.0),
            Err(FrameError::InvalidFrame(0.0))
        );
        let error = frames_for_duration(f64::NAN, 16.0);
        assert!(matches!(error, Err(FrameError::InvalidTarget(value)) if value.is_nan()));
    }
}
