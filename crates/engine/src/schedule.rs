// SPDX-License-Identifier: AGPL-3.0-or-later
//! Planes temporales en frames y telemetría de presentación.
//!
//! Un [`FramePlan`] se construye una sola vez y luego sólo se consulta por índice: el
//! bucle de presentación no asigna memoria ni bloquea. La telemetría usa ventanas de
//! tamaño fijo, de modo que también se puede alimentar desde el camino crítico.

use core::fmt;

/// Capacidad de cada ventana de telemetría.
pub const TELEMETRY_CAPACITY: usize = 600;

/// Tipo de segmento de un plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SegmentKind {
    /// Se muestra un estímulo.
    Stimulus,
    /// No se muestra nada (hueco entre estímulos).
    Gap,
}

impl SegmentKind {
    /// Indica si el segmento muestra un estímulo.
    #[must_use]
    pub const fn is_stimulus(self) -> bool {
        matches!(self, Self::Stimulus)
    }
}

/// Segmento de un plan temporal.
///
/// `start` y `frames` son absolutos y relativos al inicio del plan, respectivamente.
/// `stimulus` es el índice de estímulo (0-based) para los segmentos de estímulo y `None`
/// para los huecos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Segment {
    /// Tipo de segmento.
    pub kind: SegmentKind,
    /// Frame absoluto de inicio.
    pub start: u32,
    /// Duración en frames (siempre > 0).
    pub frames: u32,
    /// Índice de estímulo, si aplica.
    pub stimulus: Option<u32>,
}

impl Segment {
    /// Frame absoluto en el que termina el segmento (exclusivo).
    #[must_use]
    pub const fn end(self) -> u32 {
        self.start.saturating_add(self.frames)
    }

    /// Indica si `frame` cae dentro del segmento.
    #[must_use]
    pub const fn contains(self, frame: u32) -> bool {
        frame >= self.start && frame < self.end()
    }
}

/// Error al construir un plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PlanError {
    /// Un segmento no puede durar cero frames.
    #[error("un segmento no puede durar 0 frames")]
    ZeroFrames,
    /// El plan excede el rango representable en `u32`.
    #[error("el plan excede el máximo de frames representable")]
    TooLong,
    /// El plan no tiene ningún segmento.
    #[error("el plan no tiene segmentos")]
    Empty,
}

/// Plan temporal precomputado.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FramePlan {
    segments: Vec<Segment>,
    total_frames: u32,
    stimuli: u32,
}

impl FramePlan {
    /// Crea un constructor de planes.
    #[must_use]
    pub fn builder() -> FramePlanBuilder {
        FramePlanBuilder::new()
    }

    /// Segmentos en orden temporal.
    #[must_use]
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Duración total en frames.
    #[must_use]
    pub const fn total_frames(&self) -> u32 {
        self.total_frames
    }

    /// Cantidad de estímulos del plan.
    #[must_use]
    pub const fn stimulus_count(&self) -> u32 {
        self.stimuli
    }

    /// Indica si el plan está vacío (no debería ocurrir tras [`FramePlan::builder`]).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Duración total en milisegundos para una cadencia de frame dada.
    #[must_use]
    pub fn duration_ms(&self, frame_ms: f64) -> f64 {
        f64::from(self.total_frames) * frame_ms
    }

    /// Segmento que contiene `frame`, si existe.
    ///
    /// Búsqueda binaria: O(log n) y sin asignaciones.
    #[must_use]
    pub fn segment_at(&self, frame: u32) -> Option<Segment> {
        if frame >= self.total_frames {
            return None;
        }
        let index = match self
            .segments
            .binary_search_by(|segment| segment.start.cmp(&frame))
        {
            Ok(index) => index,
            Err(index) => index.checked_sub(1)?,
        };
        self.segments.get(index).copied()
    }

    /// Tipo de segmento que contiene `frame`.
    #[must_use]
    pub fn kind_at(&self, frame: u32) -> Option<SegmentKind> {
        self.segment_at(frame).map(|segment| segment.kind)
    }

    /// Índice de estímulo visible en `frame`, si lo hay.
    #[must_use]
    pub fn stimulus_at(&self, frame: u32) -> Option<u32> {
        self.segment_at(frame).and_then(|segment| segment.stimulus)
    }

    /// Estímulos como `(índice, frame de inicio)`.
    pub fn stimuli(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.segments
            .iter()
            .filter_map(|segment| Some((segment.stimulus?, segment.start)))
    }
}

