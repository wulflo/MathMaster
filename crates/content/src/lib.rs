// SPDX-License-Identifier: AGPL-3.0-or-later
//! Contenido de MathMaster.
//!
//! Todo texto de UI y todo contenido pedagógico vive en RON y se valida al cargarlo.
//! Ningún componente debe hardcodear una cadena visible.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::Deserialize;

/// Textos de la interfaz para un locale.
///
/// `deny_unknown_fields` hace que un RON con claves de más falle en vez de ignorarse.
/// La paridad entre locales se comprueba en los tests: ambos deben deserializar el mismo
/// conjunto de campos.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiStrings {
    /// Nombre de la aplicación.
    pub app_name: String,
    /// Lema visual corto (kicker).
    pub kicker: String,
    /// Lema corto.
    pub tagline: String,
    /// Etiqueta del selector de color de acento.
    pub accent_picker_label: String,
    /// Nombres de los diez acentos, en orden.
    pub accent_names: Vec<String>,
    /// Etiqueta del selector de tema claro/oscuro.
    pub theme_label: String,
    /// Nombre del tema claro.
    pub theme_light: String,
    /// Nombre del tema oscuro.
    pub theme_dark: String,
    /// Etiqueta del selector de idioma.
    pub language: String,
    /// Nombre del idioma español.
    pub spanish: String,
    /// Nombre del idioma inglés.
    pub english: String,
    /// Etiqueta compacta del español en la cabecera.
    pub lang_es: String,
    /// Etiqueta compacta del inglés en la cabecera.
    pub lang_en: String,
    /// Texto de carga inicial.
    pub loading: String,
    /// Etiqueta del enlace al código fuente.
    pub source_label: String,
    /// Etiqueta del enlace a la licencia.
    pub license_label: String,
    /// Separador visual de enlaces.
    pub separator: String,
    /// Título del panel del soroban.
    pub soroban_label: String,
    /// Etiqueta del valor actual.
    pub value_label: String,
    /// Texto cuando la lectura del valor está oculta.
    pub value_hidden: String,
    /// Acción para mostrar el valor.
    pub show_value: String,
    /// Acción para ocultar el valor.
    pub hide_value: String,
    /// Acción para reiniciar el soroban.
    pub reset_label: String,
    /// Etiqueta del control de fading.
    pub fade_label: String,
    /// Nombre de la etapa completa.
    pub stage_full: String,
    /// Nombre de la etapa fantasma.
    pub stage_ghost: String,
    /// Nombre de la etapa de varillas.
    pub stage_rods: String,
    /// Nombre de la etapa de marcas de unidad.
    pub stage_unit_marks: String,
    /// Nombre de la etapa en blanco.
    pub stage_blank: String,
    /// Título del modo Flash Anzan.
    pub drill_title: String,
    /// Instrucciones del modo Flash Anzan.
    pub drill_intro: String,
    /// Etiqueta del selector de dígitos.
    pub drill_digits: String,
    /// Etiqueta del selector de términos.
    pub drill_terms: String,
    /// Etiqueta del interruptor de sonido.
    pub drill_sound: String,
    /// Acción para empezar un drill.
    pub drill_start: String,
    /// Etiqueta del campo de respuesta.
    pub drill_answer: String,
    /// Acción para comprobar la respuesta.
    pub drill_submit: String,
    /// Resultado correcto.
    pub drill_correct: String,
    /// Resultado incorrecto.
    pub drill_wrong: String,
    /// Etiqueta de la respuesta esperada.
    pub drill_expected: String,
    /// Estado mientras se muestra la secuencia.
    pub drill_running: String,
    /// Estado antes de empezar.
    pub drill_ready: String,
    /// Título del panel de progresión.
    pub progress_title: String,
    /// Etiqueta del nivel actual.
    pub progress_level: String,
    /// Etiqueta de la precisión acumulada.
    pub progress_accuracy: String,
    /// Etiqueta de la racha actual.
    pub progress_streak: String,
    /// Etiqueta de los intentos totales.
    pub progress_attempts: String,
    /// Etiqueta de la mejor racha.
    pub progress_best: String,
    /// Acción para reiniciar el progreso.
    pub progress_reset: String,
    /// Aviso de subida de nivel.
    pub progress_promoted: String,
    /// Aviso de bajada de nivel.
    pub progress_demoted: String,
    /// Etiqueta del modo de progresión automática.
    pub progress_mode: String,
    /// Etiqueta de la práctica libre.
    pub progress_free: String,
    /// Título del historial de sesiones.
    pub session_title: String,
    /// Texto cuando no hay sesiones.
    pub session_empty: String,
    /// Acción para exportar el historial.
    pub session_export: String,
    /// Etiqueta del tiempo empleado.
    pub session_elapsed: String,
    /// Etiqueta del caso pedagógico.
    pub session_case: String,
    /// Nombre del caso de movimiento directo.
    pub case_direct: String,
    /// Nombre del caso de complemento a 5.
    pub case_complement5: String,
    /// Nombre del caso de complemento a 10.
    pub case_complement10: String,
    /// Nombre del caso de acarreo en cadena.
    pub case_carry: String,
    /// Título de las estadísticas por caso.
    pub case_stats_title: String,
    /// Etiqueta de la precisión por caso.
    pub case_stats_accuracy: String,
    /// Título del panel de repaso.
    pub review_title: String,
    /// Texto cuando toca repasar.
    pub review_now: String,
    /// Plantilla de repaso en horas.
    pub review_hours: String,
    /// Plantilla de repaso en días.
    pub review_days: String,
    /// Título de la lista de lecciones de multiplicación.
    pub mul_lessons: String,
    /// Etiqueta del objetivo de una lección.
    pub mul_goal: String,
    /// Etiqueta del ejemplo resuelto.
    pub mul_example: String,
    /// Texto del ábaco vacío antes del primer paso.
    pub mul_empty_abacus: String,
    /// Plantilla del número de paso.
    pub mul_step_of: String,
    /// Plantilla de un producto parcial y su acumulado.
    pub mul_step_product: String,
    /// Plantilla del resultado final.
    pub mul_result: String,
    /// Acción para volver al paso anterior.
    pub mul_prev: String,
    /// Acción para avanzar al paso siguiente.
    pub mul_next: String,
    /// Acción para volver al inicio de los pasos.
    pub mul_replay: String,
    /// Etiqueta accesible de las varillas del resultado.
    pub mul_rods_label: String,
    /// Título del diagrama del ábaco.
    pub mul_abacus_title: String,
    /// Título de la tabla 大九九.
    pub mul_table_title: String,
    /// Texto que explica cómo leer la tabla 大九九.
    pub mul_table_hint: String,
    /// Título de la tabla de productos parciales.
    pub mul_steps_title: String,
    /// Columna «paso» de la tabla de productos parciales.
    pub mul_col_step: String,
    /// Columna «producto» de la tabla de productos parciales.
    pub mul_col_product: String,
    /// Columna «registro» de la tabla de productos parciales.
    pub mul_col_record: String,
    /// Columna «acumulado» de la tabla de productos parciales.
    pub mul_col_total: String,
    /// Título del bloque de práctica de multiplicación.
    pub mul_practice_title: String,
    /// Instrucciones del bloque de práctica.
    pub mul_practice_intro: String,
    /// Etiqueta del selector de nivel de práctica.
    pub mul_practice_level: String,
    /// Etiqueta del campo de respuesta.
    pub mul_practice_answer: String,
    /// Acción para comprobar la respuesta.
    pub mul_practice_check: String,
    /// Acción para pedir otro ejercicio.
    pub mul_practice_new: String,
    /// Resultado correcto de la práctica.
    pub mul_correct: String,
    /// Plantilla del resultado incorrecto.
    pub mul_wrong: String,
    /// Plantilla de aciertos y intentos.
    pub mul_practice_stats: String,
    /// Plantilla del tiempo de respuesta.
    pub mul_practice_elapsed: String,
    /// Sufijo del nivel mental.
    pub mul_mental: String,

    // --- REDESENO: navegación y cabecera ---
    /// Etiqueta accesible de la barra de pestañas principales.
    pub tablist_label: String,
    /// Pestaña Aprender.
    pub tab_learn: String,
    /// Pestaña Laboratorio (soroban libre).
    pub tab_lab: String,
    /// Pestaña Entrenamiento.
    pub tab_train: String,
    /// Pestaña Progreso.
    pub tab_progress: String,
    /// Subsección de fundamentos del soroban.
    pub sub_fundamentals: String,
    /// Subsección de multiplicación frontal.
    pub sub_multiplication: String,
    /// Subsección de atajos mentales.
    pub sub_shortcuts: String,
    /// Subsección de la tabla 大九九.
    pub sub_table99: String,
    /// Subsección de ábaco interactivo.
    pub sub_soroban: String,
    /// Subsección de demostrador de movimientos.
    pub sub_movements: String,
    /// Subsección de Flash Anzan.
    pub sub_flash: String,
    /// Subsección de Mitori Anzan.
    pub sub_mitori: String,
    /// Subsección de multiplicación por niveles.
    pub sub_mul_training: String,
    /// Subsección de entrenamiento dirigido.
    pub sub_directed: String,
    /// Subsección de panel de dominio.
    pub sub_mastery: String,
    /// Subsección de repaso espaciado.
    pub sub_review: String,
    /// Subsección de diagnóstico e historial.
    pub sub_diagnostics: String,
    /// Etiqueta accesible de una barra de subpestañas.
    pub subtablist_label: String,
    /// Nombre del método de multiplicación frontal.
    pub mul_method_name: String,
    /// Cabecera «operación» de las tablas de fórmulas.
    pub formula_operation: String,
    /// Cabecera «equivalencia» de las tablas de fórmulas.
    pub formula_equation: String,
    /// Cabecera «nota» de las tablas de fórmulas.
    pub formula_note: String,
    /// Etiqueta del fundamento matemático de una lección.
    pub mul_foundation: String,
    /// Etiqueta de la regla de posicionamiento de varillas.
    pub mul_positioning: String,
    /// Etiqueta del procedimiento detallado.
    pub mul_procedure: String,
    /// Etiqueta de las trampas habituales.
    pub mul_pitfalls: String,
    /// Etiqueta del ejemplo resuelto y comentado.
    pub mul_annotated: String,
    /// Leyenda del esquema: barra divisoria.
    pub diagram_hari: String,
    /// Leyenda del esquema: cuenta celestial.
    pub diagram_go_dama: String,
    /// Leyenda del esquema: cuentas terrestres.
    pub diagram_ichi_dama: String,
    /// Leyenda del esquema: punto de unidad.
    pub diagram_teii_ten: String,
    /// Leyenda del esquema: movimiento del pulgar.
    pub diagram_thumb: String,
    /// Leyenda del esquema: movimiento del índice.
    pub diagram_index: String,
    /// Leyenda del esquema: acarreo.
    pub diagram_carry: String,
    /// Leyenda del esquema: familia de complementos mixtos.
    pub diagram_mixed: String,
    /// Leyenda del esquema: antes de aplicar la regla.
    pub diagram_before: String,
    /// Leyenda del esquema: después de aplicar la regla.
    pub diagram_after: String,

    // --- REDESENO: respuesta ultrarrápida ---
    /// Interruptor de avance automático al acertar.
    pub answer_auto_advance: String,
    /// Pista bajo el campo de respuesta.
    pub answer_hint: String,
    /// Acción para pasar al siguiente ejercicio.
    pub answer_next: String,
    /// Botón «Ver en el soroban» tras un fallo.
    pub answer_see_soroban: String,
    /// Etiqueta del teclado numérico opcional.
    pub answer_keypad: String,
    /// Acción de borrar la respuesta escrita.
    pub answer_clear: String,
    /// Acción de repetir la misma secuencia.
    pub answer_repeat: String,

    // --- REDESENO: Flash Anzan ampliado ---
    /// Texto de la cuenta regresiva de fijación (`{n}`).
    pub drill_countdown: String,
    /// Instrucción durante la cuenta regresiva.
    pub drill_fixation: String,
    /// Interruptor de operaciones mixtas (sumas y restas).
    pub drill_mixed: String,
    /// Etiqueta del tiempo de exposición por término en milisegundos.
    pub drill_exposure: String,
    /// Etiqueta del hueco entre términos en milisegundos.
    pub drill_gap: String,
    /// Frames equivalentes a la configuración actual (`{n}`).
    pub drill_frames: String,
    /// Cadencia en términos por segundo (`{n}`).
    pub drill_cadence: String,
    /// Signo menos usado en los términos sustractivos.
    pub minus_sign: String,

    // --- REDESENO: laboratorio y demostrador ---
    /// Etiqueta del selector de varillas.
    pub lab_rods: String,
    /// Título del demostrador de movimientos.
    pub demo_title: String,
    /// Valor inicial de la operación.
    pub demo_start: String,
    /// Selector de operación: sumar.
    pub demo_add: String,
    /// Selector de operación: restar.
    pub demo_sub: String,
    /// Cantidad a sumar o restar.
    pub demo_amount: String,
    /// Acción de generar los pasos.
    pub demo_run: String,
    /// Plantilla de paso del demostrador (`{n}` de `{total}`).
    pub demo_step: String,
    /// Etiqueta de la regla aplicada en un paso.
    pub demo_rule: String,
    /// Regla de movimiento directo.
    pub rule_direct: String,
    /// Regla de complemento a 5.
    pub rule_complement5: String,
    /// Regla de complemento a 10.
    pub rule_complement10: String,
    /// Regla de acarreo en cadena.
    pub rule_carry: String,
    /// Regla de préstamo en cadena.
    pub rule_borrow: String,
    /// Resultado de la operación demonstrada (`{value}`).
    pub demo_result: String,
    /// Aviso de operación no válida.
    pub demo_invalid: String,

    // --- REDESENO: Mitori Anzan ---
    /// Título del modo Mitori Anzan.
    pub mitori_title: String,
    /// Introducción del modo Mitori Anzan.
    pub mitori_intro: String,
    /// Etiqueta del número de dígitos por fila.
    pub mitori_digits: String,
    /// Etiqueta del número de filas.
    pub mitori_rows: String,
    /// Etiqueta del tiempo límite en segundos.
    pub mitori_limit: String,
    /// Acción para mostrar la columna.
    pub mitori_start: String,
    /// Interruptor de lectura guiada.
    pub mitori_guided: String,
    /// Tiempo transcurrido (`{ms}`).
    pub mitori_timer: String,
    /// Acción de revelar la siguiente fila.
    pub mitori_next_row: String,
    /// Etiqueta del campo de suma de la columna.
    pub mitori_answer: String,

    // --- REDESENO: entrenamiento dirigido y analítica ---
    /// Título del entrenamiento dirigido.
    pub directed_title: String,
    /// Introducción del entrenamiento dirigido.
    pub directed_intro: String,
    /// Caso débil detectado (`{case}`).
    pub directed_weakest: String,
    /// Aviso cuando aún no hay datos suficientes.
    pub directed_no_data: String,
    /// Título del panel de dominio.
    pub mastery_title: String,
    /// Objetivo hacia el siguiente nivel (`{n}`).
    pub mastery_goal: String,
    /// Plantilla del estado de repaso de un nivel (`{level}`, `{state}`).
    pub review_detail: String,
    /// Acción de lanzar el repaso pendiente.
    pub review_start: String,
    /// Título del diagnóstico por regla.
    pub diagnostics_title: String,
    /// Título de la tabla de últimas sesiones.
    pub diagnostics_history: String,
}

