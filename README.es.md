# MathMaster

[English](README.md) · **Español**

Entrenador progresivo de cálculo mental con **soroban virtual**, inspirado en el
*Flash Anzan*. Escritorio primero; también disponible como web estática y offline.

**▶ Web: <https://wulflo.github.io/MathMaster/>**

- **100 % Rust** (UI Leptos/wasm + host Tauri + core/engine nativos).
- **Offline** por diseño: la web usa service worker; el escritorio no necesita red.
- **Precisión temporal** como prioridad 1: planes en frames, telemetría real, sin maquillar.
- **Accesibilidad y seguridad fotosensible**: claro/oscuro, contraste y movimiento reducidos,
  sin destellos de pantalla completa.
- **Español e inglés**, sin strings hardcodeados.

## Estado

**Fase 4 completa: host de escritorio con audio nativo; build web publicada en GitHub Pages.**

- Core del soroban con property tests, fading y geometría de impacto independiente de la
  opacidad.
- `engine::schedule` planifica estímulos en frames y mide deltas de frame y error de
  aparición con ventanas de tamaño fijo, sin asignaciones en el bucle.
- `engine::drill` genera secuencias Flash Anzan deterministas (dígitos, términos, semilla) y
  `engine::audio` sintetiza clics con envolvente antes de reproducir.
- `engine::progress` define 6 niveles con promoción tras dos aciertos y descenso tras dos
  fallos; `engine::session` guarda las últimas 100 sesiones y exporta CSV;
  `engine::pedagogy` clasifica las sumas por caso y `engine::review` programa repasos.
- La UI Leptos reproduce el plan con `requestAnimationFrame`, captura la respuesta y muestra
  telemetría, progresión, repaso, estadísticas por caso e historial; ajustes y progreso
  persistidos, textos es/en.
- **Escritorio (Tauri 2.12.5)**: embebe el mismo build wasm, añade audio nativo con cpal
  (`Fixed(256)`, 48 kHz), reloj de muestras con telemetría de jitter y persistencia atómica
  en `store.json`; los bundles AppImage y `.deb` se generan localmente.
- **Web**: publicada en GitHub Pages con service worker `anzanlab-v8`.
- Android y háptico quedan **diferidos** por la decisión ADR-007.

Números y límites en [`docs/MEDICIONES.md`](docs/MEDICIONES.md) y
[`docs/INFORME-FASE-4.md`](docs/INFORME-FASE-4.md); empaquetado en
[`docs/EMPAQUETADO.md`](docs/EMPAQUETADO.md); decisiones en
[`docs/DECISIONES.md`](docs/DECISIONES.md).

## Documentación

- [`docs/SUPERPROMPT.md`](docs/SUPERPROMPT.md) — instrucciones vigentes (solo lectura).
- [`docs/CHECKLIST.md`](docs/CHECKLIST.md) — fuente única de verdad y punto de reanudación.
- [`docs/LOG.md`](docs/LOG.md) — bitácora append-only.
- [`docs/DECISIONES.md`](docs/DECISIONES.md) — ADRs.
- [`docs/MEDICIONES.md`](docs/MEDICIONES.md) — mediciones reales.
- [`docs/PLAN-FASE-5.md`](docs/PLAN-FASE-5.md) — plan de la próxima fase (accesibilidad, web
  móvil y rendimiento).
- [`spikes/README.md`](spikes/README.md) — cómo reproducir los arneses A/B/C.

## Requisitos transversales

- **Publicación en GitHub Pages** (sitio estático + PWA offline).
- **Material Aero**: tokens Material 3 + superficies Aero translúcidas, temas claro/oscuro/
  auto y fallbacks de accesibilidad.
- **i18n es/en** con diccionarios RON validados y paridad de claves.
- **Copyleft**: código AGPL-3.0-or-later; contenido CC BY-SA 4.0; sonidos y diccionarios
  originales CC0.

## Licencia

- Código: [AGPL-3.0-or-later](LICENSE).
- Contenido pedagógico: [CC BY-SA 4.0](LICENSE-CONTENT).
- Sonidos y diccionarios originales: CC0 1.0 (ver [`CREDITS.md`](CREDITS.md)).

Código fuente: <https://github.com/wulflo/MathMaster>.