impl fmt::Display for FramePlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} estímulos, {} frames, {} segmentos",
            self.stimuli,
            self.total_frames,
            self.segments.len()
        )
    }
}

/// Constructor de [`FramePlan`].
#[derive(Debug, Clone, Default)]
pub struct FramePlanBuilder {
    segments: Vec<Segment>,
    cursor: u32,
    stimuli: u32,
}

impl FramePlanBuilder {
    /// Constructor vacío.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            segments: Vec::new(),
            cursor: 0,
            stimuli: 0,
        }
    }

    /// Añade un estímulo de `frames` frames.
    ///
    /// # Errors
    /// Devuelve [`PlanError`] si la duración es cero o el plan desborda `u32`.
    pub fn stimulus(mut self, frames: u32) -> Result<Self, PlanError> {
        self.push(SegmentKind::Stimulus, frames)?;
        Ok(self)
    }

    /// Añade un hueco de `frames` frames.
    ///
    /// # Errors
    /// Devuelve [`PlanError`] si la duración es cero o el plan desborda `u32`.
    pub fn gap(mut self, frames: u32) -> Result<Self, PlanError> {
        self.push(SegmentKind::Gap, frames)?;
        Ok(self)
    }

    fn push(&mut self, kind: SegmentKind, frames: u32) -> Result<(), PlanError> {
        if frames == 0 {
            return Err(PlanError::ZeroFrames);
        }
        let end = self.cursor.checked_add(frames).ok_or(PlanError::TooLong)?;
        let stimulus = if kind.is_stimulus() {
            let index = self.stimuli;
            self.stimuli = self.stimuli.checked_add(1).ok_or(PlanError::TooLong)?;
            Some(index)
        } else {
            None
        };
        self.segments.push(Segment {
            kind,
            start: self.cursor,
            frames,
            stimulus,
        });
        self.cursor = end;
        Ok(())
    }

    /// Finaliza el plan.
    ///
    /// # Errors
    /// Devuelve [`PlanError::Empty`] si no se añadió ningún segmento.
    pub fn build(self) -> Result<FramePlan, PlanError> {
        if self.segments.is_empty() {
            return Err(PlanError::Empty);
        }
        Ok(FramePlan {
            segments: self.segments,
            total_frames: self.cursor,
            stimuli: self.stimuli,
        })
    }
}

/// Error al registrar telemetría.
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum TelemetryError {
    /// La marca de tiempo no es finita o es negativa.
    #[error("marca de tiempo inválida: {0}")]
    InvalidTimestamp(f64),
    /// La cadencia de frame no es positiva y finita.
    #[error("cadencia de frame inválida: {0} ms")]
    InvalidFrame(f64),
}

/// Resumen de telemetría.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TelemetrySummary {
    /// Cantidad de deltas de frame registrados.
    pub frame_samples: usize,
    /// Mediana del delta de frame en ms.
    pub frame_p50_ms: f32,
    /// Percentil 95 del delta de frame en ms.
    pub frame_p95_ms: f32,
    /// Delta máximo de frame en ms.
    pub frame_max_ms: f32,
    /// Cantidad de apariciones de estímulo medidas.
    pub onset_samples: usize,
    /// Mediana del error absoluto de aparición en ms.
    pub onset_p50_ms: f32,
    /// Percentil 95 del error absoluto de aparición en ms.
    pub onset_p95_ms: f32,
    /// Error absoluto máximo de aparición en ms.
    pub onset_max_ms: f32,
}

impl TelemetrySummary {
    /// Indica si hay alguna medición.
    #[must_use]
    pub const fn has_samples(self) -> bool {
        self.frame_samples > 0 || self.onset_samples > 0
    }
}

/// Telemetría de presentación con ventanas de tamaño fijo.
///
/// No asigna memoria ni usa locks: se puede alimentar desde el callback de
/// `requestAnimationFrame`.
#[derive(Debug, Clone)]
pub struct FrameTelemetry {
    deltas: [f32; TELEMETRY_CAPACITY],
    delta_len: usize,
    delta_next: usize,
    delta_sum: f64,
    last_ms: Option<f64>,
    onsets: [f32; TELEMETRY_CAPACITY],
    onset_len: usize,
    onset_next: usize,
}

