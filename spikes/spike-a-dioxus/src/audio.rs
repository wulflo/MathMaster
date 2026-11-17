// Sonda de audio del spike. No es código de producto.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleFormat, StreamConfig};
use rtrb::{Producer, RingBuffer};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct Click {
    pub samples: Vec<f32>, // intercalado estéreo
    pub frames: usize,
}

fn synth_click(sr: u32) -> Click {
    let n = (sr as f32 * 0.05) as usize;
    let mut mono = vec![0.0f32; n];
    let mut state: u32 = 0x9E37_79B9;
    for (i, s) in mono.iter_mut().enumerate() {
        let t = i as f32 / sr as f32;
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let noise = ((state >> 9) as f32 / (1u32 << 23) as f32) * 2.0 - 1.0;
        let burst = noise * (-t / 0.0015).exp();
        let r1 = (std::f32::consts::TAU * 1750.0 * t).sin() * (-t / 0.012).exp();
        let r2 = (std::f32::consts::TAU * 2650.0 * t).sin() * (-t / 0.007).exp();
        let r3 = (std::f32::consts::TAU * 4300.0 * t).sin() * (-t / 0.004).exp();
        *s = 0.35 * burst + 0.25 * r1 + 0.18 * r2 + 0.10 * r3;
    }
    let peak = mono.iter().fold(0.0f32, |m, v| m.max(v.abs())).max(1e-6);
    let g = 0.8 / peak;
    let mut samples = Vec::with_capacity(n * 2);
    for s in &mono {
        samples.push(s * g);
        samples.push(s * g);
    }
    Click {
        samples,
        frames: n,
    }
}

pub struct AudioProbe {
    pub host: String,
    pub device: String,
    pub sample_format: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub requested_buffer: Option<u32>,
    pub callbacks: Arc<AtomicU64>,
    pub late_callbacks: Arc<AtomicU64>,
    pub max_callback_gap_ns: Arc<AtomicU64>,
    pub play_offset_ns: Arc<AtomicU64>,
    pub consumed: Arc<AtomicU64>,
    _stream: cpal::Stream,
    producer: Producer<f32>,
    pushed: u64,
    click: Click,
    sr: u32,
    ch: u16,
}

fn now_ns(base: Instant) -> f64 {
    base.elapsed().as_nanos() as f64
}

impl AudioProbe {
    pub fn start(preferred_buffer: Option<u32>) -> Result<Self, String> {
        let host = cpal::default_host();
        let host_name = format!("{:?}", host.id());
        let device = host
            .default_output_device()
            .ok_or_else(|| "sin dispositivo de salida".to_string())?;
        let dev_name = device
            .description()
            .map(|d| d.name().to_string())
            .unwrap_or_else(|_| "?".into());
        let supported = device
            .default_output_config()
            .map_err(|e| format!("default_output_config: {e}"))?;
        let sample_format = supported.sample_format();
        if sample_format != SampleFormat::F32 {
            return Err(format!("formato no f32: {sample_format:?}"));
        }
        let sr = supported.sample_rate();
        let ch = supported.channels();
        let mut config: StreamConfig = supported.config();
        if let Some(b) = preferred_buffer {
            config.buffer_size = BufferSize::Fixed(b);
        }

        let (producer, mut consumer) = RingBuffer::<f32>::new((sr as usize) * 2);
        let callbacks = Arc::new(AtomicU64::new(0));
        let late = Arc::new(AtomicU64::new(0));
        let max_gap = Arc::new(AtomicU64::new(0));
        let play_offset = Arc::new(AtomicU64::new(0));
        let consumed = Arc::new(AtomicU64::new(0));

        let (c_cb, c_late, c_gap, c_off, c_cons) = (
            callbacks.clone(),
            late.clone(),
            max_gap.clone(),
            play_offset.clone(),
            consumed.clone(),
        );
        let base = Instant::now();
        let mut local_last_ns: f64 = 0.0;
        let mut local_consumed: u64 = 0;
        let expected_cb_ns = (config.buffer_size_frame_count(sr) as f64 / sr as f64) * 1e9;

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                    let t = now_ns(base);
                    c_cb.fetch_add(1, Ordering::Relaxed);
                    if local_last_ns > 0.0 {
                        let gap = t - local_last_ns;
                        if gap as u64 > c_gap.load(Ordering::Relaxed) {
                            c_gap.store(gap as u64, Ordering::Relaxed);
                        }
                        if gap > expected_cb_ns * 2.0 {
                            c_late.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    local_last_ns = t;

                    let ts = info.timestamp();
                    c_off.store(
                        ts.playback.duration_since(ts.callback).as_nanos() as u64,
                        Ordering::Relaxed,
                    );

                    for sample in data.iter_mut() {
                        match consumer.pop() {
                            Ok(v) => {
                                *sample = v;
                                local_consumed += 1;
                            }
                            Err(_) => *sample = 0.0,
                        }
                    }
                    c_cons.store(local_consumed, Ordering::Release);
                },
                move |err| eprintln!("[audio] error de stream: {err}"),
                None,
            )
            .map_err(|e| format!("build_output_stream: {e}"))?;
        stream.play().map_err(|e| format!("play: {e}"))?;

        Ok(Self {
            host: host_name,
            device: dev_name,
            sample_format: format!("{sample_format:?}"),
            sample_rate: sr,
            channels: ch,
            requested_buffer: preferred_buffer,
            callbacks,
            late_callbacks: late,
            max_callback_gap_ns: max_gap,
            play_offset_ns: play_offset,
            consumed,
            _stream: stream,
            producer,
            pushed: 0,
            click: synth_click(sr),
            sr,
            ch,
        })
    }

