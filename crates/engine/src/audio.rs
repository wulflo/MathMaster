// SPDX-License-Identifier: AGPL-3.0-or-later
//! Audio precalculado.
//!
//! El motor no abre dispositivos ni sintetiza en el callback: genera buffers completos
//! antes de reproducir. El backend (WebAudio o cpal) sólo copia o programa esas muestras,
//! sin asignaciones ni locks en el camino crítico.

use crate::schedule::FramePlan;

/// Frecuencia de muestreo por defecto.
pub const DEFAULT_SAMPLE_RATE: u32 = 48_000;

/// Especificación de un tono corto.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToneSpec {
    /// Frecuencia en Hz.
    pub frequency_hz: f32,
    /// Duración total en ms.
    pub duration_ms: f32,
    /// Amplitud máxima (0..=1).
    pub amplitude: f32,
    /// Ataque lineal en ms.
    pub attack_ms: f32,
    /// Liberación lineal en ms.
    pub release_ms: f32,
}

impl Default for ToneSpec {
    fn default() -> Self {
        Self {
            frequency_hz: 880.0,
            duration_ms: 30.0,
            amplitude: 0.35,
            attack_ms: 2.0,
            release_ms: 8.0,
        }
    }
}

/// Error de síntesis o de conversión temporal.
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum AudioError {
    /// La frecuencia de muestreo es cero.
    #[error("frecuencia de muestreo inválida: {0}")]
    SampleRate(u32),
    /// Algún parámetro del tono no es válido.
    #[error("tono inválido: {0}")]
    Tone(&'static str),
    /// La cadencia de frame no es positiva y finita.
    #[error("cadencia de frame inválida: {0} ms")]
    Frame(f64),
}

/// Buffer de un clic ya sintetizado.
#[derive(Debug, Clone, PartialEq)]
pub struct ClickBuffer {
    sample_rate: u32,
    samples: Vec<f32>,
}

impl ClickBuffer {
    /// Sintetiza un clic con [`ToneSpec::default()`].
    ///
    /// # Errors
    /// Devuelve [`AudioError`] si la frecuencia de muestreo o el tono no son válidos.
    pub fn new(sample_rate: u32) -> Result<Self, AudioError> {
        Self::from_spec(ToneSpec::default(), sample_rate)
    }

    /// Sintetiza un tono con envolvente lineal de ataque y liberación.
    ///
    /// # Errors
    /// Devuelve [`AudioError`] si algún parámetro no es válido.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "la duración se valida positiva y finita antes de redondearla"
    )]
    pub fn from_spec(spec: ToneSpec, sample_rate: u32) -> Result<Self, AudioError> {
        if sample_rate == 0 {
            return Err(AudioError::SampleRate(sample_rate));
        }
        if !spec.frequency_hz.is_finite() || spec.frequency_hz <= 0.0 {
            return Err(AudioError::Tone("frecuencia"));
        }
        if !spec.duration_ms.is_finite() || spec.duration_ms <= 0.0 {
            return Err(AudioError::Tone("duración"));
        }
        if !spec.amplitude.is_finite() || !(0.0..=1.0).contains(&spec.amplitude) {
            return Err(AudioError::Tone("amplitud"));
        }
        if !spec.attack_ms.is_finite() || spec.attack_ms < 0.0 {
            return Err(AudioError::Tone("ataque"));
        }
        if !spec.release_ms.is_finite() || spec.release_ms < 0.0 {
            return Err(AudioError::Tone("liberación"));
        }
        if spec.attack_ms + spec.release_ms > spec.duration_ms {
            return Err(AudioError::Tone("envolvente más larga que el tono"));
        }

        let length =
            (f64::from(spec.duration_ms) / 1000.0 * f64::from(sample_rate)).round() as usize;
        if length == 0 {
            return Err(AudioError::Tone("duración más corta que una muestra"));
        }
        let attack = (f64::from(spec.attack_ms) / 1000.0 * f64::from(sample_rate)) as usize;
        let release = (f64::from(spec.release_ms) / 1000.0 * f64::from(sample_rate)) as usize;
        let mut samples = Vec::with_capacity(length);
        let omega = core::f64::consts::TAU * f64::from(spec.frequency_hz);
        for index in 0..length {
            let time = index as f64 / f64::from(sample_rate);
            let envelope = if attack > 0 && index < attack {
                index as f64 / attack as f64
            } else if release > 0 && index + release >= length {
                (length - index) as f64 / release as f64
            } else {
                1.0
            };
            let value = (omega * time).sin() * envelope * f64::from(spec.amplitude);
            samples.push(value as f32);
        }

        Ok(Self {
            sample_rate,
            samples,
        })
    }

    /// Frecuencia de muestreo del buffer.
    #[must_use]
    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Muestras mono en `-1..=1`.
    #[must_use]
    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    /// Duración del buffer en ms.
    #[must_use]
    pub fn duration_ms(&self) -> f64 {
        self.samples.len() as f64 / f64::from(self.sample_rate) * 1000.0
    }
}

