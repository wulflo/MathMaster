// SPDX-License-Identifier: AGPL-3.0-or-later
//! Acceso a la persistencia del host.
//!
//! En la web se usa `localStorage`. Dentro de Tauri se lee el snapshot de disco que el
//! host inyecta como `window.__ANZAN_STORE__` y se escribe con `invoke("store_set")`,
//! porque la webview no comparte `localStorage` con el proceso de escritorio. En builds
//! nativos sin navegador las funciones no hacen nada.

#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

/// Snapshot de disco inyectado por el host Tauri, si existe.
#[cfg(target_arch = "wasm32")]
fn native_store() -> Option<wasm_bindgen::JsValue> {
    use wasm_bindgen::JsValue;

    let window = web_sys::window()?;
    let store = js_sys::Reflect::get(&window, &JsValue::from_str("__ANZAN_STORE__")).ok()?;
    if store.is_undefined() || store.is_null() {
        None
    } else {
        Some(store)
    }
}

/// Lee una clave: primero el snapshot nativo, después `localStorage`.
#[cfg(target_arch = "wasm32")]
pub(crate) fn read_key(key: &str) -> Option<String> {
    use wasm_bindgen::JsValue;

    if let Some(store) = native_store() {
        return js_sys::Reflect::get(&store, &JsValue::from_str(key))
            .ok()
            .and_then(|value| value.as_string());
    }
    web_sys::window()?
        .local_storage()
        .ok()
        .flatten()?
        .get_item(key)
        .ok()
        .flatten()
}

/// Escribe una clave en `localStorage` y, si hay host nativo, también en su store.
#[cfg(target_arch = "wasm32")]
pub(crate) fn write_key(key: &str, value: &str) {
    if let Some(window) = web_sys::window()
        && let Some(storage) = window.local_storage().ok().flatten()
    {
        let _ = storage.set_item(key, value);
    }
    if native_store().is_some() {
        let _ = tauri_invoke_with("store_set", &[("key", key), ("value", value)]);
    }
}

/// Sin navegador no hay persistencia.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn read_key(_key: &str) -> Option<String> {
    None
}

/// Sin navegador no hay persistencia.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn write_key(_key: &str, _value: &str) {}

/// Invoca un comando de Tauri sin argumentos, si el host está presente.
#[cfg(target_arch = "wasm32")]
pub(crate) fn tauri_invoke(command: &str) -> Option<js_sys::Promise> {
    tauri_invoke_with(command, &[])
}

/// Invoca un comando de Tauri con argumentos de texto.
#[cfg(target_arch = "wasm32")]
pub(crate) fn tauri_invoke_with(
    command: &str,
    arguments: &[(&str, &str)],
) -> Option<js_sys::Promise> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;

    let window = web_sys::window()?;
    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")).ok()?;
    if tauri.is_undefined() || tauri.is_null() {
        return None;
    }
    let core = js_sys::Reflect::get(&tauri, &JsValue::from_str("core")).ok()?;
    let invoke = js_sys::Reflect::get(&core, &JsValue::from_str("invoke")).ok()?;
    let function: js_sys::Function = invoke.dyn_into().ok()?;
    let arguments_value = js_sys::Object::new();
    for (name, value) in arguments {
        let _ = js_sys::Reflect::set(
            &arguments_value,
            &JsValue::from_str(name),
            &JsValue::from_str(value),
        );
    }
    let promise = function
        .call2(&core, &JsValue::from_str(command), &arguments_value)
        .ok()?;
    promise.dyn_into().ok()
}
