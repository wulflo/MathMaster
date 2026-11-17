// Arnés de medición compartido entre spikes (A y B).
// No es código de producto. Objetivo: medir en FRAMES, no en ms.

(function () {
  "use strict";

  function hide(el) {
    if (el) el.style.visibility = "hidden";
  }
  function show(el) {
    if (el) el.style.visibility = "visible";
  }

  // Calibración: mide intervalos entre callbacks rAF.
  window.__anzan_calibrate = function (frames) {
    return new Promise(function (resolve) {
      var ts = [];
      var count = 0;
      var dateStart = Date.now();
      function step(t) {
        if (count > 0) ts.push(t);
        count++;
        if (count <= frames) {
          requestAnimationFrame(step);
        } else {
          var d = [];
          for (var i = 1; i < ts.length; i++) d.push(ts[i] - ts[i - 1]);
          d.sort(function (a, b) { return a - b; });
          var med = d[Math.floor(d.length / 2)] || 16.6667;
          var span = ts.length > 1 ? (ts[ts.length - 1] - ts[0]) / (ts.length - 1) : med;
          // Histograma de intervalos redondeados a 0.1 ms.
          var hist = {};
          for (var k = 0; k < d.length; k++) {
            var key = d[k].toFixed(1);
            hist[key] = (hist[key] || 0) + 1;
          }
          // Resolución efectiva de performance.now().
          var p0 = performance.now();
          var minDelta = Infinity;
          var prev = p0;
          for (var j = 0; j < 20000; j++) {
            var now = performance.now();
            if (now > prev && now - prev < minDelta) minDelta = now - prev;
            prev = now;
          }
          resolve({
            n: d.length,
            median_ms: med,
            span_ms: span,
            mean_ms: d.reduce(function (a, b) { return a + b; }, 0) / (d.length || 1),
            p95_ms: d[Math.floor(d.length * 0.95)] || med,
            p99_ms: d[Math.floor(d.length * 0.99)] || med,
            min_ms: d[0] || med,
            max_ms: d[d.length - 1] || med,
            hz: 1000 / med,
            hz_span: 1000 / span,
            date_span_ms: Date.now() - dateStart,
            hist: hist,
            now_min_delta_ms: minDelta === Infinity ? 0 : minDelta,
            ua: navigator.userAgent,
          });
        }
      }
      requestAnimationFrame(step);
    });
  };

  // Presenta `plan.reps` estímulos alternando DOS capas preasignadas.
  // plan = { reps, show_frames, blank_frames, frame_ms, digits: [..] }
  // Devuelve registros por estímulo; nunca crea nodos ni cambia layout.
  window.__anzan_present = function (plan) {
    return new Promise(function (resolve) {
      var a = document.getElementById("d0");
      var b = document.getElementById("d1");
      if (!a || !b) {
        resolve({ error: "missing #d0/#d1" });
        return;
      }
      var settled = false;
      var lastActivity = performance.now();
      var watchdog = setInterval(function () {
        if (!settled && performance.now() - lastActivity > 3000) {
          settled = true;
          clearInterval(watchdog);
          resolve({
            error: "raf_stalled",
            records: recs,
            gaps_ms: gaps,
            total_frames: frame,
            wall_ms: 0,
            visibility: document.visibilityState,
            has_focus: document.hasFocus(),
            ms_since_last_frame: performance.now() - lastActivity,
            idx_at_stall: idx,
          });
        }
      }, 500);
      // Texto preasignado una sola vez, antes del bucle.
      var digits = plan.digits || [];
      a.textContent = String(digits.length ? digits[0] : 7);
      b.textContent = String(digits.length > 1 ? digits[1] : 9);
      hide(a);
      hide(b);

      var frame = 0;
      var idx = 0;
      var visible = 0; // 0 => capa a, 1 => capa b
      var showStartFrame = 0;
      var showStartTs = 0;
      var awaitingBlank = false;
      var blankStartFrame = 0;
      var prevTs = 0;
      var gaps = []; // intervalos rAF anómalos
      var recs = [];
      var layers = [a, b];
      var firstShowTs = 0;

      function nextLayer() {
        return layers[1 - visible];
      }

      function step(t) {
        frame++;
        lastActivity = performance.now();
        if (prevTs && t - prevTs > plan.frame_ms * 1.5) gaps.push(t - prevTs);
        prevTs = t;

        if (idx >= plan.reps) {
          hide(a);
          hide(b);
          settled = true;
          clearInterval(watchdog);
          resolve({
            records: recs,
            gaps_ms: gaps,
            total_frames: frame,
            wall_ms: firstShowTs ? t - firstShowTs : 0,
          });
          return;
        }

        if (frame === 1) {
          // Mostrar el primer estímulo en el primer frame.
          show(layers[visible]);
          showStartFrame = frame;
          showStartTs = t;
          firstShowTs = t;
          requestAnimationFrame(step);
          return;
        }

        if (!awaitingBlank) {
          var elapsed = frame - showStartFrame;
          if (elapsed >= plan.show_frames) {
            // Ocultar y registrar la duración observada.
            hide(layers[visible]);
            var ms = t - showStartTs;
            recs.push({
              i: idx,
              planned_frames: plan.show_frames,
              actual_ms: ms,
              actual_frames_est: ms / plan.frame_ms,
              at_ts: t,
            });
            idx++;
            if (idx >= plan.reps) {
              hide(a);
              hide(b);
              settled = true;
              clearInterval(watchdog);
              resolve({
                records: recs,
                gaps_ms: gaps,
                total_frames: frame,
                wall_ms: t - firstShowTs,
              });
              return;
            }
            if (plan.blank_frames > 0) {
              awaitingBlank = true;
              blankStartFrame = frame;
            } else {
              // Cambio directo de capa: sin fondo en blanco (no parpadea la pantalla).
              visible = 1 - visible;
              layers[visible].textContent = String(digits[idx] !== undefined ? digits[idx] : idx % 10);
              show(layers[visible]);
              showStartFrame = frame;
              showStartTs = t;
            }
          }
        } else if (frame - blankStartFrame >= plan.blank_frames) {
          awaitingBlank = false;
          visible = 1 - visible;
          layers[visible].textContent = String(digits[idx] !== undefined ? digits[idx] : idx % 10);
          show(layers[visible]);
          showStartFrame = frame;
          showStartTs = t;
        }
        requestAnimationFrame(step);
      }

      requestAnimationFrame(step);
    });
  };

  // Ping-pong con el proceso Rust para medir el coste del canal IPC.
  // Cada frame envía un mensaje y espera respuesta; mide RTT en el reloj JS.
  window.__anzan_ipc_ping = function (n) {
    return new Promise(function (resolve) {
      var rtts = [];
      var sent = 0;
      var got = 0;
      var pending = new Map();
      window.__anzan_ipc_handler = function (msg) {
        var t0 = pending.get(msg.seq);
        if (t0 === undefined) return;
        pending.delete(msg.seq);
        rtts.push(performance.now() - t0);
        got++;
      };
      function tick() {
        if (sent >= n) {
          if (got >= n || sent - got > 60) {
            rtts.sort(function (x, y) { return x - y; });
            resolve({
              sent: sent,
              got: got,
              p50: rtts[Math.floor(rtts.length * 0.5)] || 0,
              p95: rtts[Math.floor(rtts.length * 0.95)] || 0,
              p99: rtts[Math.floor(rtts.length * 0.99)] || 0,
              max: rtts[rtts.length - 1] || 0,
            });
            return;
          }
          setTimeout(tick, 200);
          return;
        }
        var seq = sent++;
        pending.set(seq, performance.now());
        dioxus.send({ seq: seq, ts: performance.now() });
        requestAnimationFrame(tick);
      }
      requestAnimationFrame(tick);
    });
  };

  window.__anzan_ipc_ready = function () {
    return true;
  };
})();