/// Convierte un frame del plan en un offset de muestra.
///
/// # Errors
/// Devuelve [`AudioError`] si la cadencia o la frecuencia de muestreo no son válidas.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "el offset se acota a u64::MAX antes de convertirlo"
)]
pub fn frame_to_sample(frame: u32, frame_ms: f64, sample_rate: u32) -> Result<u64, AudioError> {
    if sample_rate == 0 {
        return Err(AudioError::SampleRate(sample_rate));
    }
    if !frame_ms.is_finite() || frame_ms <= 0.0 {
        return Err(AudioError::Frame(frame_ms));
    }
    let seconds = f64::from(frame) * frame_ms / 1000.0;
    let sample = (seconds * f64::from(sample_rate)).round();
    if !sample.is_finite() || sample < 0.0 || sample > u64::MAX as f64 {
        return Err(AudioError::Frame(frame_ms));
    }
    Ok(sample as u64)
}

/// Offsets de muestra de cada estímulo del plan.
///
/// # Errors
/// Devuelve [`AudioError`] si la cadencia o la frecuencia de muestreo no son válidas.
pub fn click_offsets(
    plan: &FramePlan,
    frame_ms: f64,
    sample_rate: u32,
) -> Result<Vec<u64>, AudioError> {
    plan.stimuli()
        .map(|(_, start)| frame_to_sample(start, frame_ms, sample_rate))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schedule::FramePlan;

    const FRAME_MS: f64 = 1000.0 / 60.0;

    #[test]
    fn default_click_is_short_and_bounded() {
        let click = ClickBuffer::new(DEFAULT_SAMPLE_RATE);
        assert!(click.is_ok());
        if let Ok(click) = click {
            assert_eq!(click.sample_rate(), DEFAULT_SAMPLE_RATE);
            assert!((click.duration_ms() - 30.0).abs() < 0.1);
            assert_eq!(click.samples().len(), 1_440);
            for sample in click.samples() {
                assert!(sample.is_finite());
                assert!(*sample >= -0.35 && *sample <= 0.35);
            }
            assert!(
                click
                    .samples()
                    .first()
                    .is_some_and(|value| value.abs() < 1e-6)
            );
            assert!(
                click
                    .samples()
                    .last()
                    .is_some_and(|value| value.abs() < 0.01)
            );
        }
    }

    #[test]
    fn envelope_ramps_up_and_down() {
        let click = ClickBuffer::from_spec(
            ToneSpec {
                frequency_hz: 100.0,
                duration_ms: 100.0,
                amplitude: 1.0,
                attack_ms: 10.0,
                release_ms: 10.0,
            },
            10_000,
        );
        assert!(click.is_ok());
        if let Ok(click) = click {
            let samples = click.samples();
            assert!(samples[0].abs() < 1e-6);
            assert!(samples[75].abs() > samples[25].abs());
            assert!(samples[samples.len() - 1].abs() < 0.01);
        }
    }

    #[test]
    fn rejects_invalid_specs() {
        assert_eq!(ClickBuffer::new(0), Err(AudioError::SampleRate(0)));
        let base = ToneSpec::default();
        assert!(
            ClickBuffer::from_spec(
                ToneSpec {
                    frequency_hz: 0.0,
                    ..base
                },
                48_000
            )
            .is_err()
        );
        assert!(
            ClickBuffer::from_spec(
                ToneSpec {
                    amplitude: 2.0,
                    ..base
                },
                48_000
            )
            .is_err()
        );
        assert!(
            ClickBuffer::from_spec(
                ToneSpec {
                    attack_ms: 20.0,
                    release_ms: 20.0,
                    duration_ms: 30.0,
                    ..base
                },
                48_000
            )
            .is_err()
        );
    }

    #[test]
    fn frames_map_to_samples_at_sixty_hertz() {
        assert_eq!(frame_to_sample(0, FRAME_MS, 48_000), Ok(0));
        assert_eq!(frame_to_sample(6, FRAME_MS, 48_000), Ok(4_800));
        assert_eq!(frame_to_sample(1, FRAME_MS, 48_000), Ok(800));
        assert!(frame_to_sample(1, 0.0, 48_000).is_err());
    }

    #[test]
    fn click_offsets_follow_the_plan() {
        let plan = FramePlan::builder()
            .stimulus(30)
            .and_then(|builder| builder.gap(6))
            .and_then(|builder| builder.stimulus(30))
            .and_then(|builder| builder.build())
            .unwrap_or_default();
        assert_eq!(plan.stimulus_count(), 2);
        let offsets = click_offsets(&plan, FRAME_MS, 48_000);
        assert_eq!(offsets, Ok(vec![0, 28_800]));
    }
}