/// Contenido de la sección «Aprender a multiplicar» para un locale.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiplicationContent {
    /// Título de la sección.
    pub title: String,
    /// Introducción breve al método.
    pub intro: String,
    /// Lecciones en orden de dificultad.
    pub lessons: Vec<MultiplicationLesson>,
    /// Manual práctico de fundamentos del soroban (capítulos 1–6).
    ///
    /// Vive junto a la multiplicación en el RON de contenido pedagógico para que todo texto
    /// visible siga residiendo en los cuatro ficheros validados por los tests.
    pub fundamentals: FundamentalsContent,
}

/// Manual práctico de fundamentos del soroban y del paso al cálculo mental.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FundamentalsContent {
    /// Título del manual.
    pub title: String,
    /// Introducción del manual.
    pub intro: String,
    /// Capítulos en orden de estudio.
    pub chapters: Vec<FundamentalsChapter>,
}

/// Un capítulo del manual de fundamentos.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FundamentalsChapter {
    /// Identificador estable (`C1`…`C6`).
    pub id: String,
    /// Título del capítulo.
    pub title: String,
    /// Objetivo verificable del capítulo.
    pub goal: String,
    /// Secciones que componen el capítulo.
    pub sections: Vec<FundamentalsSection>,
}

/// Una sección de un capítulo: prosa, fórmulas, pasos guiados y trampas.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FundamentalsSection {
    /// Encabezado de la sección.
    pub heading: String,
    /// Párrafos explicativos.
    pub paragraphs: Vec<String>,
    /// Tabla de fórmulas y sus inversiones, si la sección la tiene.
    #[serde(default)]
    pub formulas: Vec<FormulaRow>,
    /// Procedimiento guiado paso a paso, si aplica.
    #[serde(default)]
    pub steps: Vec<String>,
    /// Errores frecuentes a evitar.
    #[serde(default)]
    pub tips: Vec<String>,
    /// Identificador del esquema vectorial que ilustra la sección, si tiene.
    ///
    /// Valores: `anatomy`, `reading`, `fingering`, `complement5`, `complement10`,
    /// `mixed`, `fading`. La UI decide cómo dibujarlo; el RON sólo elige cuál mostrar.
    #[serde(default)]
    pub diagram: Option<String>,
}