// ───────── Runner específico del spike B (Tauri 2 + Leptos CSR) ─────────
(function () {
  "use strict";
  function makePlan(reps, targetMs, frameMs, seed) {
    var show = Math.max(1, Math.round(targetMs / frameMs));
    var s = seed >>> 0;
    var digits = [];
    for (var i = 0; i < reps; i++) {
      s = (Math.imul(s, 1664525) + 1013904223) >>> 0;
      digits.push(s % 10);
    }
    return { reps: reps, show_frames: show, blank_frames: 0, frame_ms: frameMs, digits: digits };
  }

  function ipcInvoke(n) {
    return new Promise(function (resolve) {
      var rtts = [];
      var i = 0;
      var t0 = performance.now();
      function next() {
        if (i >= n) {
          rtts.sort(function (a, b) { return a - b; });
          resolve({
            n: rtts.length,
            total_ms: performance.now() - t0,
            p50: rtts[Math.floor(rtts.length * 0.5)] || 0,
            p95: rtts[Math.floor(rtts.length * 0.95)] || 0,
            max: rtts[rtts.length - 1] || 0,
          });
          return;
        }
        var a = performance.now();
        window.__TAURI__.core.invoke("ping", { seq: i }).then(function () {
          rtts.push(performance.now() - a);
          i++;
          requestAnimationFrame(next);
        });
      }
      requestAnimationFrame(next);
    });
  }

  window.addEventListener("load", function () {
    setTimeout(async function () {
      try {
        var calib = await window.__anzan_calibrate(240);
        var frameMs = calib.span_ms > 1 ? calib.span_ms : calib.median_ms;
        var plan100 = makePlan(1000, 100, frameMs, 0xa11ce);
        var pres100 = await window.__anzan_present(plan100);
        var plan200 = makePlan(300, 200, frameMs, 0xbeef);
        var pres200 = await window.__anzan_present(plan200);
        var ipc = await ipcInvoke(600);
        await window.__TAURI__.core.invoke("report", {
          json: JSON.stringify({
            spike: "B-tauri-2.12.5-leptos-0.8.23-csr",
            calibracion: calib,
            presentacion_100ms: pres100,
            presentacion_200ms: pres200,
            ipc_ping_60hz: ipc,
            ua: navigator.userAgent,
          }),
        });
      } catch (e) {
        await window.__TAURI__.core.invoke("fail", { msg: String(e) });
      }
    }, 3000);
  });
})();
