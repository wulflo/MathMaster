# CHECKLIST — Rediseño integral de UX, pedagogía y entrenamiento

Fuente única de verdad de esta iniciativa (seis etapas). Se lee **antes** de cualquier
modificación y tras cualquier corte de sesión/red. Estados: `[ ]` pendiente · `[~]` en curso ·
`[x]` completada. Protocolo de reanudación:

1. Leer este archivo y `docs/LOG.md`.
2. Buscar la primera tarea `[~]` (o, si no hay, la primera `[ ]`).
3. Verificar `git status --short`.
4. Retomar exactamente en esa tarea; no repetir trabajo ya verificado.
5. Al cerrar cada etapa: marcar sus tareas, escribir resumen en `docs/LOG.md` y ejecutar
   `./ci.sh` (fmt + construcciones prohibidas + clippy `-D warnings` + tests/proptest +
   `./scripts/check-deps.sh` + `cargo deny check` + wasm32).

## Invariantes (no negociables)

- `unsafe_code = "deny"` en el workspace; `#![forbid(unsafe_code)]` en `core` y `content`.
- Cero cadenas visibles en Rust: textos y contenido pedagógico sólo en
  `crates/content/content/i18n/{es,en}.ron` y `crates/content/content/multiplication/{es,en}.ron`,
  con paridad exacta de claves y tests en `crates/content/src/lib.rs`.
- Sin asignaciones ni bloqueos en el callback de `requestAnimationFrame` (`crates/ui/src/flash.rs`)
  ni en el hilo de audio (`crates/desktop/src/audio.rs`).
- Los arneses `--ui-selftest` y `--measure` siguen encontrando `.anzan-start`, `.anzan-answer`,
  `.anzan-submit`, `.anzan-drill-result` y `.anzan-session-list li`: la pestaña activa por
  defecto es **Entrenamiento → Flash Anzan** (no se persiste la pestaña entre sesiones).
- Sin `unwrap(`, `expect(`, `panic!`, `todo!`, `unimplemented!` ni `dbg!` (`scripts/check-forbidden.sh`).

---

## Etapa 1 — Planificación y contratos de datos

| ID | Archivos exactos | Criterio de aceptación | Estado |
|----|------------------|------------------------|--------|
| R1-01 | `docs/CHECKLIST-REDESENO.md`, `docs/CHECKLIST.md`, `docs/LOG.md` | Checklist creado, enlazado y con las 6 etapas; entrada de inicio en el LOG | [x] |
| R1-02 | `crates/content/src/lib.rs` | Esquemas `FundamentalsContent`/`FundamentalsChapter`/`FundamentalsSection`/`FormulaRow`, lecciones ampliadas (fundamento, posición *Teii-hō*, procedimiento, trampas, ejemplo comentado) y claves nuevas de UI, con `deny_unknown_fields` y documentación pública | [x] |
| R1-03 | `crates/content/content/i18n/es.ron`, `en.ron`, `multiplication/es.ron`, `en.ron` | Paridad exacta de claves; seis capítulos de fundamentos y M0–M7/S1–S4 con los campos nuevos, sin cadenas visibles en Rust | [x] |
| R1-04 | `crates/content/src/lib.rs` | Tests de paridad/estructura/capítulos/ejemplos; `./ci.sh` verde | [x] |

## Etapa 2 — Motor pedagógico y nuevos modos

| ID | Archivos exactos | Criterio de aceptación | Estado |
|----|------------------|------------------------|--------|
| R2-01 | `crates/engine/src/speed.rs`, `crates/engine/src/lib.rs` | `DrillSpeed` validado (exposición 100..=1500 ms, hueco 0..=1500 ms), `frames_for` vía `frames_for_duration`, cadencia términos/s; tests | [x] |
| R2-02 | `crates/engine/src/mixed.rs`, `crates/engine/src/lib.rs` | `MixedDrill` suma/resta determinista con acumulado parcial ≥ 0, términos con signo y `proptest` de 10 000 casos | [x] |
| R2-03 | `crates/engine/src/focus.rs`, `crates/engine/src/lib.rs` | `focused_drill` por `PedagogicalCase` (complemento 5, complemento 10, acarreo) con `proptest` de 10 000 casos | [x] |
| R2-04 | `crates/engine/src/mitori.rs`, `crates/engine/src/lib.rs` | `MitoriConfig` (2..=6 dígitos, 3..=15 filas, límite opcional) y columnas alineadas con separador cada 3 cifras; tests | [x] |
| R2-05 | `crates/engine/src/*.rs` | `./ci.sh` verde con el motor ampliado | [x] |

## Etapa 3 — Pestañas horizontales y remodelado visual

