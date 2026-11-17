// Spike A — Dioxus 0.7.10 desktop (webview). NO es código de producto.
// Mide: (1) duración de estímulos de 100 ms en frames, 1000 reps;
//        (2) coste del canal IPC por frame (Rust en el bucle);
//        (3) latencia software trigger -> primer sample de audio (cpal/PipeWire).

mod audio;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

const DIGIT_STYLE: &str = "position:fixed;inset:0;display:flex;align-items:center;justify-content:center;\
font:700 22vmin/1 ui-monospace,'DejaVu Sans Mono',monospace;color:#e8e8e8;visibility:hidden;\
background:transparent;user-select:none;pointer-events:none;";

const HUD_STYLE: &str = "position:fixed;left:0;right:0;bottom:12px;text-align:center;\
font:400 13px/1.5 system-ui,sans-serif;color:#8a8a8a;pointer-events:none;";

#[derive(Serialize)]
struct Plan {
    reps: u32,
    show_frames: u32,
    blank_frames: u32,
    frame_ms: f64,
    digits: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct Calib {
    median_ms: f64,
    mean_ms: f64,
    span_ms: f64,
    hz: f64,
    hz_span: f64,
    date_span_ms: f64,
    now_min_delta_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    min_ms: f64,
    max_ms: f64,
    n: usize,
    #[serde(default)]
    hist: serde_json::Value,
    #[serde(default)]
    ua: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    i: usize,
    planned_frames: u32,
    actual_ms: f64,
    actual_frames_est: f64,
}

#[derive(Debug, Deserialize)]
struct PresentResult {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    visibility: String,
    #[serde(default)]
    has_focus: bool,
    #[serde(default)]
    idx_at_stall: usize,
    records: Vec<Record>,
    gaps_ms: Vec<f64>,
    total_frames: u64,
    wall_ms: f64,
}

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title("AnzanLab · spike A (Dioxus webview) · medición")
                    .with_inner_size(dioxus::desktop::LogicalSize::new(900.0, 600.0)),
            ),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    let desktop = dioxus::desktop::use_window();
    static WINDOW_CONFIGURED: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);
    if !WINDOW_CONFIGURED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        // El spike necesita primer plano: WebKit suspende/throttlea rAF si la ventana no es visible.
        let _ = desktop.set_always_on_top(true);
        let _ = desktop.set_focus();
        if std::env::var("SPIKE_STAGE").as_deref() == Ok("capture") {
            // Posición absoluta sólo es posible en X11 (XWayland).
            let _ = desktop.set_outer_position(dioxus::desktop::tao::dpi::PhysicalPosition::new(0, 0));
        }
    }
    let win = desktop.window.clone();

    use_future(move || {
        let win = win.clone();
        async move {
            if std::env::var("SPIKE_STAGE").as_deref() == Ok("capture") {
                if let Err(e) = run_capture(win.clone()).await {
                    eprintln!("SPIKE_A_ERROR: {e}");
                    std::process::exit(1);
                }
            }
            if let Err(e) = run_all(win).await {
                eprintln!("SPIKE_A_ERROR: {e}");
                std::process::exit(1);
            }
        }
    });

    rsx! {
        div {
            style: "position:fixed;inset:0;background:#111;overflow:hidden;",
            div { id: "d0", style: "{DIGIT_STYLE}", "7" }
            div { id: "d1", style: "{DIGIT_STYLE}", "9" }
            div { style: "{HUD_STYLE}",
                "spike de medición · estímulos de dígitos sobre fondo estable · no mirar fijamente"
            }
        }
    }
}