impl Default for FrameTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameTelemetry {
    /// Telemetría vacía.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            deltas: [0.0; TELEMETRY_CAPACITY],
            delta_len: 0,
            delta_next: 0,
            delta_sum: 0.0,
            last_ms: None,
            onsets: [0.0; TELEMETRY_CAPACITY],
            onset_len: 0,
            onset_next: 0,
        }
    }

    /// Vacía la telemetría conservando la capacidad.
    pub fn reset(&mut self) {
        self.delta_len = 0;
        self.delta_next = 0;
        self.delta_sum = 0.0;
        self.last_ms = None;
        self.onset_len = 0;
        self.onset_next = 0;
    }

    /// Registra un frame con su marca de tiempo en ms.
    ///
    /// El primer frame sólo fija la referencia; los siguientes aportan un delta.
    ///
    /// # Errors
    /// Devuelve [`TelemetryError::InvalidTimestamp`] si `now_ms` no es finito o es negativo.
    pub fn record_frame(&mut self, now_ms: f64) -> Result<(), TelemetryError> {
        if !now_ms.is_finite() || now_ms < 0.0 {
            return Err(TelemetryError::InvalidTimestamp(now_ms));
        }
        if let Some(previous) = self.last_ms {
            let delta = now_ms - previous;
            if delta.is_finite() && delta > 0.0 {
                self.deltas[self.delta_next] = delta as f32;
                self.delta_next = (self.delta_next + 1) % TELEMETRY_CAPACITY;
                self.delta_len = (self.delta_len + 1).min(TELEMETRY_CAPACITY);
                self.delta_sum += delta;
            }
        }
        self.last_ms = Some(now_ms);
        Ok(())
    }

    /// Registra la aparición de un estímulo.
    ///
    /// `elapsed_ms` es el tiempo desde el inicio del plan y `planned_ms` el instante
    /// planificado. Se guarda el error absoluto.
    ///
    /// # Errors
    /// Devuelve [`TelemetryError::InvalidTimestamp`] si algún valor no es finito o es
    /// negativo.
    pub fn record_onset(&mut self, planned_ms: f64, elapsed_ms: f64) -> Result<(), TelemetryError> {
        if !planned_ms.is_finite() || planned_ms < 0.0 {
            return Err(TelemetryError::InvalidTimestamp(planned_ms));
        }
        if !elapsed_ms.is_finite() || elapsed_ms < 0.0 {
            return Err(TelemetryError::InvalidTimestamp(elapsed_ms));
        }
        let error = (elapsed_ms - planned_ms).abs() as f32;
        self.onsets[self.onset_next] = error;
        self.onset_next = (self.onset_next + 1) % TELEMETRY_CAPACITY;
        self.onset_len = (self.onset_len + 1).min(TELEMETRY_CAPACITY);
        Ok(())
    }

    /// Media de los deltas registrados, si hay alguno.
    #[must_use]
    pub fn mean_frame_ms(&self) -> Option<f64> {
        if self.delta_len == 0 {
            None
        } else {
            Some(self.delta_sum / self.delta_len as f64)
        }
    }

    /// Resumen de las ventanas actuales.
    #[must_use]
    pub fn summary(&self) -> TelemetrySummary {
        let mut deltas = self.deltas;
        deltas[..self.delta_len].sort_unstable_by(f32::total_cmp);
        let mut onsets = self.onsets;
        onsets[..self.onset_len].sort_unstable_by(f32::total_cmp);

        TelemetrySummary {
            frame_samples: self.delta_len,
            frame_p50_ms: percentile(&deltas[..self.delta_len], 0.50),
            frame_p95_ms: percentile(&deltas[..self.delta_len], 0.95),
            frame_max_ms: deltas[..self.delta_len].last().copied().unwrap_or(0.0),
            onset_samples: self.onset_len,
            onset_p50_ms: percentile(&onsets[..self.onset_len], 0.50),
            onset_p95_ms: percentile(&onsets[..self.onset_len], 0.95),
            onset_max_ms: onsets[..self.onset_len].last().copied().unwrap_or(0.0),
        }
    }
}

