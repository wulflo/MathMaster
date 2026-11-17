# INFORME — Fase M: multiplicación 空盘前乘法

Fecha: 2026-11-16. Estado: **cerrada técnicamente; publicación pendiente de autorización**.
Sustituye en prioridad a la Fase 5 por ADR-008. Método y progresión en
`docs/METODO-MULTIPLICACION.md`.

## Alcance entregado

| Bloque | Qué se hizo |
|---|---|
| `engine::multiplication` (M-001) | `Multiplication` con factores 0…999, pasos parciales, varilla de cada producto, acumulado y `state_after`; `MultiplicationDrill` M1…M7; `multiplication_level` con saturación. |
| `engine::multiplication::MulProgress` (M-006) | Aciertos e intentos por nivel con serialización versionada `v1` y rechazo de formatos desconocidos. |
| Contenido (M-002/M-005) | 12 lecciones es/en: M0 (tabla 大九九 y registro de dos cifras), M1…M7 y atajos S1…S4 con su condición de uso. |
| UI «Aprender» (M-003) | Lista de lecciones, objetivo, párrafos, ejemplo resuelto, visor de varillas de sólo lectura y reproductor anterior/siguiente/inicio con `aria-live`. |
| UI «Practicar» (M-004) | Niveles M1…M7, ejercicio generado por semilla, corrección contra el producto exacto, feedback con el resultado esperado, aciertos/intentos y tiempo. |
| Persistencia (M-006) | Claves propias `anzan.mul.level` y `anzan.mul.progress`; el nivel se satura y se reescribe normalizado; el progreso roto vuelve a cero. |
| Accesibilidad e i18n (M-007) | `aria-label` limpio en lecciones, `aria-pressed` en lecciones y niveles, `role="status"` en feedback, foco visible y conservado, color de error por tema. |

## Verificación real

- `./ci.sh` → **CI OK**: fmt, construcciones prohibidas, clippy `-D warnings`, tests,
  regla de dependencias, `cargo deny` y check wasm32.
- **135 tests** verdes: 46 core (45 + presupuesto), 75 engine, 5 content, 7 ui y 2 app.
- **280.000 casos** de propiedades (28 property tests × 10.000): el motor de multiplicación
  aporta 80.000 (producto exacto, reconstrucción desde los pasos, dos cifras por producto,
  posiciones por valor de lugar, estado tras cada paso, conmutatividad, drills por nivel y
  ida y vuelta del progreso).
- Build web: **1.708.695 B** wasm (**347.160 B gzip**), 17 % del presupuesto de 2 MB
  comprimido.
- Navegador de escritorio (Chromium del entorno, `dist/` servido en 127.0.0.1:8790):
  - lecciones M0–M7 y atajos S1–S4 en ambos idiomas; `?lang=en` traduce títulos, párrafos,
    botones y telemetría;
  - reproductor: M0 `2×4 = 08` (varillas 0,8); M2 `28×3` en dos pasos (06 → 60; 24 → 84);
    M3 `7×34` en dos pasos (21 → 210; 28 → 238); M6 `213×324` en nueve pasos hasta 69012
    con seis varillas; M7 y los atajos ocultan el reproductor y muestran su conclusión;
  - práctica: M1 `2×9 = 18` correcto; M4 `65×27 = 1755`; M6 `930×118` incorrecto con
    «era 109740»; M7 mental por Enter; «Otro ejercicio» limpia el estado;
  - persistencia: tras recargar se conservan el nivel M4 y «Aciertos 1/1»; con
    `anzan.mul.progress = v9;basura` y `anzan.mul.level = 99` arranca en M7 y 0/0, reescribe
    `v1;…` y el nivel `7`, sin errores de consola;
  - teclado: Tab recorre niveles, respuesta, comprobar y otro ejercicio; Enter envía y el
    foco permanece en el campo, que pasa a `readonly`.
