// SPDX-License-Identifier: AGPL-3.0-or-later
//! Audio nativo con cpal.
//!
//! El hilo de audio es el único dueño del `Stream`; los clics viajan por una cola SPSC
//! (`rtrb`). El callback sólo lee muestras precalculadas, escribe el buffer y actualiza
//! contadores atómicos: ni asignaciones ni locks. La telemetría (jitter por bloque,
//! clics y errores) se guarda en un histograma de tamaño fijo.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, FromSample, OutputCallbackInfo, SampleFormat, SizedSample, StreamConfig};
use rtrb::{Producer, RingBuffer};
use serde::Serialize;

/// Frames por bloque del callback de audio.
pub const BLOCK_FRAMES: u32 = 256;
/// Capacidad de la cola de clics pendientes.
const CLICK_QUEUE: usize = 64;
/// Bins del histograma de jitter (50 µs cada uno).
const JITTER_BINS: usize = 32;
/// Resolución del histograma en microsegundos.
const JITTER_STEP_US: u64 = 50;

/// Información del dispositivo abierto.
#[derive(Debug, Clone, Serialize)]
pub struct AudioInfo {
    /// Nombre del dispositivo de salida.
    pub device: String,
    /// Frecuencia de muestreo en Hz.
    pub sample_rate: u32,
    /// Canales de salida.
    pub channels: u16,
    /// Frames por bloque solicitados.
    pub block_frames: u32,
}

/// Contadores y percentiles del callback de audio.
#[derive(Debug, Clone, Serialize)]
pub struct AudioStats {
    /// Dispositivo activo.
    pub device: String,
    /// Frecuencia de muestreo en Hz.
    pub sample_rate: u32,
    /// Canales de salida.
    pub channels: u16,
    /// Frames por bloque.
    pub block_frames: u32,
    /// Callbacks ejecutados.
    pub callbacks: u64,
    /// Frames renderizados (reloj de audio).
    pub frames: u64,
    /// Segundos transcurridos según el reloj de audio.
    pub seconds: f64,
    /// Clics pedidos por la UI.
    pub clicks_queued: u64,
    /// Clics iniciados en el callback.
    pub clicks_started: u64,
    /// Clics rechazados por cola llena.
    pub queue_full: u64,
    /// Bloques con retraso superior al 50 % del periodo.
    pub late_blocks: u64,
    /// Errores notificados por cpal.
    pub errors: u64,
    /// Corriente activa.
    pub active: bool,
    /// Jitter del último callback en µs.
    pub jitter_last_us: u64,
    /// Jitter máximo observado en µs.
    pub jitter_max_us: u64,
    /// Jitter p95 aproximado en µs.
    pub jitter_p95_us: u64,
}

/// Estado compartido entre el hilo de audio, el callback y los comandos Tauri.
struct Shared {
    producer: Mutex<Option<Producer<()>>>,
    active: AtomicBool,
    clock_frames: AtomicU64,
    callbacks: AtomicU64,
    clicks_queued: AtomicU64,
    clicks_started: AtomicU64,
    queue_full: AtomicU64,
    late_blocks: AtomicU64,
    errors: AtomicU64,
    jitter_last_us: AtomicU64,
    jitter_max_us: AtomicU64,
    jitter_hist: [AtomicU64; JITTER_BINS],
}

impl Shared {
    fn new() -> Self {
        Self {
            producer: Mutex::new(None),
            active: AtomicBool::new(false),
            clock_frames: AtomicU64::new(0),
            callbacks: AtomicU64::new(0),
            clicks_queued: AtomicU64::new(0),
            clicks_started: AtomicU64::new(0),
            queue_full: AtomicU64::new(0),
            late_blocks: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            jitter_last_us: AtomicU64::new(0),
            jitter_max_us: AtomicU64::new(0),
            jitter_hist: core::array::from_fn(|_| AtomicU64::new(0)),
        }
    }

