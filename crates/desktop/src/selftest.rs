// SPDX-License-Identifier: AGPL-3.0-or-later
//! Automatización de la app real para medir la ventana completa.
//!
//! Sólo se ejecuta con `--ui-selftest`. El script vive en el host, no en la UI: busca los
//! controles por sus clases, arranca un drill, responde y devuelve un JSON con la
//! telemetría visible y las estadísticas nativas.

/// Script que se evalúa dentro del webview.
pub const SCRIPT: &str = r#"
(async () => {
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const invoke = window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke;
  const out = {
    tauri: !!invoke,
    ua: navigator.userAgent,
    viewport: { w: innerWidth, h: innerHeight, dpr: devicePixelRatio },
  };
  const finish = async () => {
    try { await invoke('selftest_report', { json: JSON.stringify(out) }); } catch (e) {}
  };
  if (!invoke) {
    out.error = 'sin API de Tauri';
    await finish();
    return;
  }
  const t0 = performance.now();
  try {
    let start = null;
    for (let i = 0; i < 400 && !start; i++) {
      await sleep(25);
      start = document.querySelector('.anzan-start');
    }
    if (!start) throw new Error('la UI no cargó el botón de inicio');
    out.audioInfo = await invoke('audio_start');
    start.click();
    const launched = performance.now();
    let form = null;
    for (let i = 0; i < 600; i++) {
      await sleep(25);
      form = document.querySelector('.anzan-answer');
      if (form && !form.hidden) break;
    }
    if (!form || form.hidden) throw new Error('el drill no terminó a tiempo');
    out.stimulusMs = Math.round(performance.now() - launched);
    const input = form.querySelector('input');
    input.value = '0';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    await sleep(20);
    const submit = form.querySelector('.anzan-submit');
    submit.click();
    for (let i = 0; i < 200; i++) {
      await sleep(25);
      const result = document.querySelector('.anzan-drill-result');
      if (result && !result.hidden) break;
    }
    await sleep(400);
    const telemetry = document.querySelector('.anzan-drill-telemetry output');
    const native = document.querySelector('.anzan-native-audio');
    out.telemetry = telemetry ? telemetry.textContent.trim() : '';
    out.native = native ? native.textContent.trim() : '';
    out.nativeVisible = !!native && !native.hidden;
    out.audioStats = await invoke('audio_stats');
    out.sessionRows = document.querySelectorAll('.anzan-session-list li').length;
    const result = document.querySelector('.anzan-drill-result strong');
    out.result = result ? result.textContent.trim() : '';
    out.ok = true;
  } catch (error) {
    out.error = String(error);
  }
  out.totalMs = Math.round(performance.now() - t0);
  await finish();
})();
"#;