| ID | Archivos exactos | Criterio de aceptación | Estado |
|----|------------------|------------------------|--------|
| R3-01 | `crates/ui/src/tabs.rs`, `crates/ui/src/lib.rs` | Cabecera compacta (título, lema, nivel/racha vivos, ES/EN, tema, acento), `role="tablist"` con flechas/Home/End, indicador activo; 4 pestañas con subsecciones; pestaña por defecto Entrenamiento → Flash | [x] |
| R3-02 | `crates/ui/assets/style.css` | Material Aero conservado; pestañas adosadas, transiciones suaves, `prefers-reduced-motion`, breakpoints móviles; sin desbordamiento horizontal | [x] |
| R3-03 | `crates/ui/src/lib.rs` | Paneles cableados (aprendizaje, laboratorio, entrenamiento, progreso) compartiendo señales de tema/acento/demo; selectores de arnés visibles; `./ci.sh` verde | [x] |

## Etapa 4 — Zona de aprendizaje completa

| ID | Archivos exactos | Criterio de aceptación | Estado |
|----|------------------|------------------------|--------|
| R4-01 | `crates/ui/src/fundamentals.rs` | Capítulos 1–6 renderizados desde RON con tablas de fórmulas, pasos guiados y ábaco interactivo; sin texto hardcodeado | [x] |
| R4-02 | `crates/ui/src/multiplication.rs` | Lecciones muestran fundamento, *Teii-hō* (*m+n*, 前空减一), procedimiento, trampas y ejemplo comentado; vista de atajos S1–S4 y tabla 大九九 interactiva con resaltado de fila/columna | [x] |
| R4-03 | `crates/ui/src/lab.rs`, `crates/ui/src/soroban.rs` | Laboratorio con 4–13 varillas, cinco etapas de fading, lectura opcional y atajos por varilla; demostrador de `plan_add`/`plan_sub` con regla por toque (Directo/5/10/Acarreo/Préstamo) | [x] |
| R4-04 | `crates/ui/src/*` | `./ci.sh` verde | [x] |

## Etapa 5 — Entrenamiento avanzado y auto-detección

| ID | Archivos exactos | Criterio de aceptación | Estado |
|----|------------------|------------------------|--------|
| R5-01 | `crates/ui/src/answer.rs` | `AnswerBox` reutilizable: auto-foco, `on:input` que valida al digitar, acierto automático con confirmación ~300 ms, avance automático activado por defecto, fallo por longitud/Enter con respuesta esperada y «Ver en el soroban», keypad táctil, `0–9`/Backspace/Enter/Espacio | [x] |
| R5-02 | `crates/ui/src/flash.rs` | Cuenta regresiva 3·2·1 con punto de fijación, modo sumas/restas con signo `−`, control de exposición y gap en ms con frames y cadencia en vivo, «Repetir misma secuencia», integración con `AnswerBox`; rAF sin asignaciones; selectores de arnés intactos | [x] |
| R5-03 | `crates/ui/src/training.rs` | Mitori Anzan: columnas 3–15 filas, 2–6 dígitos, cronómetro, lectura guiada, separador de millares; integración con `AnswerBox` | [x] |
| R5-04 | `crates/ui/src/training.rs`, `crates/ui/src/multiplication.rs` | Multiplicación por niveles con auto-respuesta; entrenamiento dirigido al caso de menor precisión con repetición y resolución en el soroban | [x] |
| R5-05 | `crates/ui/src/*` | `./ci.sh` verde y verificación en navegador (si el entorno lo permite) | [x] |

## Etapa 6 — Integración de escritorio, auditoría y cierre

| ID | Archivos exactos | Criterio de aceptación | Estado |
|----|------------------|------------------------|--------|
| R6-01 | `crates/desktop/src/selftest.rs`, `crates/desktop/src/measure.rs` | Arneses compatibles sin romper `--ui-selftest`/`--measure` (selectores acotados a `.anzan-drill` en la pestaña inicial); ejecución real si hay webview | [x] |
| R6-02 | `README.md` | Readme en inglés embellecido con arquitectura, uso, comandos y licencias de dependencias y herramientas | [x] |
| R6-03 | `docs/CHECKLIST-REDESENO.md`, `docs/LOG.md`, `docs/CHECKLIST.md` | 100 % `[x]`; `./ci.sh` y `./scripts/build-web.sh` verdes; resumen final registrado | [x] |

## Punto de reanudación

> **Rediseño completo: 6 etapas y 38 tareas en estado `[x]`.** `./ci.sh` y
> `./scripts/build-web.sh` verdes; verificación en navegador realizada (flujo del arnés,
> auto-detección al teclear, «Ver en el soroban», temas claro/oscuro y esquemas SVG).
> No se ha publicado: los cambios siguen en local, pendientes de autorización de push.
