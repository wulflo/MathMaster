# INFORME — Fase 3a: progresión y estadísticas

Fecha: 2026-11-13. Estado: **cerrada técnicamente**, pendiente de publicación (P2-010) y de
la Fase 3b (sesiones, exportación, errores por caso pedagógico y repaso espaciado).

## Alcance entregado

| Bloque | Qué se hizo |
|---|---|
| `engine::progress` | Tabla de 6 niveles (`LEVELS`, `level_spec`) con dígitos, términos, frames de estímulo y hueco. `ProgressState` con promoción tras 2 aciertos, descenso tras 2 fallos, precisión, mejor racha y `encode`/`decode` v1 validado. |
| UI de progresión | Panel con nivel, acierto, racha, intentos y mejor racha; aviso de subida/bajada; reinicio de progreso. |
| Modo | Interruptor «progresión automática» / «práctica libre». En automático, los campos de dígitos y términos se deshabilitan y siguen al nivel; en libre se editan a mano. |
| Persistencia | `anzan.progress` (una línea versionada) y `anzan.progressive` en `localStorage`; al cargar se validan y un valor corrupto vuelve al estado por defecto. |
| Contenido | 12 claves nuevas es/en con paridad y estilos del panel. |

## Verificación real

- `./ci.sh` → **CI OK**: fmt, construcciones prohibidas, clippy `-D warnings`, tests,
  regla de dependencias, `cargo deny` y check wasm32.
- **86 tests** verdes (antes 75): 45 de core, 1 contador de presupuesto, 35 de engine,
  2 de content y 3 de ui (los tres del módulo Flash Anzan).
- **190.000 casos** de propiedades (antes 150.000): los cuatro proptest de `progress`
  aportan 40.000 (10.000 cada uno). Core conserva 110.000 y `drill` 40.000.
- Build web: `anzan_app_bg.wasm` = **1.005.246 B** (**219.812 B gzip**), dentro del
  presupuesto de < 2 MB comprimido. El crecimiento de gzip frente a la Fase 2 es de
  ~16,5 KB.
- Navegador de escritorio (Chromium del entorno OpenCode, servidor local 127.0.0.1:8768):
  - el panel arranca en nivel 1 con 3 términos de 1 dígito;
  - dos respuestas correctas seguidas → «Level up 2», nivel 2 (1 dígito, 5 términos) con
    los campos de configuración sincronizados;
  - precisión, racha, intentos y mejor racha se actualizan en cada respuesta;
  - tras recargar, `localStorage` contiene `anzan.progress = v1,2,0,0,2,2,2` y el panel
    sigue en nivel 2;
  - desactivar la progresión automática muestra «Free practice», persiste
    `anzan.progressive = 0` y vuelve a habilitar los campos manuales;
  - sin errores de consola.

## Diseño del modelo

- La progresión es deliberadamente simple y auditable: **+1 nivel cada 2 aciertos
  consecutivos**, **−1 nivel cada 2 fallos consecutivos**, con saturación en 1 y 6. No es
  un algoritmo de repaso espaciado completo; el superprompt pide «FSRS o equivalente con
  datos propios» y esa parte queda en P3-008, dentro de la Fase 3b.
- Los parámetros de nivel viven en el motor (`LEVELS`), no en la UI: mañana el host Tauri o
  un futuro currículo pueden reutilizarlos. La UI sólo dibuja el estado.
- El estado se serializa en una línea (`v1,level,streak,misses,attempts,correct,best`) y
  `decode` rechaza versión distinta, nivel fuera de 1..=6, aciertos mayores que intentos,
  rachas imposibles o campos extra.

## Riesgos y límites

- **Duración no monótona:** el nivel 6 no dura más frames que el nivel 1 (más términos pero
  más rápidos: 168 vs 135 frames). La dificultad sube por dígitos y velocidad, no por
  duración. Un test lo fija explícitamente para que nadie asuma lo contrario.
- **Progresión en un solo eje:** no distingue tipos de operación ni casos pedagógicos; eso
  llega con P3-007 y P3-008.
- **Sin historial:** el estado actual es acumulado; no hay sesiones fechadas ni exportación
  (P3-006).
- El progreso y los ajustes viven en `localStorage` de la cápsula web; no hay migración a
  disco ni sincronización (host Tauri, Fase 4).

## NO VERIFICADO

- **Audio audible** y su latencia: igual que en Fase 2, el `AudioContext` se crea sin
  errores, pero no se comprobó que suene.
- **Firefox, Safari, Android y web móvil**: sin probar.
- **Publicación**: la URL de Pages sigue sirviendo la Fase 1; los builds de Fase 2 y 3a
  están en `dist/` sin push (P2-010).
- **Pestaña en segundo plano**: `requestAnimationFrame` se suspende y la corrida no es
  válida; no hay pausa explícita.
- **Host Tauri de escritorio** y audio nativo: Fase 4.

## Cómo probarlo

```bash
./scripts/build-web.sh
cd dist && python3 -m http.server 8768
# abrir http://127.0.0.1:8768/
```

Pasos: pulsar «Empezar», responder bien dos veces seguidas y comprobar el aviso de subida
de nivel y la sincronización de dígitos/términos; recargar para ver la persistencia;
desactivar la progresión automática y comprobar que los campos vuelven a habilitarse.

## Siguiente paso propuesto

**Fase 3b:** historial de sesiones con exportación CSV (P3-006), clasificación de errores
por caso pedagógico (P3-007) y repaso espaciado tipo FSRS con datos propios (P3-008);
después, cierre completo de Fase 3 (P3-009). Requiere autorización, igual que la
publicación de P2-010.