/// Una fila de una tabla de fórmulas del manual.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormulaRow {
    /// Operación que se quiere resolver (`+4`, `−4`, …).
    pub label: String,
    /// Equivalencia mecanográfica en el ábaco (`+4 = +5 − 1`).
    pub equation: String,
    /// Condición o nota didáctica.
    pub note: String,
}

/// Una lección de multiplicación.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiplicationLesson {
    /// Identificador del nivel (`M0`…`M6`).
    pub id: String,
    /// Título de la lección.
    pub title: String,
    /// Objetivo verificable de la lección.
    pub goal: String,
    /// Explicación en párrafos cortos.
    pub paragraphs: Vec<String>,
    /// Ejemplo resuelto que la UI reproduce paso a paso.
    pub example: WorkedExample,
    /// Si la lección muestra el reproductor de pasos del método general.
    ///
    /// Las lecciones mentales y los atajos lo desactivan: su explicación no sigue la
    /// secuencia de productos de la 空盘前乘法.
    #[serde(default = "default_true")]
    pub show_steps: bool,
    /// Fundamento matemático de la lección.
    #[serde(default)]
    pub foundation: String,
    /// Regla de posicionamiento de varillas (*Teii-hō* / 定位法).
    #[serde(default)]
    pub positioning: String,
    /// Procedimiento detallado paso a paso.
    #[serde(default)]
    pub procedure: Vec<String>,
    /// Trampas habituales: ceros, acarreos cruzados, cero inicial.
    #[serde(default)]
    pub pitfalls: Vec<String>,
    /// Ejemplo resuelto y comentado, línea a línea.
    #[serde(default)]
    pub annotated_example: Vec<String>,
}