- Rendimiento medido en el navegador:
  - reproductor: 1.000 transiciones (100 vueltas completas a los 9 pasos de M6 más el
    reinicio) en **24,8 ms**, ≈ **0,025 ms** por transición;
  - práctica: 50 rondas completas (pregunta → respuesta → comprobar → otro) en **1,675 s**
    incluyendo 150 esperas de `requestAnimationFrame`; 50/50 correctas y el contador pasó
    de 1/2 a 51/52.
- El código de multiplicación **no usa** `requestAnimationFrame`, `setInterval` ni
  `setTimeout`: todo el trabajo ocurre en el manejador del evento.

## Diseño y decisiones locales

- **Multiplicador por fuera, multiplicando por dentro.** Se recorre el multiplicador de
  izquierda a derecha y, por cada cifra suya, el multiplicando completo. Cada producto de
  una cifra se registra con dos cifras y se coloca por valor de lugar (unidades en
  `p+q`, decenas en `p+q+1`). El ejemplo canónico 23×45 produce 08, 12, 10 y 15.
- **El motor no dibuja.** La UI pide `state_after(paso)` y pinta las varillas; no hay
  simulación del ábaco ni animación por frame.
- **Progreso separado del de sumas.** Las claves `anzan.mul.*` no tocan `anzan.progress`,
  `anzan.sessions` ni `anzan.review.*`; una regresión en multiplicación no corrompe el
  entrenamiento de sumas.
- **Migración segura, no silenciosa.** `MulProgress::decode` exige `v1` y exactamente siete
  pares; cualquier otra cosa se rechaza y la UI reescribe un estado vacío válido.
- **Atajos con condición explícita.** S1…S4 sólo se presentan cuando se cumple su
  condición (factores 11–19, misma decena, mismas unidades, complemento a 100) y el texto
  lo dice; si no se cumple, se remite a la 空盘前乘法.
- **Práctica determinista.** La semilla avanza de uno en uno; el ejercicio se puede
  reproducir y los tests comprueban el producto exacto. No se usa aleatoriedad del sistema.

## Ajuste visual previo a la publicación

A petición del usuario se quitó el estado «Fase 4 completa» de la cabecera y se dio un
acento propio a cada sección sin cambiar el fondo azul: **soroban teal**, **multiplicación
violeta** y **Flash Anzan ámbar**, cada uno con su color de texto sobre acento en tema claro
y oscuro. El kicker ahora menciona la multiplicación. Build final: wasm **1 625 355 B**
(**343 400 B gzip**) y service worker **`anzanlab-v9`**.

## Rediseño visual (posterior al cierre)

A petición del usuario la sección dejó de ser una lista compacta y pasó a un diseño con
esquemas y tablas:

- **Ábaco dibujado** con una varilla por cifra, bola superior, beam y cuatro bolas
  inferiores; las bolas activas usan el acento violeta de la sección.
- **Tabla 大九九 9×9** con los 81 productos escritos a dos cifras; la celda resaltada sigue
  al paso actual (M0 `2×4` resalta `08`; al avanzar, `4×2` resalta el mismo `08`).
- **Tabla de productos parciales** con paso, producto de una cifra, registro de dos cifras
  y acumulado; la fila actual y las ya hechas se distinguen.
- **Tarjetas** de lección (id, título y `m×n`), de nivel (`M1`…`M7`), panel de fórmula para
  mental/atajos y tarjeta de pregunta con medidor de aciertos.
- La tarjeta sube a **62 rem** y la sección usa **dos columnas** (lectura y ábaco); el
  soroban se limita a 42 rem para no dominar la página.
- 8 cadenas nuevas es/en (`mul_abacus_title`, `mul_table_title`, `mul_table_hint`,
  `mul_steps_title` y las cuatro columnas).

Build local del rediseño: wasm **1 937 324 B** (**378 391 B gzip**), 19 % del presupuesto
de 2 MB comprimido. Verificado en Chromium de escritorio, es/en y tema claro con los tokens
de color: M6 `213×324` muestra seis varillas, nueve pasos y el acumulado `67900` tras el
paso 4; la práctica M4 `65×27` responde «Correcto» y el medidor llega al 100 %; sin errores
de consola. El rediseño está en `6770eed` y **no se ha publicado**.