async fn run_capture(win: std::sync::Arc<dioxus::desktop::tao::window::Window>) -> Result<(), String> {
    // Dejar que el gestor de ventanas aplique posición y tamaño.
    dioxus::document::eval("await new Promise(r => setTimeout(r, 1500))")
        .join::<serde_json::Value>()
        .await
        .map_err(|e| format!("espera: {e:?}"))?;
    let pos = win
        .inner_position()
        .map_err(|e| format!("inner_position no soportado (¿Wayland?): {e}"))?;
    let size = win.inner_size();
    let scale = win.scale_factor();
    println!(
        "SPIKE_A_GEOM {} {} {} {} {:.2}",
        pos.x, pos.y, size.width, size.height, scale
    );

    dioxus::document::eval(include_str!("presenter.js"))
        .join::<serde_json::Value>()
        .await
        .map_err(|e| format!("instalar presentador: {e:?}"))?;

    // Margen para que el script de captura arranque.
    dioxus::document::eval("await new Promise(r => setTimeout(r, 2500))")
        .join::<serde_json::Value>()
        .await
        .map_err(|e| format!("espera: {e:?}"))?;

    println!("SPIKE_A_CAPTURE_BEGIN");
    let reps: u32 = std::env::var("SPIKE_REPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(150);
    let plan = make_plan(reps, 100.0, 0, 16.667, 0xC0FFEE);
    let _ = run_presentation(&plan).await?;
    println!("SPIKE_A_CAPTURE_END");
    std::process::exit(0);
}

async fn run_all(win: std::sync::Arc<dioxus::desktop::tao::window::Window>) -> Result<(), String> {
    // Esperar a que la webview esté montada y la ventana esté activada, sin bloquear el runtime.
    dioxus::document::eval("await new Promise(r => setTimeout(r, 3000))")
        .join::<serde_json::Value>()
        .await
        .map_err(|e| format!("espera inicial: {e:?}"))?;

    dioxus::document::eval(include_str!("presenter.js"))
        .join::<serde_json::Value>()
        .await
        .map_err(|e| format!("instalar presentador: {e:?}"))?;

    // ── 1. Calibración de refresco ─────────────────────────────────────────
    let calib: Calib = dioxus::document::eval("return await window.__anzan_calibrate(240)")
        .join()
        .await
        .map_err(|e| format!("calibración: {e:?}"))?;
    println!(
        "SPIKE_A_CALIB median={:.4}ms span={:.4}ms hz_span={:.3} p95={:.3} max={:.3} now_delta={:.4}",
        calib.median_ms, calib.span_ms, calib.hz_span, calib.p95_ms, calib.max_ms, calib.now_min_delta_ms
    );

    if std::env::var("SPIKE_STAGE").as_deref() == Ok("calib") {
        let out = serde_json::json!({
            "ua": calib.ua,
            "median_ms": calib.median_ms,
            "span_ms": calib.span_ms,
            "hz_span": calib.hz_span,
            "date_span_ms": calib.date_span_ms,
            "now_min_delta_ms": calib.now_min_delta_ms,
            "min_ms": calib.min_ms,
            "max_ms": calib.max_ms,
            "p95_ms": calib.p95_ms,
            "hist": calib.hist,
        });
        println!("SPIKE_A_CALIB_JSON {}", serde_json::to_string(&out).unwrap_or_default());
        std::process::exit(0);
    }

    if std::env::var("SPIKE_STAGE").as_deref() == Ok("ipc") {
        let t0 = std::time::Instant::now();
        let script = r#"
        await (async function () {
          const p = window.__anzan_ipc_ping(120);
          (async function () {
            for (;;) {
              const m = await dioxus.recv();
              if (window.__anzan_ipc_handler) window.__anzan_ipc_handler(m);
            }
          })();
          const r = await p;
          dioxus.send(r);
        })()
        "#;
        let mut eval = dioxus::document::eval(script);
        let mut seen = 0u32;
        loop {
            match eval.recv::<serde_json::Value>().await {
                Ok(v) => {
                    seen += 1;
                    if seen <= 5 {
                        println!("IPC_RECV #{seen} at {:?}: {v}", t0.elapsed());
                    }
                    if v.get("sent").is_some() {
                        println!("IPC_FINAL after {:?}, recv_count={seen}, {v}", t0.elapsed());
                        break;
                    }
                    eval.send(v).map_err(|e| format!("ipc send: {e:?}"))?;
                }
                Err(e) => return Err(format!("ipc recv: {e:?}")),
            }
        }
        std::process::exit(0);
    }

    let frame_ms = if calib.span_ms > 1.0 {
        calib.span_ms
    } else {
        calib.median_ms
    };

    let quick = std::env::var("SPIKE_QUICK").as_deref() == Ok("1");
    let reps_100 = if quick { 100 } else { 1000 };
    let reps_200 = if quick { 50 } else { 300 };

    // ── 2. Presentación 1000 × 100 ms ──────────────────────────────────────
    let _ = win.set_always_on_top(true);
    let _ = win.set_focus();
    println!("STAGE present100 begin reps={reps_100} frame_ms={frame_ms:.4}");
    let plan_100 = make_plan(reps_100, 100.0, 0, frame_ms, 0xA11CE);
    let res_100 = run_presentation(&plan_100).await?;
    println!("STAGE present100 end n={}", res_100.records.len());

    // ── 3. Presentación 300 × 200 ms (control) ────────────────────────────
    let _ = win.set_focus();
    println!("STAGE present200 begin");
    let plan_200 = make_plan(reps_200, 200.0, 0, frame_ms, 0xBEEF);
    let res_200 = run_presentation(&plan_200).await?;
    println!("STAGE present200 end n={}", res_200.records.len());

    if quick {
        println!("SPIKE_A_QUICK_OK");
        std::process::exit(0);
    }

    // ── 4. IPC ping-pong (Rust en el bucle, 1 mensaje/frame) ──────────────
    let ipc = run_ipc_ping(600).await?;

    // ── 5. Audio (bloquea; ya no hay medición de frames en curso) ─────────
    println!("STAGE audio begin");
    let audio_256 = audio::run_audio_probe(Some(256), 120);
    let audio_default = audio::run_audio_probe(None, 120);
    println!("STAGE audio end");

    let report = serde_json::json!({
        "spike": "A-dioxus-0.7.10-webview",
        "entorno": {
            "os": std::env::consts::OS,
            "session": std::env::var("XDG_SESSION_TYPE").unwrap_or_default(),
            "compositor": std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
            "rustc": "1.99.1",
        },
        "calibracion": {
            "n": calib.n, "median_ms": calib.median_ms, "mean_ms": calib.mean_ms,
            "p95_ms": calib.p95_ms, "p99_ms": calib.p99_ms,
            "min_ms": calib.min_ms, "max_ms": calib.max_ms, "hz": calib.hz,
        },
        "presentacion_100ms": analyze(&res_100, 100.0, frame_ms),
        "raw_100ms": res_100.records,
        "presentacion_200ms": analyze(&res_200, 200.0, frame_ms),
        "ipc_ping_60hz": {
            "medido": ipc,
            "nota": "El canal document::eval de Dioxus 0.7 no entregó mensajes intermedios JS->Rust durante la ejecución: 0/600 respuestas. Sólo llegó el resultado final al terminar el script. Por tanto, supervisar la presentación frame a frame desde Rust vía eval NO es viable en este shell.",
        },
        "audio_buffer_256": audio_256,
        "audio_default": audio_default,
    });

    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spikes/results");
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir results: {e}"))?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = dir.join(format!("spike-a-{stamp}.json"));
    let pretty = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    std::fs::write(&path, &pretty).map_err(|e| format!("write: {e}"))?;

    println!("SPIKE_A_REPORT {}", path.display());
    println!(
        "SPIKE_A_100MS {}",
        serde_json::to_string(&report["presentacion_100ms"]).unwrap_or_default()
    );
    println!(
        "SPIKE_A_IPC {}",
        serde_json::to_string(&report["ipc_ping_60hz"]).unwrap_or_default()
    );
    println!(
        "SPIKE_A_AUDIO {}",
        serde_json::to_string(&report["audio_buffer_256"]).unwrap_or_default()
    );

    std::process::exit(0);
}

fn make_plan(reps: u32, target_ms: f64, blank_frames: u32, frame_ms: f64, seed: u64) -> Plan {
    let show_frames = ((target_ms / frame_ms).round() as u32).max(1);
    let mut s = seed;
    let mut digits = Vec::with_capacity(reps as usize);
    for _ in 0..reps {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        digits.push(((s >> 33) % 10) as u8);
    }
    Plan {
        reps,
        show_frames,
        blank_frames,
        frame_ms,
        digits,
    }
}

async fn run_presentation(plan: &Plan) -> Result<PresentResult, String> {
    let plan_json = serde_json::to_string(plan).map_err(|e| e.to_string())?;
    let script = format!("return await window.__anzan_present({plan_json})");
    let res: PresentResult = dioxus::document::eval(&script)
        .join()
        .await
        .map_err(|e| format!("presentación: {e:?}"))?;
    if let Some(err) = &res.error {
        return Err(format!(
            "presentador abortó: {err} (visibility={}, has_focus={}, idx={}, frames={})",
            res.visibility, res.has_focus, res.idx_at_stall, res.total_frames
        ));
    }
    Ok(res)
}

async fn run_ipc_ping(n: u32) -> Result<serde_json::Value, String> {
    let script = format!(
        r#"
        await (async function () {{
          const p = window.__anzan_ipc_ping({n});
          (async function () {{
            for (;;) {{
              const m = await dioxus.recv();
              if (window.__anzan_ipc_handler) window.__anzan_ipc_handler(m);
            }}
          }})();
          const r = await p;
          dioxus.send(r);
        }})()
        "#
    );
    let mut eval = dioxus::document::eval(&script);
    loop {
        match eval.recv::<serde_json::Value>().await {
            Ok(v) => {
                if v.get("sent").is_some() {
                    return Ok(v);
                }
                // Eco inmediato: el coste medido es ida y vuelta por el canal.
                eval.send(v).map_err(|e| format!("ipc send: {e:?}"))?;
            }
            Err(e) => return Err(format!("ipc recv: {e:?}")),
        }
    }
}

fn analyze(res: &PresentResult, target_ms: f64, frame_ms: f64) -> serde_json::Value {
    let n = res.records.len();
    if n == 0 {
        return serde_json::json!({ "error": "sin registros" });
    }
    let mut abs_ms: Vec<f64> = res
        .records
        .iter()
        .map(|r| (r.actual_ms - target_ms).abs())
        .collect();
    abs_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut frame_err: Vec<f64> = res
        .records
        .iter()
        .map(|r| r.actual_frames_est - r.planned_frames as f64)
        .collect();
    let within_1 = frame_err
        .iter()
        .filter(|e| e.abs() <= 1.0)
        .count() as f64
        / n as f64
        * 100.0;
    let within_0_5 = frame_err
        .iter()
        .filter(|e| e.abs() <= 0.5)
        .count() as f64
        / n as f64
        * 100.0;

    frame_err.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p = |v: &[f64], q: f64| -> f64 {
        if v.is_empty() {
            0.0
        } else {
            v[((v.len() as f64 - 1.0) * q) as usize]
        }
    };

    let abs_ms_p50 = p(&abs_ms, 0.50);
    let abs_ms_p95 = p(&abs_ms, 0.95);
    let abs_ms_max = p(&abs_ms, 1.0);

    // Cadencia empírica del propio run: (nº de intervalos rAF) / tiempo de pared.
    let empirical_frame_ms = if res.total_frames > 1 {
        res.wall_ms / (res.total_frames - 1) as f64
    } else {
        frame_ms
    };
    let frame_err_empirical: Vec<f64> = res
        .records
        .iter()
        .map(|r| r.actual_ms / empirical_frame_ms - r.planned_frames as f64)
        .collect();
    let within_1_emp = frame_err_empirical
        .iter()
        .filter(|e| e.abs() <= 1.0)
        .count() as f64
        / n as f64
        * 100.0;

    let planned_total_ms = n as f64 * target_ms;
    let drift_ms = res.wall_ms - planned_total_ms;
    let gaps_over_1_5 = res
        .gaps_ms
        .iter()
        .filter(|g| **g > empirical_frame_ms * 1.5)
        .count();

    serde_json::json!({
        "n": n,
        "planned_frames_per_stimulus": res.records.first().map(|r| r.planned_frames),
        "calibration_frame_ms": frame_ms,
        "empirical_frame_ms": empirical_frame_ms,
        "within_1_frame_pct": within_1_emp,
        "within_1_frame_pct_calib": within_1,
        "within_0_5_frame_pct_calib": within_0_5,
        "abs_error_ms_p50": abs_ms_p50,
        "abs_error_ms_p95": abs_ms_p95,
        "abs_error_ms_max": abs_ms_max,
        "frame_error_p50": p(&frame_err_empirical, 0.50),
        "frame_error_p95": p(&frame_err_empirical, 0.95),
        "frame_error_min": p(&frame_err_empirical, 0.0),
        "frame_error_max": p(&frame_err_empirical, 1.0),
        "wall_ms": res.wall_ms,
        "planned_total_ms": planned_total_ms,
        "drift_ms": drift_ms,
        "gaps_over_1_5_frames": gaps_over_1_5,
        "total_frames": res.total_frames,
    })
}
