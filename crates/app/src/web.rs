// SPDX-License-Identifier: AGPL-3.0-or-later
//! Arranque de la versión web (wasm).
//!
//! `wasm-bindgen` ejecuta `start` al inicializar el módulo. No se usa `unwrap`: si falta
//! el documento o el RON no parsea, se devuelve un `JsValue` con el error y la consola
//! muestra el fallo.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::{Document, Window};

/// Arranca la app web.
///
/// # Errors
/// Devuelve un error JS si no hay `window`/`document` o si el contenido no parsea.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no hay window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("no hay document"))?;

    let locale = detect_locale(&window);
    let strings =
        anzan_content::load(locale).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let multiplication = anzan_content::load_multiplication(locale)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

    set_html_lang(&document, locale)?;

    if let Some(loading) = document.get_element_by_id("loading") {
        loading.remove();
    }

    anzan_ui::mount_to_body(strings, multiplication);
    Ok(())
}

/// Detecta el locale: query `?lang=`, luego localStorage, luego `navigator.language`.
#[cfg(target_arch = "wasm32")]
fn detect_locale(window: &Window) -> anzan_content::Locale {
    let query = window.location().search().unwrap_or_default();
    if query.contains("lang=en") {
        store_locale(window, anzan_content::Locale::En);
        return anzan_content::Locale::En;
    }
    if query.contains("lang=es") {
        store_locale(window, anzan_content::Locale::Es);
        return anzan_content::Locale::Es;
    }
    if let Some(stored) = stored_locale(window) {
        return stored;
    }
    let tag = window.navigator().language().unwrap_or_default();
    anzan_content::Locale::from_tag(&tag)
}

/// Locale guardado en `localStorage`, si existe y es válido.
#[cfg(target_arch = "wasm32")]
fn stored_locale(window: &Window) -> Option<anzan_content::Locale> {
    let storage = window.local_storage().ok().and_then(|option| option);
    let value = storage.and_then(|storage| storage.get_item("lang").ok().flatten());
    value.and_then(|value| match value.as_str() {
        "en" => Some(anzan_content::Locale::En),
        "es" => Some(anzan_content::Locale::Es),
        _ => None,
    })
}

/// Guarda el locale elegido para la próxima visita.
#[cfg(target_arch = "wasm32")]
fn store_locale(window: &Window, locale: anzan_content::Locale) {
    if let Ok(Some(storage)) = window.local_storage() {
        let _ = storage.set_item("lang", locale.code());
    }
}

/// Actualiza `<html lang="…">` para lectores de pantalla y SEO.
#[cfg(target_arch = "wasm32")]
fn set_html_lang(document: &Document, locale: anzan_content::Locale) -> Result<(), JsValue> {
    if let Some(element) = document.document_element() {
        element.set_attribute("lang", locale.code())?;
    }
    Ok(())
}