    fn record_jitter(&self, nanos: u64) {
        let micros = nanos / 1_000;
        self.jitter_last_us.store(micros, Ordering::Relaxed);
        self.jitter_max_us.fetch_max(micros, Ordering::Relaxed);
        #[allow(
            clippy::cast_possible_truncation,
            reason = "el índice se acota a JITTER_BINS - 1"
        )]
        let bin = ((micros / JITTER_STEP_US) as usize).min(JITTER_BINS - 1);
        self.jitter_hist[bin].fetch_add(1, Ordering::Relaxed);
    }

    fn percentile_us(&self, percentile: u64) -> u64 {
        let total: u64 = self
            .jitter_hist
            .iter()
            .map(|bin| bin.load(Ordering::Relaxed))
            .sum();
        if total == 0 {
            return 0;
        }
        let target = total * percentile / 100;
        let mut seen = 0u64;
        for (index, bin) in self.jitter_hist.iter().enumerate() {
            seen += bin.load(Ordering::Relaxed);
            if seen >= target {
                return (index as u64 + 1) * JITTER_STEP_US;
            }
        }
        JITTER_BINS as u64 * JITTER_STEP_US
    }

    fn stats(&self, info: &AudioInfo) -> AudioStats {
        let frames = self.clock_frames.load(Ordering::Relaxed);
        AudioStats {
            device: info.device.clone(),
            sample_rate: info.sample_rate,
            channels: info.channels,
            block_frames: info.block_frames,
            callbacks: self.callbacks.load(Ordering::Relaxed),
            frames,
            seconds: frames as f64 / f64::from(info.sample_rate),
            clicks_queued: self.clicks_queued.load(Ordering::Relaxed),
            clicks_started: self.clicks_started.load(Ordering::Relaxed),
            queue_full: self.queue_full.load(Ordering::Relaxed),
            late_blocks: self.late_blocks.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            active: self.active.load(Ordering::Relaxed),
            jitter_last_us: self.jitter_last_us.load(Ordering::Relaxed),
            jitter_max_us: self.jitter_max_us.load(Ordering::Relaxed),
            jitter_p95_us: self.percentile_us(95),
        }
    }
}

/// Manejador que mantiene vivo el hilo de audio.
pub struct AudioHandle {
    shared: Arc<Shared>,
    info: AudioInfo,
    shutdown: Option<Sender<()>>,
}

impl AudioHandle {
    /// Devuelve la información del dispositivo.
    #[must_use]
    pub fn info(&self) -> &AudioInfo {
        &self.info
    }

    /// Encola un clic para el siguiente bloque. Devuelve `false` si la cola está llena.
    #[must_use]
    pub fn click(&self) -> bool {
        let Ok(mut guard) = self.shared.producer.lock() else {
            return false;
        };
        let Some(producer) = guard.as_mut() else {
            return false;
        };
        match producer.push(()) {
            Ok(()) => {
                self.shared.clicks_queued.fetch_add(1, Ordering::Relaxed);
                true
            }
            Err(_) => {
                self.shared.queue_full.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }

    /// Reloj de audio en milisegundos desde el arranque de la corriente.
    #[must_use]
    pub fn clock_ms(&self) -> f64 {
        let frames = self.shared.clock_frames.load(Ordering::Relaxed);
        frames as f64 / f64::from(self.info.sample_rate) * 1_000.0
    }

    /// Copia de las estadísticas actuales.
    #[must_use]
    pub fn stats(&self) -> AudioStats {
        self.shared.stats(&self.info)
    }
}

impl Drop for AudioHandle {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

/// Error de arranque del audio nativo, ya formateado para la UI.
pub type AudioError = String;

/// Abre el dispositivo por defecto y arranca el hilo de audio.
///
/// # Errors
/// Devuelve un mensaje si no hay dispositivo, si no admite `Fixed(256)` o si el stream
/// no se puede construir.
pub fn start() -> Result<AudioHandle, AudioError> {
    let (init_tx, init_rx) = mpsc::sync_channel::<Result<AudioInfo, AudioError>>(1);
    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();
    let shared = Arc::new(Shared::new());
    let thread_shared = Arc::clone(&shared);

    thread::Builder::new()
        .name("anzan-audio".to_owned())
        .spawn(move || run(thread_shared, shutdown_rx, init_tx))
        .map_err(|error| format!("no se pudo crear el hilo de audio: {error}"))?;

    match init_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(info)) => Ok(AudioHandle {
            shared,
            info,
            shutdown: Some(shutdown_tx),
        }),
        Ok(Err(error)) => Err(error),
        Err(error) => Err(format!("el audio no respondió al arrancar: {error}")),
    }
}

fn run(
    shared: Arc<Shared>,
    shutdown: Receiver<()>,
    init: mpsc::SyncSender<Result<AudioInfo, AudioError>>,
) {
    let built = build_stream(Arc::clone(&shared));
    let (stream, info, producer) = match built {
        Ok(parts) => parts,
        Err(error) => {
            let _ = init.send(Err(error));
            return;
        }
    };

    if let Ok(mut slot) = shared.producer.lock() {
        *slot = Some(producer);
    }
    if let Err(error) = stream.play() {
        let _ = init.send(Err(format!("no se pudo reproducir el stream: {error}")));
        return;
    }
    shared.active.store(true, Ordering::Relaxed);
    let _ = init.send(Ok(info.clone()));

    // El hilo queda bloqueado sosteniendo el `Stream`, que no es `Send` en todas las
    // plataformas. Al soltar el manejador, el canal se cierra y el stream se libera.
    let _ = shutdown.recv();
    shared.active.store(false, Ordering::Relaxed);
    if let Ok(mut slot) = shared.producer.lock() {
        *slot = None;
    }
    drop(stream);
}

type Built = (cpal::Stream, AudioInfo, Producer<()>);

fn build_stream(shared: Arc<Shared>) -> Result<Built, AudioError> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no hay dispositivo de salida de audio".to_owned())?;
    let device_name = device.description().map_or_else(
        |_| "desconocido".to_owned(),
        |description| description.name().to_owned(),
    );
    let supported = device
        .default_output_config()
        .map_err(|error| format!("no se pudo leer la configuración de salida: {error}"))?;

