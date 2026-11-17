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

    #[test]
    fn locale_detection_falls_back_to_spanish() {
        assert_eq!(Locale::from_tag("es-MX"), Locale::Es);
        assert_eq!(Locale::from_tag("en-US"), Locale::En);
        assert_eq!(Locale::from_tag("fr-FR"), Locale::Es);
        assert_eq!(Locale::from_tag(""), Locale::Es);
    }
}