**Paleta y nombre (`5f2383d`).** La app pasa a llamarse **MathMaster** en los textos es/en,
el título, el manifest, el icono y el host Tauri. El CSS define **diez acentos**
(`--anzan-accent-1`…`-10`) con hues distintos y variantes para tema oscuro.

**Acento único y tema (`7a88b96`).** La cabecera muestra las diez muestras y el acento
elegido se aplica una sola vez en `.anzan-root`, así que cabecera, botones, tarjetas
activas, tablas, ábaco, medidor y fondo cambian juntos; no quedan acentos por sección. El
fondo deriva del acento con dos resplandores y una mezcla del 16 % en claro y del 26 % en
oscuro. Junto a las muestras hay un interruptor claro/oscuro (`role="switch"`) que aplica
`data-theme` en `<html>` y se guarda en `anzan.theme`; el acento se guarda en
`anzan.accent`. Se elimina además toda la telemetría visible del drill y del audio nativo.
El service worker sube a `mathmaster-v11`.

Build local tras estos cambios: wasm **1 850 383 B** (**373 012 B gzip**), 138 tests y
`CI OK`. Verificado en Chromium de escritorio es/en: diez muestras, cambio global de acento
y fondo, toggle en la misma fila que las muestras, ambos ajustes persistentes tras
recargar, sin «Telemetría»/«Telemetry» y sin errores de consola.

## Límites y riesgos

- Sólo enteros hasta 3×3 cifras; decimales, división y raíces quedan fuera de esta fase.
- La práctica mide aciertos y tiempo, pero no modela el dominio del método ni decide
  cuándo promover de nivel: el usuario elige el nivel.
- El modo mental M7 es una presentación sin ábaco visible; la app no puede comprobar que
  la persona haya calculado mentalmente.
- El tiempo de respuesta usa `js_sys::Date` (resolución de milisegundos) y no descuenta
  pausas ni cambios de pestaña.
- Los atajos están verificados por producto exacto, no por eficacia pedagógica con
  estudiantes.

## NO VERIFICADO

- **Móvil real**, tableta y web móvil; sólo Chromium de escritorio.
- **Lector de pantalla** (NVDA, VoiceOver, TalkBack) y navegación por lector; la estructura
  ARIA se revisó a mano y con el árbol de accesibilidad del navegador.
- **Firefox y Safari**; no se probó el render de `color-mix` ni el foco en esos motores.
- **Perfil de CPU, memoria y batería**; no hay medición de energía ni de consumo en reposo.
- **Eficacia pedagógica**: no hay estudio con usuarios ni comparación con enseñanza
  tradicional.
- **Publicación del rediseño**: la fase M está en GitHub Pages desde `aef0ca6` (SW
  `anzanlab-v9`, wasm servido 1 707 369 B), pero el rediseño `6770eed` sólo está en local.
- **Tema claro del rediseño**: se revisó inyectando los tokens claros en el navegador, no
  con emulación real de `prefers-color-scheme`.

## Cómo probarlo

```bash
./scripts/build-web.sh
cd dist && python3 -m http.server 8790
# abrir http://127.0.0.1:8790/ y http://127.0.0.1:8790/?lang=en
```

Pasos: recorrer M0–M7 y S1–S4 con el reproductor; practicar un acierto y un fallo en M1 y
M6; recargar para comprobar la persistencia; corromper `anzan.mul.progress` en la consola y
recargar para ver la recuperación; repetir en inglés y con teclado.

## Próxima fase

**Fase 5 con alcance reducido (ADR-007):** accesibilidad avanzada, fotosensibilidad, web
móvil digna y presupuesto de rendimiento, sin Android ni háptico. Requiere el «sí» explícito
de la compuerta. La fase M ya está publicada (SW `v9`); publicar el rediseño `6770eed`
requiere autorización aparte. No se crean releases ni bundles.
