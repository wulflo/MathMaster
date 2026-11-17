# Multiplicación — método y plan de la sección de aprendizaje

Fecha: 2026-11-14. Estado: **especificación pedagógica y plan**; no implementado todavía.

La app hoy sólo entrena **sumas** (soroban libre + Flash Anzan). La multiplicación no
existe en `core`, `engine`, `ui` ni en el contenido. Esta nota fija **qué** hay que enseñar,
**cómo** se verifica y **qué tareas** lo implementan, para no inventar un método a medias.

## Qué significa «como los prodigios chinos»

El entrenamiento de los calculistas de 珠心算 (*zhūxīnsuàn*, aritmética mental con ábaco)
parte del ábaco chino (*suanpan*) y evoluciona en tres etapas:

1. **珠算** — calcular moviendo las cuentas del ábaco.
2. **看算** — calcular mirando el ábaco, sin moverlo.
3. **珠心算 / 脑算** — calcular con una imagen mental del ábaco.

Para multiplicar, el método de enseñanza actual en珠心算 es la **空盘前乘法** (*kōngpán
qiánchéngfǎ*, «multiplicación frontal con ábaco vacío»): no se colocan los factores en el
ábaco; se multiplica desde la cifra más alta y se van sumando los productos parciales.

No es un truco único: se apoya en la tabla 大九九, en registrar cada producto de dos
cifras y en sumar por posiciones. Los atajos de «prodigio» (十几乘十几, 头同尾合十, etc.)
son complementos para casos especiales, no el método general.

## Fundamentos que la sección debe enseñar

### 1. Tabla 大九九 y单积 de dos cifras

- Tabla completa del 1 al 9 (81 productos), en cualquier orden.
- Cada producto de dos cifras de un dígito se escribe **siempre con dos cifras**:
  `1×5 = 05`, `6×4 = 24`, `3×0 = 00`. Ese `0` es una cifra posicional, no decorativa.
- Énfasis en los productos que cruzan 5 y 10, porque obligan a complementos en el ábaco.

### 2. Posiciones y varillas

- Varilla de unidades = posición 0; decenas = 1; centenas = 2; etc.
- Para multiplicar `A` (m cifras) por `B` (n cifras), el producto de la cifra `a_i`
  (contando desde la izquierda) por `b_j` se coloca con sus **unidades** en la varilla
  `(m-1-i) + (n-1-j)` y sus **decenas** en la varilla siguiente a la izquierda.
- Si se fija la varilla de unidades antes de empezar, el resultado se lee directo.
- Regla tradicional de control: el producto tiene `m+n` cifras, o `m+n-1` si la primera
  varilla usada quedó vacía («位数相加，前空减一»).

### 3. 空盘前乘法: procedimiento

1. Elegir como multiplicador el factor con menos cifras (o con ceros o cifras repetidas),
   porque la multiplicación es conmutativa.
2. Fijar la varilla de unidades y la primera varilla de trabajo.
3. Recorrer las cifras del multiplicador de izquierda a derecha. Para cada una, recorrer
   las del multiplicando de izquierda a derecha.
4. Cada producto de un dígito se registra con dos cifras y se **suma** en su posición, con
   los complementos a 5 y a 10 que hagan falta.
5. Al terminar, leer el resultado en el ábaco. Las cifras que no se usaron quedan en cero.

Ejemplo `23 × 45` con multiplicando 23 y multiplicador 45 (cada fila se lee
*multiplicador × multiplicando*):

| Paso | Producto | Varillas (unidades en 0) | Ábaco acumulado |
|---|---|---|---|
| 4×2 | 08 | decenas en 3, unidades en 2 | 800 |
| 4×3 | 12 | decenas en 2, unidades en 1 | 920 |
| 5×2 | 10 | decenas en 2, unidades en 1 | 1020 |
| 5×3 | 15 | decenas en 1, unidades en 0 | 1035 |

Resultado: **1035**. Primero se agota la cifra `4` del multiplicador contra las dos cifras
del multiplicando; después la cifra `5`. En el ábaco se ve cómo cada producto se suma a lo
que ya había, sin borrar los anteriores.

El orden importa para enseñar: la cifra del multiplicador se mantiene en la cabeza y se
barre el multiplicando de izquierda a derecha. `28 × 3` sigue el mismo patrón: el producto
`3×2` se registra como `06` y el `3×8` como `24`, empezando este último una varilla a la
derecha.

Ejemplo con cero intermedio: `405 × 6 = 2430`. La cifra `0` del multiplicando no genera
producto; hay que **saltar** su posición sin desplazar las demás.

### 4. Progresión de dificultad