fn percentile(sorted: &[f32], quantile: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (quantile * sorted.len() as f32).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted.len() - 1);
    sorted[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME_MS: f64 = 1000.0 / 60.0;

    fn plan_of(terms: u32) -> FramePlan {
        let mut builder = FramePlan::builder();
        for _ in 0..terms {
            match builder.stimulus(30).and_then(|next| next.gap(6)) {
                Ok(next) => builder = next,
                Err(_) => return FramePlan::default(),
            }
        }
        builder.build().unwrap_or_default()
    }

    #[test]
    fn six_frames_are_one_hundred_milliseconds() {
        let frames = crate::frames_for_duration(100.0, FRAME_MS);
        assert_eq!(frames, Ok(6));
    }

    #[test]
    fn plan_segments_are_contiguous_and_ordered() {
        let plan = plan_of(4);
        assert_eq!(plan.stimulus_count(), 4);
        assert_eq!(plan.total_frames(), 4 * 36);
        assert_eq!(plan.segments().len(), 8);
        for pair in plan.segments().windows(2) {
            assert_eq!(pair[0].end(), pair[1].start);
        }
        assert_eq!(plan.duration_ms(FRAME_MS), 144.0 * FRAME_MS);
    }

    #[test]
    fn segment_lookup_handles_edges() {
        let plan = plan_of(2);
        assert_eq!(plan.kind_at(0), Some(SegmentKind::Stimulus));
        assert_eq!(plan.kind_at(29), Some(SegmentKind::Stimulus));
        assert_eq!(plan.kind_at(30), Some(SegmentKind::Gap));
        assert_eq!(plan.kind_at(35), Some(SegmentKind::Gap));
        assert_eq!(plan.kind_at(36), Some(SegmentKind::Stimulus));
        assert_eq!(plan.kind_at(71), Some(SegmentKind::Gap));
        assert_eq!(plan.kind_at(72), None);
        assert_eq!(plan.stimulus_at(10), Some(0));
        assert_eq!(plan.stimulus_at(40), Some(1));
        assert_eq!(plan.stimulus_at(60), Some(1));
        assert_eq!(plan.stimulus_at(66), None);
    }

    #[test]
    fn stimuli_iterator_matches_segments() {
        let plan = plan_of(3);
        let stimuli: Vec<(u32, u32)> = plan.stimuli().collect();
        assert_eq!(stimuli, vec![(0, 0), (1, 36), (2, 72)]);
    }

    #[test]
    fn builder_rejects_invalid_plans() {
        assert_eq!(FramePlan::builder().build(), Err(PlanError::Empty));
        assert!(matches!(
            FramePlan::builder().stimulus(0),
            Err(PlanError::ZeroFrames)
        ));
        assert!(matches!(
            FramePlan::builder().gap(0),
            Err(PlanError::ZeroFrames)
        ));
        assert!(matches!(
            FramePlan::builder()
                .stimulus(u32::MAX)
                .and_then(|b| b.gap(1)),
            Err(PlanError::TooLong)
        ));
    }

    #[test]
    fn telemetry_summarises_frames_and_onsets() {
        let mut telemetry = FrameTelemetry::new();
        assert_eq!(telemetry.summary().frame_samples, 0);
        assert_eq!(telemetry.record_frame(0.0), Ok(()));
        for step in 1..=100 {
            assert_eq!(telemetry.record_frame(f64::from(step) * 16.0), Ok(()));
        }
        assert_eq!(telemetry.record_onset(0.0, 1.0), Ok(()));
        assert_eq!(telemetry.record_onset(100.0, 102.0), Ok(()));
        let summary = telemetry.summary();
        assert_eq!(summary.frame_samples, 100);
        assert!((summary.frame_p50_ms - 16.0).abs() < 1e-4);
        assert!((summary.frame_max_ms - 16.0).abs() < 1e-4);
        assert_eq!(summary.onset_samples, 2);
        assert!((summary.onset_p50_ms - 1.0).abs() < 1e-4);
        assert!((summary.onset_p95_ms - 2.0).abs() < 1e-4);
        assert!(summary.has_samples());
    }

    #[test]
    fn telemetry_rejects_invalid_values() {
        let mut telemetry = FrameTelemetry::new();
        assert!(matches!(
            telemetry.record_frame(f64::NAN),
            Err(TelemetryError::InvalidTimestamp(value)) if value.is_nan()
        ));
        assert_eq!(
            telemetry.record_onset(-1.0, 0.0),
            Err(TelemetryError::InvalidTimestamp(-1.0))
        );
    }

    #[test]
    fn telemetry_window_wraps_without_allocating() {
        let mut telemetry = FrameTelemetry::new();
        assert_eq!(telemetry.record_frame(0.0), Ok(()));
        for step in 1..=(TELEMETRY_CAPACITY + 50) {
            assert_eq!(
                telemetry.record_frame(f64::from(step as u32) * 10.0),
                Ok(())
            );
        }
        let summary = telemetry.summary();
        assert_eq!(summary.frame_samples, TELEMETRY_CAPACITY);
        assert!((summary.frame_p50_ms - 10.0).abs() < 1e-4);
    }
}