/// Valor por defecto de [`MultiplicationLesson::show_steps`].
const fn default_true() -> bool {
    true
}

/// Ejemplo resuelto de una lección.
///
/// Los números son datos, no texto: los tests comprueban que `expected` sea el producto
/// exacto y la UI los pasa al motor para construir los pasos.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkedExample {
    /// Multiplicando (el número cuyas cifras se barren en cada paso).
    pub multiplicand: u32,
    /// Multiplicador (el número que fija el orden de los pasos).
    pub multiplier: u32,
    /// Producto esperado, comprobado por test.
    pub expected: u64,
    /// Ejemplo escrito, por ejemplo `23 × 45 = 1035`.
    pub caption: String,
    /// Cierre que explica el resultado en el ábaco.
    pub conclusion: String,
}

/// Locales soportados en 1.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    /// Español.
    Es,
    /// Inglés.
    En,
}

impl Locale {
    /// Código BCP-47 corto.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Es => "es",
            Self::En => "en",
        }
    }

    /// Detecta el locale a partir de una etiqueta tipo `es-MX` o `en-US`.
    ///
    /// El fallback es español: el producto nace en español y no queremos una UI vacía
    /// ante un locale desconocido.
    #[must_use]
    pub fn from_tag(tag: &str) -> Self {
        let primary = tag.split(['-', '_']).next().unwrap_or("es");
        if primary.eq_ignore_ascii_case("en") {
            Self::En
        } else {
            Self::Es
        }
    }
}