| Nivel | Forma | Ejemplo | Qué introduce |
|---|---|---|---|
| M1 | 1×1 | 7×8 | tabla y registro de dos cifras |
| M2 | 2×1 | 28×3 | recorrido del multiplicando y acarreo |
| M3 | 1×2 | 7×34 | multiplicador de dos cifras y desplazamiento |
| M4 | 2×2 | 23×45 | cuatro productos parciales y suma posicional |
| M5 | 3×2 | 213×32 | más filas y control de ceros |
| M6 | 3×3 | 213×324 | caso completo de la sección |
| M7 | mental 2×2 | 23×45 sin ábaco | imagen mental, primero precisión y luego tiempo |

### 5. Del ábaco a la mente

- Primero se resuelve con el ábaco visible.
- Después se oculta el ábaco y se pide **imaginar** la misma secuencia de productos.
- Al final se practica sin ábaco y con tiempo, pero la app no afirma que el usuario «vea»
  el ábaco: sólo mide aciertos y tiempo.
- La sección debe decir con claridad que la velocidad es consecuencia de la práctica, no
  un requisito para empezar.

### 6. Atajos de cálculo (avanzado, opcional)

Se enseñan **después** del método general y siempre indicando su condición de uso:

| Atajo | Condición | Fórmula | Ejemplo |
|---|---|---|---|
| 十几乘十几 | ambos entre 11 y 19 | `(10+a)(10+b) = 100 + 10(a+b) + ab` | 12×14 = 100+60+8 = **168** |
| 头同尾合十 | misma decena, unidades suman 10 | `a(a+1)·100 + b·c` | 43×47 = 20·100+21 = **2021** |
| 尾同头合十 | mismas unidades, decenas suman 10 | `(a·b+c)·100 + c²` | 34×74 = 25·100+16 = **2516** |
| 补数乘法 | factor cerca de 100/1000 | `A(B) = A·10^k − A·complemento` | 5739×996 = 5 739 000 − 22 956 = **5 716 044** |

Advertencia visible: los atajos no sustituyen la 空盘前乘法; si la condición no se cumple,
se usa el método general.

## Cómo se integra en la app

La app es una sola página Leptos con tres bloques: soroban libre, Flash Anzan y el panel de
progreso. La sección nueva se añade como **«Aprender a multiplicar»**, con dos modos:

### Aprender

- Lista de unidades M0…M6 con título, objetivo y duración estimada.
- Cada lección tiene: explicación breve, ejemplo resuelto **paso a paso**, botones
  *anterior/siguiente*, y una varilla/ábaco de sólo lectura que muestra el estado tras cada
  paso.
- El reproductor de pasos se controla con teclado y anuncia el paso en `aria-live`.
- Al final de la lección, botón «practicar este nivel».

### Practicar

- Ejercicios generados de forma determinista por nivel y semilla, con respuesta numérica.
- Ayuda opcional: mostrar el siguiente producto parcial o la varilla donde va.
- Corrección inmediata, explicación del error y repetición del mismo tipo.
- Telemetría propia: tiempo de respuesta, aciertos y nivel; **no** se mezcla con las
  sesiones de suma para no romper el formato v1.
- Progreso de multiplicación en claves propias, versionadas, con `localStorage` en web y
  `store.json` en escritorio.

### Ábaco de la lección

- El soroban actual tiene 4 varillas y es interactivo; la multiplicación necesita hasta
  7–9 varillas de trabajo.
- Se añade un **visor de varillas de sólo lectura** (dígitos 0–9, sin cuentas animadas) para
  las lecciones y los pasos; el ábaco interactivo se mantiene para el soroban libre.
- Sin animación por frame: el estado cambia por paso; respeta `prefers-reduced-motion`.

## Verificación

- **Motor:** propiedades con ≥10 000 casos por categoría: `a×b` exacto, conmutatividad,
  cero, acarreos, colocación de cada producto parcial y reconstrucción del resultado desde
  los pasos. Límites documentados (por ejemplo, hasta 3×3 cifras con `u64`).
- **Contenido:** paridad es/en, todos los ejemplos con resultado comprobado por test (no a
  mano), y ninguna lección sin ejercicio.
- **UI:** el reproductor avanza y retrocede, el foco no se pierde, el paso se anuncia, y el
  resultado mostrado coincide con el motor.
- **Práctica:** cada nivel genera sólo el rango que le corresponde; respuesta correcta e
  incorrecta; persistencia tras recarga.
- **Límites:** la calidad del método mental y la velocidad de un prodigio real **NO
  VERIFICADAS**; la app enseña el procedimiento y mide ejecución, no promete resultados.

## Estado de implementación