    let sample_rate = supported.sample_rate();
    let channels = supported.channels();
    let format = supported.sample_format();
    let mut config: StreamConfig = supported.config();
    config.buffer_size = BufferSize::Fixed(BLOCK_FRAMES);

    let click = anzan_engine::audio::ClickBuffer::new(sample_rate)
        .map_err(|error| format!("no se pudo sintetizar el clic: {error}"))?;
    let click: Arc<[f32]> = Arc::from(click.samples());

    let (producer, consumer) = RingBuffer::<()>::new(CLICK_QUEUE);
    let info = AudioInfo {
        device: device_name,
        sample_rate,
        channels,
        block_frames: BLOCK_FRAMES,
    };

    let stream = match format {
        SampleFormat::F32 => build::<f32>(
            &device,
            config,
            Arc::clone(&shared),
            Arc::clone(&click),
            consumer,
        ),
        SampleFormat::I16 => build::<i16>(&device, config, Arc::clone(&shared), click, consumer),
        SampleFormat::U16 => build::<u16>(&device, config, Arc::clone(&shared), click, consumer),
        other => {
            return Err(format!(
                "formato de muestra no soportado por el host: {other:?}"
            ));
        }
    }
    .map_err(|error| format!("no se pudo construir el stream: {error}"))?;

    Ok((stream, info, producer))
}

fn build<T>(
    device: &cpal::Device,
    config: StreamConfig,
    shared: Arc<Shared>,
    click: Arc<[f32]>,
    mut consumer: rtrb::Consumer<()>,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample + FromSample<f32> + 'static,
{
    let channels = usize::from(config.channels);
    let sample_rate = config.sample_rate;
    let mut voice: Option<usize> = None;
    let mut previous: Option<cpal::StreamInstant> = None;

    let error_shared = Arc::clone(&shared);
    device.build_output_stream::<T, _, _>(
        config,
        move |data: &mut [T], info: &OutputCallbackInfo| {
            let frames = data.len() / channels.max(1);
            shared.callbacks.fetch_add(1, Ordering::Relaxed);

            let playback = info.timestamp().playback;
            if let Some(previous) = previous {
                let delta = playback.saturating_duration_since(previous);
                let expected = Duration::from_nanos(
                    (frames as u64).saturating_mul(1_000_000_000) / u64::from(sample_rate),
                );
                let observed = delta.as_nanos();
                let expected = expected.as_nanos();
                let jitter = observed.abs_diff(expected);
                shared.record_jitter(u64::try_from(jitter).unwrap_or(u64::MAX));
                if observed > expected + expected / 2 {
                    shared.late_blocks.fetch_add(1, Ordering::Relaxed);
                }
            }
            previous = Some(playback);

            if voice.is_none() && consumer.pop().is_ok() {
                voice = Some(0);
                shared.clicks_started.fetch_add(1, Ordering::Relaxed);
            }

            for frame in data.chunks_mut(channels) {
                let value = match voice {
                    Some(position) => {
                        let sample = click.get(position).copied().unwrap_or(0.0);
                        voice = Some(position + 1);
                        sample
                    }
                    None => 0.0,
                };
                let sample = T::from_sample(value);
                for slot in frame.iter_mut() {
                    *slot = sample;
                }
            }
            if let Some(position) = voice
                && position >= click.len()
            {
                voice = None;
            }

            shared
                .clock_frames
                .fetch_add(frames as u64, Ordering::Relaxed);
        },
        move |_error| {
            error_shared.errors.fetch_add(1, Ordering::Relaxed);
        },
        None,
    )
}

/// Prueba de humo del audio nativo: abre el dispositivo, encola clics espaciados y
/// devuelve las estadísticas del callback. No abre ventana ni depende de la UI.
///
/// # Errors
/// Devuelve el error de arranque del dispositivo.
pub fn selftest(clicks: u32) -> Result<AudioStats, AudioError> {
    let handle = start()?;
    for _ in 0..clicks {
        let _ = handle.click();
        thread::sleep(Duration::from_millis(90));
    }
    thread::sleep(Duration::from_millis(120));
    Ok(handle.stats())
}