const ES_RON: &str = include_str!("../content/i18n/es.ron");
const EN_RON: &str = include_str!("../content/i18n/en.ron");
const MULTIPLICATION_ES_RON: &str = include_str!("../content/multiplication/es.ron");
const MULTIPLICATION_EN_RON: &str = include_str!("../content/multiplication/en.ron");

/// Error al cargar contenido.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ContentError {
    /// El RON no se pudo parsear o no cumple el esquema.
    #[error("contenido inválido para el locale '{locale}': {detail}")]
    Parse {
        /// Locale que falló.
        locale: &'static str,
        /// Mensaje del parser.
        detail: String,
    },
}

/// Carga los textos de UI de un locale.
///
/// # Errors
/// Devuelve [`ContentError::Parse`] si el RON embebido no cumple el esquema.
pub fn load(locale: Locale) -> Result<UiStrings, ContentError> {
    let (code, source) = match locale {
        Locale::Es => ("es", ES_RON),
        Locale::En => ("en", EN_RON),
    };
    ron::from_str(source).map_err(|error| ContentError::Parse {
        locale: code,
        detail: error.to_string(),
    })
}

/// Carga el contenido de multiplicación de un locale.
///
/// # Errors
/// Devuelve [`ContentError::Parse`] si el RON embebido no cumple el esquema.
pub fn load_multiplication(locale: Locale) -> Result<MultiplicationContent, ContentError> {
    let (code, source) = match locale {
        Locale::Es => ("es", MULTIPLICATION_ES_RON),
        Locale::En => ("en", MULTIPLICATION_EN_RON),
    };
    ron::from_str(source).map_err(|error| ContentError::Parse {
        locale: code,
        detail: error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_locales_parse_and_are_not_empty() {
        for locale in [Locale::Es, Locale::En] {
            let strings = load(locale);
            assert!(strings.is_ok(), "el locale {} debe parsear", locale.code());
            if let Ok(strings) = strings {
                assert!(!strings.app_name.trim().is_empty());
                assert!(!strings.tagline.trim().is_empty());
                assert!(!strings.loading.trim().is_empty());
            }
        }
    }

    #[test]
    fn multiplication_content_parses_in_both_locales() {
        for locale in [Locale::Es, Locale::En] {
            let content = load_multiplication(locale);
            assert!(
                content.is_ok(),
                "el contenido de multiplicación de {} debe parsear",
                locale.code()
            );
            if let Ok(content) = content {
                assert!(!content.title.trim().is_empty());
                assert!(!content.intro.trim().is_empty());
                assert!(!content.lessons.is_empty());
                for lesson in &content.lessons {
                    assert!(!lesson.id.trim().is_empty());
                    assert!(!lesson.title.trim().is_empty());
                    assert!(!lesson.goal.trim().is_empty());
                    assert!(!lesson.paragraphs.is_empty());
                    assert!(lesson.paragraphs.iter().all(|text| !text.trim().is_empty()));
                    assert!(!lesson.example.caption.trim().is_empty());
                    assert!(!lesson.example.conclusion.trim().is_empty());
                }
            }
        }
    }

    #[test]
    fn multiplication_locales_share_structure_and_numbers() {
        let es = load_multiplication(Locale::Es);
        let en = load_multiplication(Locale::En);
        assert!(es.is_ok() && en.is_ok());
        if let (Ok(es), Ok(en)) = (es, en) {
            assert_eq!(es.lessons.len(), en.lessons.len());
            for (spanish, english) in es.lessons.iter().zip(en.lessons.iter()) {
                assert_eq!(spanish.id, english.id);
                assert_eq!(spanish.example.multiplicand, english.example.multiplicand);
                assert_eq!(spanish.example.multiplier, english.example.multiplier);
                assert_eq!(spanish.example.expected, english.example.expected);
            }
        }
    }

    #[test]
    fn multiplication_examples_are_exact_and_ordered() {
        for locale in [Locale::Es, Locale::En] {
            let content = load_multiplication(locale);
            assert!(content.is_ok());
            if let Ok(content) = content {
                let ids: Vec<&str> = content
                    .lessons
                    .iter()
                    .map(|lesson| lesson.id.as_str())
                    .collect();
                assert_eq!(
                    ids,
                    vec![
                        "M0", "M1", "M2", "M3", "M4", "M5", "M6", "M7", "S1", "S2", "S3", "S4"
                    ]
                );
                assert!(content.lessons.iter().any(|lesson| lesson.show_steps));
                assert!(content.lessons.iter().any(|lesson| !lesson.show_steps));
                for lesson in &content.lessons {
                    if lesson.id.starts_with('S') || lesson.id == "M7" {
                        assert!(!lesson.show_steps, "{} no usa el reproductor", lesson.id);
                    } else {
                        assert!(lesson.show_steps, "{} usa el reproductor", lesson.id);
                    }
                }
                for lesson in &content.lessons {
                    let example = &lesson.example;
                    assert_eq!(
                        example.expected,
                        u64::from(example.multiplicand) * u64::from(example.multiplier),
                        "el ejemplo {} debe dar el producto exacto",
                        lesson.id
                    );
                    assert!(example.multiplicand <= 999);
                    assert!(example.multiplier <= 999);
                }
            }
        }
    }

    /// Cuenta las claves de primer nivel de un RON de textos de UI.
    ///
    /// `deny_unknown_fields` ya garantiza que ambos locales tienen exactamente las claves
    /// del struct; este recuento hace explícita la paridad en el test.
    fn top_level_keys(source: &str) -> usize {
        source
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                line.len() - trimmed.len() == 4
                    && trimmed
                        .split(':')
                        .next()
                        .is_some_and(|key| key.chars().all(|c| c.is_ascii_lowercase() || c == '_'))
            })
            .count()
    }

    #[test]
    fn ui_locales_have_exactly_the_same_keys() {
        assert_eq!(top_level_keys(ES_RON), top_level_keys(EN_RON));
        for locale in [Locale::Es, Locale::En] {
            let strings = load(locale);
            assert!(strings.is_ok(), "el locale {} debe parsear", locale.code());
            if let Ok(strings) = strings {
                assert!(!strings.tablist_label.trim().is_empty());
                assert!(!strings.answer_auto_advance.trim().is_empty());
                assert!(strings.drill_countdown.contains("{n}"));
                assert!(!strings.accent_names.is_empty());
            }
        }
    }

    #[test]
    fn fundamentals_content_parses_with_six_chapters_in_both_locales() {
        let mut snapshots = Vec::new();
        for locale in [Locale::Es, Locale::En] {
            let content = load_multiplication(locale);
            assert!(
                content.is_ok(),
                "los fundamentos de {} deben parsear",
                locale.code()
            );
            if let Ok(content) = content {
                let manual = &content.fundamentals;
                assert!(!manual.title.trim().is_empty());
                assert!(!manual.intro.trim().is_empty());
                let ids: Vec<&str> = manual
                    .chapters
                    .iter()
                    .map(|chapter| chapter.id.as_str())
                    .collect();
                assert_eq!(ids, vec!["C1", "C2", "C3", "C4", "C5", "C6"]);
                for chapter in &manual.chapters {
                    assert!(!chapter.title.trim().is_empty());
                    assert!(!chapter.goal.trim().is_empty());
                    assert!(!chapter.sections.is_empty());
                    for section in &chapter.sections {
                        assert!(!section.heading.trim().is_empty());
                        assert!(!section.paragraphs.is_empty());
                        assert!(
                            section
                                .paragraphs
                                .iter()
                                .all(|text| !text.trim().is_empty())
                        );
                        for row in &section.formulas {
                            assert!(!row.label.trim().is_empty());
                            assert!(!row.equation.trim().is_empty());
                            assert!(!row.note.trim().is_empty());
                        }
                        assert!(section.steps.iter().all(|text| !text.trim().is_empty()));
                        assert!(section.tips.iter().all(|text| !text.trim().is_empty()));
                    }
                }
                snapshots.push(
                    manual
                        .chapters
                        .iter()
                        .map(|chapter| {
                            (
                                chapter.id.clone(),
                                chapter.sections.len(),
                                chapter
                                    .sections
                                    .iter()
                                    .map(|section| section.formulas.len())
                                    .collect::<Vec<_>>(),
                            )
                        })
                        .collect::<Vec<_>>(),
                );
            }
        }
        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots[0], snapshots[1]);
    }

    #[test]
    fn sections_reference_known_vector_diagrams_in_both_locales() {
        let known = [
            "anatomy",
            "reading",
            "fingering",
            "complement5",
            "complement10",
            "mixed",
            "fading",
        ];
        let mut shapes = Vec::new();
        for locale in [Locale::Es, Locale::En] {
            let content = load_multiplication(locale);
            assert!(content.is_ok());
            if let Ok(content) = content {
                let mut codes = Vec::new();
                for chapter in &content.fundamentals.chapters {
                    for section in &chapter.sections {
                        if let Some(diagram) = &section.diagram {
                            assert!(
                                known.contains(&diagram.as_str()),
                                "diagrama desconocido: {diagram}"
                            );
                            codes.push(diagram.clone());
                        }
                    }
                }
                assert_eq!(codes.len(), known.len());
                shapes.push(codes);
            }
        }
        assert_eq!(shapes.len(), 2);
        assert_eq!(shapes[0], shapes[1]);
    }

    #[test]
    fn lessons_include_deep_pedagogy_in_both_locales() {
        let mut shapes = Vec::new();
        for locale in [Locale::Es, Locale::En] {
            let content = load_multiplication(locale);
            assert!(content.is_ok());
            if let Ok(content) = content {
                let mut shape = Vec::new();
                for lesson in &content.lessons {
                    assert!(
                        !lesson.foundation.trim().is_empty(),
                        "{} sin fundamento",
                        lesson.id
                    );
                    assert!(
                        !lesson.positioning.trim().is_empty(),
                        "{} sin Teii-hō",
                        lesson.id
                    );
                    assert!(
                        !lesson.procedure.is_empty(),
                        "{} sin procedimiento",
                        lesson.id
                    );
                    assert!(!lesson.pitfalls.is_empty(), "{} sin trampas", lesson.id);
                    assert!(
                        !lesson.annotated_example.is_empty(),
                        "{} sin ejemplo comentado",
                        lesson.id
                    );
                    assert!(lesson.procedure.iter().all(|text| !text.trim().is_empty()));
                    assert!(lesson.pitfalls.iter().all(|text| !text.trim().is_empty()));
                    assert!(
                        lesson
                            .annotated_example
                            .iter()
                            .all(|text| !text.trim().is_empty())
                    );
                    shape.push((
                        lesson.id.clone(),
                        lesson.procedure.len(),
                        lesson.pitfalls.len(),
                        lesson.annotated_example.len(),
                    ));
                }
                shapes.push(shape);
            }
        }
        assert_eq!(shapes.len(), 2);
        assert_eq!(shapes[0], shapes[1]);
    }

    #[test]
    fn locale_detection_falls_back_to_spanish() {
        assert_eq!(Locale::from_tag("es-MX"), Locale::Es);
        assert_eq!(Locale::from_tag("en-US"), Locale::En);
        assert_eq!(Locale::from_tag("fr-FR"), Locale::Es);
        assert_eq!(Locale::from_tag(""), Locale::Es);
    }
}
