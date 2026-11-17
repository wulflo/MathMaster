// SPDX-License-Identifier: AGPL-3.0-or-later
//! Medición de la app real: mismo arnés y plan que el spike B dentro de la ventana
//! completa de MathMaster.
//!
//! Sólo se ejecuta con `--measure`. Primero completa un drill real (para que haya audio,
//! progreso y persistencia en juego) y después presenta 1000 estímulos de 100 ms y 300 de
//! 200 ms sobre dos capas preasignadas, registrando la duración observada por estímulo.
//! El resultado se entrega por el comando `measure_report`.

/// Script que se evalúa dentro del webview.
pub const SCRIPT: &str = r#"
(async () => {
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const invoke = window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke;
  const out = {
    app: "anzan-desktop",
    tauri: !!invoke,
    ua: navigator.userAgent,
    viewport: { w: innerWidth, h: innerHeight, dpr: devicePixelRatio },
    ok: false,
  };
  const finish = async () => {
    try { await invoke("measure_report", { json: JSON.stringify(out) }); }
    catch (e) { out.reportError = String(e); }
  };
  if (!invoke) {
    out.error = "sin API de Tauri";
    await finish();
    return;
  }
  const percentile = (sorted, p) => sorted.length
    ? sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * p))]
    : 0;
  const round = (value) => Math.round(value * 1000) / 1000;

  try {
    // Espera a que la UI real esté montada.
    let start = null;
    for (let i = 0; i < 400 && !start; i++) {
      await sleep(25);
      start = document.querySelector(".anzan-start");
    }
    if (!start) throw new Error("la UI no cargó el botón de inicio");

    out.hasFocusInicial = document.hasFocus();
    const config = window.__ANZAN_MEASURE_CONFIG__ || {};
    if (!config.noAudio) out.audioInfo = await invoke("audio_start");

    // Drill real: audio nativo, progreso, sesiones y telemetría de la UI.
    start.click();
    const launched = performance.now();
    let form = null;
    for (let i = 0; i < 800; i++) {
      await sleep(25);
      form = document.querySelector(".anzan-answer");
      if (form && !form.hidden) break;
    }
    if (!form || form.hidden) throw new Error("el drill no terminó a tiempo");
    out.drill = { stimulusMs: Math.round(performance.now() - launched) };
    const input = form.querySelector("input");
    input.value = "0";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    await sleep(20);
    form.querySelector(".anzan-submit").click();
    for (let i = 0; i < 200; i++) {
      await sleep(25);
      const result = document.querySelector(".anzan-drill-result");
      if (result && !result.hidden) break;
    }
    await sleep(300);
    const telemetry = document.querySelector(".anzan-drill-telemetry output");
    const native = document.querySelector(".anzan-native-audio");
    const result = document.querySelector(".anzan-drill-result strong");
    out.drill.telemetry = telemetry ? telemetry.textContent.trim() : "";
    out.drill.native = native ? native.textContent.trim() : "";
    out.drill.result = result ? result.textContent.trim() : "";
    out.drill.sessionRows = document.querySelectorAll(".anzan-session-list li").length;

    // WebKit suspende rAF en ventanas sin foco: se espera a que el lanzador active
    // la ventana (xdotool) antes de medir.
    const focusDeadline = performance.now() + 15000;
    while (!document.hasFocus() && performance.now() < focusDeadline) await sleep(100);
    out.hasFocus = document.hasFocus();

    const reps100 = config.reps100 || 1000;
    const reps200 = config.reps200 || 300;

    // Dos capas preasignadas, creadas una sola vez; sólo cambia `visibility`.
    const host = document.createElement("div");
    host.id = "anzan-measure";
    host.style.cssText = "position:fixed;inset:0;pointer-events:none;z-index:2147483647;background:transparent";
    const makeLayer = () => {
      const el = document.createElement("div");
      el.style.cssText = "position:absolute;inset:0;display:flex;align-items:center;justify-content:center;" +
        "font:700 22vmin/1 ui-monospace,'DejaVu Sans Mono',monospace;color:#e8e8e8;" +
        "visibility:hidden;background:transparent;user-select:none;pointer-events:none";
      return el;
    };
    const layerA = makeLayer();
    const layerB = makeLayer();
    host.appendChild(layerA);
    host.appendChild(layerB);
    document.body.appendChild(host);
    if (config.hideUI) {
      for (const child of Array.from(document.body.children)) {
        if (child !== host) child.style.display = "none";
      }
      const style = document.createElement("style");
      style.textContent = "html,body{background:#111 !important}" +
        "*{animation:none !important;transition:none !important;box-shadow:none !important;filter:none !important}";
      document.head.appendChild(style);
    }
    layerA.textContent = "7";
    layerB.textContent = "9";

    const calibrate = (frames) => new Promise((resolve) => {
      const ts = [];
      let count = 0;
      const dateStart = Date.now();
      const step = (t) => {
        if (count > 0) ts.push(t);
        count += 1;
        if (count <= frames) { requestAnimationFrame(step); return; }
        const d = [];
        for (let i = 1; i < ts.length; i++) d.push(ts[i] - ts[i - 1]);
        const sorted = d.slice().sort((x, y) => x - y);
        const median = sorted.length ? sorted[Math.floor(sorted.length / 2)] : 16.6667;
        const span = ts.length > 1 ? (ts[ts.length - 1] - ts[0]) / (ts.length - 1) : median;
        const hist = {};
        for (const value of d) {
          const key = value.toFixed(1);
          hist[key] = (hist[key] || 0) + 1;
        }
        resolve({
          n: d.length,
          median_ms: median,
          span_ms: span,
          mean_ms: d.reduce((x, y) => x + y, 0) / (d.length || 1),
          p95_ms: percentile(sorted, 0.95),
          p99_ms: percentile(sorted, 0.99),
          min_ms: sorted[0] || median,
          max_ms: sorted[sorted.length - 1] || median,
          hz: 1000 / median,
          date_span_ms: Date.now() - dateStart,
          hist,
        });
      };
      requestAnimationFrame(step);
    });

    const present = (reps, showFrames, frameMs, digits) => new Promise((resolve) => {
      let settled = false;
      let lastActivity = performance.now();
      const started = performance.now();
      const watchdog = setInterval(() => {
        if (!settled && performance.now() - lastActivity > 5000) {
          settled = true;
          clearInterval(watchdog);
          resolve({
            error: "raf_stalled",
            records: recs,
            gaps_ms: gaps,
            total_frames: frame,
            wall_ms: performance.now() - started,
            ms_since_last_frame: performance.now() - lastActivity,
            idx,
          });
        }
      }, 1000);

      layerA.textContent = String(digits[0]);
      layerB.textContent = String(digits[1]);
      layerA.style.visibility = "hidden";
      layerB.style.visibility = "hidden";
      const layers = [layerA, layerB];
      const hideAll = () => {
        layers[0].style.visibility = "hidden";
        layers[1].style.visibility = "hidden";
      };
      const done = (wall) => {
        hideAll();
        settled = true;
        clearInterval(watchdog);
        resolve({ records: recs, gaps_ms: gaps, total_frames: frame, wall_ms: wall });
      };

      let frame = 0;
      let idx = 0;
      let visible = 0;
      let showStartFrame = 0;
      let showStartTs = 0;
      let prevTs = 0;
      let firstShowTs = 0;
      const gaps = [];
      const recs = [];

      const step = (t) => {
        frame += 1;
        lastActivity = performance.now();
        if (prevTs && t - prevTs > frameMs * 1.5) gaps.push(t - prevTs);
        prevTs = t;
        if (idx >= reps) { done(t - firstShowTs); return; }
        if (frame === 1) {
          layers[visible].style.visibility = "visible";
          showStartFrame = frame;
          showStartTs = t;
          firstShowTs = t;
          requestAnimationFrame(step);
          return;
        }
        if (frame - showStartFrame >= showFrames) {
          layers[visible].style.visibility = "hidden";
          recs.push({ i: idx, actual_ms: round(t - showStartTs) });
          idx += 1;
          if (idx >= reps) { done(t - firstShowTs); return; }
          visible = 1 - visible;
          layers[visible].textContent = String(digits[idx % digits.length]);
          layers[visible].style.visibility = "visible";
          showStartFrame = frame;
          showStartTs = t;
        }
        requestAnimationFrame(step);
      };
      requestAnimationFrame(step);
    });

    const summarize = (result, reps, targetMs, frameMs) => {
      if (result.error) return result;
      const errors = result.records.map((r) => Math.abs(r.actual_ms - targetMs)).sort((x, y) => x - y);
      const total = result.records.reduce((acc, r) => acc + r.actual_ms, 0);
      const planned = reps * targetMs;
      const wall = result.wall_ms;
      return {
        reps,
        target_ms: targetMs,
        show_frames: Math.max(1, Math.round(targetMs / frameMs)),
        frame_ms_usado: round(frameMs),
        wall_ms: round(wall),
        total_frames: result.total_frames,
        frame_real_ms: round(wall / Math.max(1, result.total_frames - 1)),
        mean_ms: round(total / Math.max(1, result.records.length)),
        drift_ms: round(wall - planned),
        drift_pct: round((wall - planned) / planned * 100),
        err_p50_ms: round(percentile(errors, 0.5)),
        err_p95_ms: round(percentile(errors, 0.95)),
        err_max_ms: round(errors[errors.length - 1] || 0),
        within_one_frame_pct: round(
          result.records.filter((r) => Math.abs(r.actual_ms - targetMs) <= frameMs).length /
            Math.max(1, reps) * 100
        ),
        gaps_count: result.gaps_ms.length,
        gaps_max_ms: round(result.gaps_ms.reduce((max, v) => Math.max(max, v), 0)),
        records: result.records,
      };
    };

    out.calibration = await calibrate(240);
    const frameMs = out.calibration.span_ms > 1 ? out.calibration.span_ms : out.calibration.median_ms;
    out.presentation_100ms = summarize(
      await present(reps100, Math.max(1, Math.round(100 / frameMs)), frameMs, "0123456789".split("")),
      reps100,
      100,
      frameMs
    );
    await sleep(300);
    out.presentation_200ms = summarize(
      await present(reps200, Math.max(1, Math.round(200 / frameMs)), frameMs, "0123456789".split("")),
      reps200,
      200,
      frameMs
    );
    host.remove();
    out.audioStats = await invoke("audio_stats");
    out.ok = true;
  } catch (error) {
    out.error = String(error);
  }
  await finish();
})();
"#;