- **M-001 hecho:** `crates/engine/src/multiplication.rs` construye la multiplicación
  (`Multiplication::new`), expone los pasos (`PartialProduct`), el estado de varillas
  (`RodState`) y el generador por nivel (`MultiplicationDrill`, niveles M1…M7). Property
  tests con 10 000 casos por categoría: producto, reconstrucción desde los pasos, dos
  cifras por producto, posiciones, estado tras cada paso y conmutatividad.
- El orden implementado es **multiplicador por fuera, multiplicando por dentro**, y el
  registro de cada producto se hace siempre con dos cifras (decenas y unidades).
- **M-002 hecho:** `content/multiplication/{es,en}.ron` con la sección y las lecciones
  M0–M3; modelo tipado y tests de paridad y de producto exacto de cada ejemplo.
- **M-003 hecho:** componente `MultiplicationLearn` con lista de lecciones, explicación,
  ejemplo resuelto, visor de varillas y reproductor de pasos accesible por teclado; el
  paso se anuncia en `aria-live`.
- **M-004 hecho:** `MultiplicationPractice` con niveles M1–M7, ejercicios generados por
  semilla, corrección contra el producto exacto, feedback con el resultado esperado,
  aciertos/intentos y tiempo de respuesta. El campo queda en sólo lectura tras comprobar
  para conservar el foco con teclado.
- **M-005 hecho:** lecciones M4–M7 (2×2, 3×2, 3×3 y mental) y atajos S1–S4 con su condición
  de uso; `show_steps` desactiva el reproductor en mental y atajos.
- **M-006 hecho:** `MulProgress` con aciertos/intentos por nivel y serialización `v1`; claves
  propias `anzan.mul.level` y `anzan.mul.progress`, nivel saturado y progreso corrupto que
  vuelve a cero sin romper la UI. `crates/ui/src/storage.rs` comparte el acceso a
  `localStorage`/Tauri con el resto de la app.
- **M-007 hecho:** `aria-label` limpio en lecciones, `aria-pressed` en lecciones y niveles,
  feedback con `role="status"`, foco conservado tras comprobar y color de error por tema.
  Rendimiento medido: 1.000 transiciones del reproductor en 24,8 ms; sin `rAF` ni
  temporizadores en la sección.
- **Fase M cerrada técnicamente (2026-11-16).** Informe completo en
  `docs/INFORME-FASE-M.md`; publicación pendiente de autorización.

## Tareas propuestas (fase propia, antes de release)

| ID | Tarea | Aceptación |
|---|---|---|
| M-001 | `engine::multiplication`: modelo de pasos y producto | API determinista, límites y property tests ≥10 000; sin `unwrap`/`expect` |
| M-002 | Contenido M0–M3 es/en (tabla, 1×1, 2×1, 1×2) | Lecciones con ejemplo resuelto y tests de resultados |
| M-003 | UI «Aprender»: lista, lección y visor de varillas | Reproductor accesible por teclado; estado tras cada paso |
| M-004 | Práctica M1–M6 con generación y corrección | Niveles correctos, feedback y telemetría; tests |
| M-005 | Contenido M4–M6, mental y atajos | 2×2, 3×2, 3×3, mental y atajos con condiciones y ejemplos |
| M-006 | Persistencia y progreso de multiplicación | Claves propias, migración segura, recarga conserva |
| M-007 | Accesibilidad, i18n, rendimiento, CI e informe | `./ci.sh` verde, sin regresiones, límites declarados |

## Decisiones pendientes

1. **Prioridad:** ¿la multiplicación va antes de la fase de accesibilidad/rendimiento?
   Recomendación: sí, es el propósito central del producto.
2. **Alcance numérico:** ¿enteros hasta 3×3 cifras (recomendado para 1.0) o también
   decimales? Los decimales quedan fuera salvo pedido explícito.
3. **Atajos:** ¿se incluyen desde el inicio o como unidad avanzada opcional? Recomendación:
   unidad avanzada, después de dominar 2×2.
4. **Edad/experiencia:** ¿la sección asume que el usuario ya domina sumas en el soroban, o
   incluye un repaso? Recomendación: incluir un repaso breve y un test de entrada.

## Referencias consultadas

- 珠算乘法基本知识 (空盘前乘法, 大九九, 两位数记积法): <https://www.5068.com/zxs/129382.html>
- 世界珠算心算联合会, 课程设置 (乘法 con 空盘前乘法): <https://www.waama.org.cn/yanjiu/82.html>
- Wikibooks, *Traditional Abacus and Bead Arithmetic/Multiplication*:
  <https://en.wikibooks.org/wiki/Traditional_Abacus_and_Bead_Arithmetic/Multiplication>
- Kani Math, abacus multiplication (colocación por posición `i+j`):
  <https://www.kani-math.com/zh/learn/abacus-multiplication>

Los textos de las lecciones serán **originales** (contenido CC BY-SA 4.0); las fuentes se
citan para respaldar el método, no se copian.
