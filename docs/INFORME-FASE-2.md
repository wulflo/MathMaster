# INFORME — Fase 2: motor temporal, audio y drills Flash Anzan

Fecha: 2026-11-13. Estado: **cerrada técnicamente**, pendiente de publicación (P2-010) y de
la compuerta a Fase 3.

## Alcance entregado

| Bloque | Qué se hizo |
|---|---|
| `engine::schedule` | `FramePlan` de segmentos estímulo/hueco con consulta binaria, `FramePlanBuilder` validado y `FrameTelemetry` de ventanas fijas (deltas de frame y error de aparición). |
| `engine::drill` | Generador determinista de secuencias Flash Anzan por dígitos, términos y semilla; respuesta y verificación. |
| `engine::audio` | `ToneSpec`, `ClickBuffer` sintetizado con envolvente, mapeo frame→muestra y offsets de clic. |
| `anzan-ui::flash` | Modo Flash Anzan: plan reproducido con `requestAnimationFrame`, DOM directo, respuesta, feedback y telemetría visible. |
| Audio web | WebAudio crea un nodo por estímulo (nunca por frame) y respeta el mute. |
| Contenido | 17 claves nuevas es/en con paridad; estado de fase actualizado. |
| Persistencia | Dígitos, términos y sonido en `localStorage` (`anzan.drill`). |
| Plataforma | Service worker v6; build wasm y sitio estático regenerados en `dist/`. |

## Verificación real

- `./ci.sh` → **CI OK**: fmt, construcciones prohibidas, clippy `-D warnings`, tests,
  regla de dependencias, `cargo deny` y check wasm32.
- **75 tests** verdes (antes 52): 45 de core, 1 contador de presupuesto, 25 de engine,
  2 de content, 2 de ui.
- **150.000 casos** de propiedades (antes 110.000): los cuatro proptest nuevos de
  `engine::drill` aportan 40.000. Los de core conservan sus 110.000.
- Build web: `anzan_app_bg.wasm` = **877.327 B** (**203.248 B gzip**), JS = 38.059 B.
  Dentro del presupuesto de < 2 MB comprimido.
- Navegador de escritorio (Chromium del entorno OpenCode, servidor local 127.0.0.1:8768):
  - el modo muestra la secuencia y al terminar habilita el campo de respuesta;
  - respuesta incorrecta → «Incorrecto» y «Respuesta correcta: 189»;
  - `?lang=en` muestra «Flash Anzan / Ready to start / Start / Telemetry»;
  - ajustes 3 dígitos y 7 términos sobreviven a la recarga (`localStorage` = `3,7,1`);
  - sin errores de consola (sólo el aviso de CSP del propio Electron).

### Medición de presentación web

Una corrida de 5 términos × 36 frames = 180 frames (≈3 s), plan de 30 frames por estímulo y
6 de hueco, medida en el callback de `requestAnimationFrame`:

| Métrica | Valor |
|---|---|
| Frame p95 | 16,8 ms |
| Error absoluto de aparición p95 | 0,0 ms (5 estímulos) |
| Navegador | Chromium/Electron del entorno, vsync del portátil |
| Fecha | 2026-11-13 |

Método: `FrameTelemetry` registra el delta entre callbacks y, en cada cambio de estímulo,
el error entre el tiempo transcurrido y `frame_planificado × cadencia media medida`. El
reloj de inicio se fija en el primer callback, no antes de pedirlo.

**Límites:** una sola corrida, una sola máquina y un solo navegador. No es una medición
estadística ni de scanout físico. No se heredan los números de Tauri.

## NO VERIFICADO

- **Audio audible** (DAC, altavoz, auriculares): el `AudioContext` se crea y no hay errores,
  pero no se comprobó que el clic suene ni su latencia acústica.
- **Firefox, Safari, Android y web móvil**: sin probar. La cadencia de `rAF` y WebAudio
  pueden diferir.
- **Corte de red real sobre Pages** con la Fase 2: la versión nueva no está publicada.
- **Pestaña en segundo plano**: `requestAnimationFrame` se suspende; la secuencia queda en
  pausa y la telemetría de esa corrida no es válida. No se implementó pausa explícita.
- **Host Tauri de escritorio**: no existe aún como crate de producción (Fase 4). El audio
  nativo cpal y la supervisión frame a frame desde Rust siguen pendientes.
- **Scanout físico**: la telemetría mide timestamps de callback, no el píxel en pantalla.

## Decisiones y riesgos

- El plan se expresa en frames; la UI avanza un frame por callback de `rAF`. No se convierte
  ms a frames “a ojo” ni se declara precisión sin telemetría (ADR-002).
- El audio web se dispara al aparecer el estímulo, no se programa por adelantado: evita
  deriva si la cadencia real difiere de 60 Hz, a costa de la latencia del `AudioContext`.
  Es una decisión local de la cápsula web, no un cambio de ADR.
- El callback de render no asigna memoria: sólo consulta el plan, escribe `textContent` y
  registra telemetría. La única asignación por estímulo es el nodo de audio de WebAudio.
- Riesgo abierto: con ventanas de 600 frames, una sesión de más de 20 estímulos llena la
  ventana de telemetría (se conservan los últimos 600 frames); es suficiente para el modo
  actual y se revisará si se amplía.

## Cómo probarlo

```bash
./scripts/build-web.sh
cd dist && python3 -m http.server 8768
# abrir http://127.0.0.1:8768/
```

Pasos: pulsar «Empezar», esperar a que aparezca «Resultado», escribir la suma y
«Comprobar». La telemetría aparece bajo el panel. Repetir con `?lang=en` y recargar para
comprobar la persistencia de ajustes.

## Siguiente fase propuesta

**Fase 3 — progresión, sesiones y estadísticas:** currículo por niveles, historial de
sesiones, repaso espaciado con datos propios, estadísticas por caso pedagógico y
exportación. Requiere el «sí» de la compuerta (y, si se quiere publicar la Fase 2,
autorización de push para P2-010).
