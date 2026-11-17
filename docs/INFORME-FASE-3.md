# INFORME — Fase 3: progresión, sesiones y estadísticas

Fecha: 2026-11-14. Estado: **cerrada técnicamente y publicada**. Incluye la Fase 3a
(progresión, `docs/INFORME-FASE-3A.md`) y la Fase 3b (historial, casos y repaso).

## Alcance entregado

| Bloque | Qué se hizo |
|---|---|
| `engine::progress` (3a) | 6 niveles, promoción tras 2 aciertos, descenso tras 2 fallos, precisión y serialización v1. |
| `engine::session` (3b) | `SessionRecord` por drill (fecha Unix, nivel, dígitos, términos, acierto, tiempo y caso dominante), historial de hasta 100 sesiones y `to_csv`. |
| `engine::pedagogy` (3b) | Clasificación de cada suma parcial: directo, complemento a 5, complemento a 10 o acarreo en cadena. `Drill::cases` y `Drill::hardest_case`. |
| `engine::review` (3b) | Planificador propio tipo SM-2 con facilidad acotada (1,30–3,00), intervalos de 10 min a 1 año, `Again/Hard/Good/Easy` y serialización por nivel. |
| UI (3a+3b) | Panel de progresión, aviso de repaso, estadísticas por caso, últimas 6 sesiones, descarga CSV y reinicio de progreso. |
| Persistencia | `anzan.progress`, `anzan.progressive`, `anzan.sessions`, `anzan.review.<nivel>` y `anzan.drill`. |
| Contenido | 29 claves i18n nuevas es/en con paridad; estado de fase y service worker v7. |
| Publicación | Fase 2 + 3a publicadas en GitHub Pages (P2-010); la Fase 3b se publica con el cierre. |

## Verificación real

- `./ci.sh` → **CI OK**: fmt, construcciones prohibidas, clippy `-D warnings`, tests,
  regla de dependencias, `cargo deny` y check wasm32.
- **109 tests** verdes: 45 core + 1 presupuesto + 58 engine + 2 content + 3 ui (antes 86).
- **240.000 casos** de propiedades: 110.000 de core, 40.000 de `drill`, 40.000 de
  `progress`, 20.000 de `pedagogy`, 20.000 de `session` y 10.000 de `review`.
- Build web: **1.187.902 B** wasm (**259.425 B gzip**), dentro del presupuesto de < 2 MB.
- Navegador de escritorio (Chromium del entorno, servidor local 127.0.0.1:8768):
  - sesión respondida: fila con fecha, nivel, tiempo, caso «Complemento a 10» y resultado;
  - estadísticas por caso: 1/1 (100 %) en el caso dominante y 0/0 en el resto;
  - repaso tras un acierto: «En 24 h»; tras un fallo: «En 1 h» y facilidad 2300;
  - `anzan.sessions = v1,…,c10` y `anzan.review.1 = v1,0,1,600000,2300,…` tras el fallo;
  - recarga: historial, estadísticas y repaso se conservan;
  - `?lang=en`: «Sessions / Review / By case / Time / Export CSV / Complement to 10»;
  - descarga CSV interceptada en el navegador: cabecera y una fila por sesión, con el
    contenido exacto `timestamp_unix_ms,…,case`;
  - sin errores de consola.
- Publicación: el workflow **Pages** de GitHub Actions terminó en verde
  (run 43534450991, commit `98a03fe`) y <https://wulflo.github.io/MathMaster/> sirve la
  Fase 3 completa. Comprobado tras el despliegue:
  - el `service-worker.js` servido es **v7** (md5 `d4be61a3…`) y la caché activa es
    `anzanlab-v7`;
  - el service worker está **activado y controlando** la página;
  - el precache contiene las seis entradas (`/`, `index.html`, `anzan_app.js`, wasm,
    manifiesto e icono) y el wasm cacheado es **byte a byte** el servido (SHA-256);
  - un registro limpio (desregistrar, borrar cachés, cerrar la pestaña y abrir de nuevo)
    instala v7 y precachea correctamente; sin errores de consola.
  - Nota: en una comprobación intermedia la caché apareció vacía porque el borrado manual
    compitió con la instalación; no se reprodujo en el registro limpio.

## Diseño y decisiones locales

- **Caso dominante por drill.** Un drill mezcla sumas de varios casos; atribuir el fallo a
  un caso concreto sería inventar. Cada sesión se atribuye al caso *más difícil* que
  ejercitó, y el CSV guarda ese código. Las estadísticas por caso son, por tanto,
  «sesiones cuyo caso dominante fue X», no «errores de la suma X». Queda documentado.
- **Repaso propio, no FSRS.** Se implementó un planificador transparente tipo SM-2
  (facilidad, intervalos, lapsos) en lugar de portar FSRS. El superprompt admite
  «FSRS o equivalente con datos propios»; el estado se serializa en una línea por nivel y
  no hay dependencias opacas.
- **El repaso es informativo.** La UI muestra cuándo toca repasar el nivel actual, pero no
  bloquea ni cambia el nivel del drill. Programar la práctica por vencimiento queda como
  mejora posterior, no como promesa.
- **CSV sin comillas.** Todos los campos son enteros o códigos ASCII estables; no hay
  texto libre, así que no hace falta escapado. La marca de tiempo es Unix en ms para no
  depender de la zona horaria.
- **Cero asignaciones en el frame.** Todo el trabajo nuevo (sesión, CSV, revisión,
  clasificación) ocurre al generar el drill o al enviar la respuesta, nunca en el callback
  de `requestAnimationFrame`.

## Límites y riesgos

- El historial se guarda en `localStorage` (cápsula web); no hay sincronización ni
  migración a disco. El host Tauri de escritorio es Fase 4.
- La fecha se muestra en ISO derivada de Unix; no se localiza por idioma.
- El repaso no tiene pasos de aprendizaje separados ni parámetros por caso.
- La ventana de telemetría (600 frames) sigue siendo la de Fase 2.

## NO VERIFICADO

- **Audio audible** y latencia acústica.
- **Firefox, Safari, Android y web móvil**; tampoco la descarga CSV fuera de Chromium.
- **Pestaña en segundo plano** y scanout físico.
- **Host Tauri de escritorio** y audio nativo (Fase 4).
- **Carga realmente offline**: el precache v7 está completo y el wasm coincide con el
  servido, pero el navegador del entorno no ofrece emulación de red sin conexión; no se
  simuló un corte real. La estrategia «red primero, caché de respaldo» del service worker
  ya se validó en fases anteriores solo por inspección y precache.

## Cómo probarlo

```bash
./scripts/build-web.sh
cd dist && python3 -m http.server 8768
# abrir http://127.0.0.1:8768/
```

Pasos: completar un drill, comprobar la fila de sesión, el repaso y las estadísticas por
caso; pulsar «Exportar CSV» y abrir el archivo; recargar para ver la persistencia; repetir
con `?lang=en`.

## Próxima fase

**Fase 4 — host de escritorio Tauri y audio nativo:** crate/binario Tauri con la misma UI
Leptos, audio cpal con `Fixed(256)`, reloj y supervisión desde Rust, persistencia en disco
y medición de la app real. Requiere el «sí» de la compuerta.