    /// Dispara un clic y mide trigger -> callback que consume su primer sample.
    pub fn trigger_and_wait(&mut self, timeout: Duration) -> Option<f64> {
        let start = self.pushed + 1;
        let t0 = Instant::now();
        let total = self.click.samples.len();
        if self.producer.slots() < total {
            return None;
        }
        for &s in &self.click.samples {
            if self.producer.push(s).is_err() {
                return None;
            }
        }
        self.pushed += total as u64;
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if self.consumed.load(Ordering::Acquire) >= start {
                return Some(t0.elapsed().as_nanos() as f64 / 1e6);
            }
            std::hint::spin_loop();
        }
        None
    }

    pub fn click_ms(&self) -> f32 {
        self.click.frames as f32 * 1000.0 / self.sr as f32
    }
    pub fn play_offset_ms(&self) -> f32 {
        self.play_offset_ns.load(Ordering::Relaxed) as f32 / 1e6
    }
    pub fn sample_rate(&self) -> u32 {
        self.sr
    }
    pub fn channels(&self) -> u16 {
        self.ch
    }
}

trait BufferFrames {
    fn buffer_size_frame_count(&self, sr: u32) -> u32;
}
impl BufferFrames for StreamConfig {
    fn buffer_size_frame_count(&self, sr: u32) -> u32 {
        match self.buffer_size {
            BufferSize::Fixed(f) => f,
            BufferSize::Default => (sr / 100).max(64), // estimación: 10 ms
        }
    }
}

pub fn run_audio_probe(preferred_buffer: Option<u32>, n_clicks: usize) -> serde_json::Value {
    let mut probe = match AudioProbe::start(preferred_buffer) {
        Ok(p) => p,
        Err(e) => return serde_json::json!({ "error": e }),
    };
    std::thread::sleep(Duration::from_millis(300));
    let mut lat = Vec::with_capacity(n_clicks);
    let mut misses = 0;
    for _ in 0..n_clicks {
        std::thread::sleep(Duration::from_millis(120));
        match probe.trigger_and_wait(Duration::from_millis(200)) {
            Some(ms) => lat.push(ms),
            None => misses += 1,
        }
    }
    lat.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p = |q: f64| -> f64 {
        if lat.is_empty() {
            0.0
        } else {
            lat[((lat.len() as f64 - 1.0) * q) as usize]
        }
    };
    serde_json::json!({
        "host": probe.host,
        "device": probe.device,
        "sample_format": probe.sample_format,
        "sample_rate": probe.sample_rate,
        "channels": probe.channels,
        "requested_buffer_frames": preferred_buffer,
        "click_ms": probe.click_ms(),
        "callbacks": probe.callbacks.load(Ordering::Relaxed),
        "late_callbacks": probe.late_callbacks.load(Ordering::Relaxed),
        "max_callback_gap_ms": probe.max_callback_gap_ns.load(Ordering::Relaxed) as f64 / 1e6,
        "playback_offset_ms": probe.play_offset_ms(),
        "n": lat.len(),
        "misses": misses,
        "trigger_to_first_sample_p50_ms": p(0.5),
        "trigger_to_first_sample_p95_ms": p(0.95),
        "trigger_to_first_sample_max_ms": p(1.0),
        "nota": "latencia software: trigger -> callback que consume el primer sample. No incluye DAC/amplificador/altavoz ni el offset de playback devuelto por el backend.",
    })
}

#[allow(dead_code)]
fn _unused(p: &AudioProbe) -> (u32, u16) {
    (p.sample_rate(), p.channels())
}
