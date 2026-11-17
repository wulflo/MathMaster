# MathMaster

**English** · [Español](README.es.md)

Progressive mental-math trainer with a **virtual soroban**, inspired by *Flash Anzan*.
Desktop first, also available as a static, offline web app.

**▶ Web app: <https://wulflo.github.io/MathMaster/>**

- **100 % Rust** (Leptos/wasm UI + Tauri host + native core/engine).
- **Offline by design**: the web app uses a service worker; the desktop app needs no network.
- **Timing precision** as priority 1: frame-based plans, real telemetry, no sugar-coating.
- **Accessibility and photosensitivity safety**: light/dark themes, reduced contrast and
  motion, no full-screen flashes.
- **Spanish and English**, with no hard-coded strings.

## Status

**Phase 4 complete: desktop host with native audio; web build published on GitHub Pages.**

- Soroban core with property tests, fading and hit-testing independent of opacity.
- `engine::schedule` plans stimuli in frames and measures frame deltas and onset error with
  fixed-size windows, without allocations in the loop.
- `engine::drill` generates deterministic Flash Anzan sequences (digits, terms, seed) and
  `engine::audio` synthesizes clicks with an envelope before playback.
- `engine::progress` defines 6 levels with promotion after two correct answers and demotion
  after two mistakes; `engine::session` keeps the last 100 sessions and exports CSV;
  `engine::pedagogy` classifies sums by case and `engine::review` schedules spaced review.
- The Leptos UI plays the plan with `requestAnimationFrame`, captures the answer and shows
  telemetry, progression, review, per-case statistics and history; settings and progress are
  persisted, with es/en texts.
- **Desktop (Tauri 2.12.5)**: embeds the same wasm build, adds native audio through cpal
  (`Fixed(256)`, 48 kHz), a sample clock with jitter telemetry and atomic `store.json`
  persistence; AppImage and `.deb` bundles are generated locally.
- **Web**: published on GitHub Pages with service worker `anzanlab-v8`.
- Android and haptics are **deferred** by decision ADR-007.

Numbers and limits live in [`docs/MEDICIONES.md`](docs/MEDICIONES.md) and
[`docs/INFORME-FASE-4.md`](docs/INFORME-FASE-4.md); packaging details in
[`docs/EMPAQUETADO.md`](docs/EMPAQUETADO.md); decisions in
[`docs/DECISIONES.md`](docs/DECISIONES.md).

## Documentation

- [`docs/SUPERPROMPT.md`](docs/SUPERPROMPT.md) — current instructions (read-only).
- [`docs/CHECKLIST.md`](docs/CHECKLIST.md) — single source of truth and resume point.
- [`docs/LOG.md`](docs/LOG.md) — append-only log.
- [`docs/DECISIONES.md`](docs/DECISIONES.md) — ADRs.
- [`docs/MEDICIONES.md`](docs/MEDICIONES.md) — real measurements.
- [`docs/PLAN-FASE-5.md`](docs/PLAN-FASE-5.md) — next phase plan (accessibility, mobile web,
  performance).
- [`spikes/README.md`](spikes/README.md) — how to reproduce harnesses A/B/C.

## Cross-cutting requirements

- **GitHub Pages publishing** (static site + offline PWA).
- **Material Aero**: Material 3 tokens + translucent Aero surfaces, light/dark/auto themes
  and accessibility fallbacks.
- **es/en i18n** with validated RON dictionaries and key parity.
- **Copyleft**: code under AGPL-3.0-or-later; content under CC BY-SA 4.0; original sounds and
  dictionaries under CC0.

## License

- Code: [AGPL-3.0-or-later](LICENSE).
- Educational content: [CC BY-SA 4.0](LICENSE-CONTENT).
- Original sounds and dictionaries: CC0 1.0 (see [`CREDITS.md`](CREDITS.md)).

Source code: <https://github.com/wulflo/MathMaster>.
