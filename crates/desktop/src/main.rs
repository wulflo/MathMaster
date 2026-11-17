// SPDX-License-Identifier: AGPL-3.0-or-later
//! Host de escritorio de MathMaster.
//!
//! Tauri abre la misma UI Leptos compilada a wasm (`dist/`) y aporta lo que la web no
//! puede: audio nativo cpal, reloj de audio y persistencia en disco de la app.

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod audio;
mod measure;
mod selftest;
mod store;

use std::sync::Mutex;

use audio::{AudioHandle, AudioInfo, AudioStats};
use store::Store;
use tauri::Manager;

/// Estado del host compartido por los comandos Tauri.
struct HostState {
    audio: Mutex<Option<AudioHandle>>,
    store: Mutex<Store>,
}

impl HostState {
    fn new(store: Store) -> Self {
        Self {
            audio: Mutex::new(None),
            store: Mutex::new(store),
        }
    }
}

/// Arranca el audio nativo o devuelve la información si ya estaba activo.
///
/// # Errors
/// Devuelve un mensaje si el dispositivo no se puede abrir.
#[tauri::command]
fn audio_start(state: tauri::State<'_, HostState>) -> Result<AudioInfo, String> {
    let mut guard = state
        .audio
        .lock()
        .map_err(|_| "estado de audio envenenado".to_owned())?;
    if let Some(handle) = guard.as_ref() {
        return Ok(handle.info().clone());
    }
    let handle = audio::start()?;
    let info = handle.info().clone();
    *guard = Some(handle);
    Ok(info)
}

/// Encola un clic en el siguiente bloque de audio.
#[tauri::command]
fn audio_click(state: tauri::State<'_, HostState>) -> bool {
    state
        .audio
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(AudioHandle::click))
        .unwrap_or(false)
}

/// Estadísticas del callback y reloj de audio.
#[tauri::command]
fn audio_stats(state: tauri::State<'_, HostState>) -> Option<AudioStats> {
    state
        .audio
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(AudioHandle::stats))
}

/// Reloj de audio en milisegundos.
#[tauri::command]
fn audio_clock_ms(state: tauri::State<'_, HostState>) -> Option<f64> {
    state
        .audio
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(AudioHandle::clock_ms))
}

/// Lee una clave del almacén en disco.
#[tauri::command]
fn store_get(state: tauri::State<'_, HostState>, key: String) -> Option<String> {
    state.store.lock().ok().and_then(|store| store.get(&key))
}

/// Escribe una clave del almacén en disco.
///
/// # Errors
/// Devuelve un mensaje si la clave/valor no son válidos o si falla la escritura.
#[tauri::command]
fn store_set(state: tauri::State<'_, HostState>, key: String, value: String) -> Result<(), String> {
    let mut store = state
        .store
        .lock()
        .map_err(|_| "almacén envenenado".to_owned())?;
    store.set(&key, &value)
}

/// Borra una clave del almacén en disco.
///
/// # Errors
/// Devuelve un mensaje si la clave no es válida o si falla la escritura.
#[tauri::command]
fn store_remove(state: tauri::State<'_, HostState>, key: String) -> Result<(), String> {
    let mut store = state
        .store
        .lock()
        .map_err(|_| "almacén envenenado".to_owned())?;
    store.remove(&key)
}

/// Recibe el JSON de la autoprueba de UI, lo imprime y cierra la app.
#[tauri::command]
fn selftest_report(json: String, app: tauri::AppHandle) {
    println!("ANZAN_SELFTEST {json}");
    if let Ok(path) = std::env::var("ANZAN_SELFTEST_OUT") {
        let _ = std::fs::write(path, &json);
    }
    app.exit(0);
}

/// Recibe el JSON de la medición, lo escribe en `ANZAN_MEASURE_OUT` y cierra la app.
#[tauri::command]
fn measure_report(json: String, app: tauri::AppHandle) {
    if let Ok(path) = std::env::var("ANZAN_MEASURE_OUT") {
        let _ = std::fs::write(&path, &json);
        println!("ANZAN_MEASURE {path}");
    } else {
        println!("ANZAN_MEASURE {json}");
    }
    app.exit(0);
}

fn env_reps(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn main() {
    if std::env::args()
        .skip(1)
        .any(|arg| arg == "--audio-selftest")
    {
        match audio::selftest(8) {
            Ok(stats) => match serde_json::to_string_pretty(&stats) {
                Ok(json) => println!("{json}"),
                Err(error) => {
                    eprintln!("estadísticas no serializables: {error}");
                    std::process::exit(1);
                }
            },
            Err(error) => {
                eprintln!("audio nativo no disponible: {error}");
                std::process::exit(1);
            }
        }
        return;
    }

    let strings = anzan_content::load(anzan_content::Locale::Es);
    let title = strings.map_or_else(|_| "MathMaster".to_owned(), |strings| strings.app_name);
    let selftest = std::env::args().skip(1).any(|arg| arg == "--ui-selftest");
    let measure = std::env::args().skip(1).any(|arg| arg == "--measure");

    let result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            audio_start,
            audio_click,
            audio_stats,
            audio_clock_ms,
            store_get,
            store_set,
            store_remove,
            selftest_report,
            measure_report
        ])
        .setup(move |app| {
            let dir = app.path().app_data_dir()?;
            let store = Store::load(dir);
            let script = store.init_script();
            app.manage(HostState::new(store));

            let mut builder = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title(&title)
            .inner_size(960.0, 820.0)
            .min_inner_size(720.0, 560.0)
            .resizable(true)
            .center()
            .initialization_script(&script);

            // Los scripts de prueba se inyectan al crear el documento, no con `eval`
            // desde `setup`: así no compiten con la carga de la página.
            if selftest {
                builder = builder.initialization_script(selftest::SCRIPT);
            }
            if measure {
                let reps100 = env_reps("ANZAN_MEASURE_REPS100", 1000);
                let reps200 = env_reps("ANZAN_MEASURE_REPS200", 300);
                let hide_ui = std::env::var("ANZAN_MEASURE_HIDE_UI").is_ok_and(|value| value == "1");
                let no_audio = std::env::var("ANZAN_MEASURE_NO_AUDIO").is_ok_and(|value| value == "1");
                let measure_script = format!(
                    "window.__ANZAN_MEASURE_CONFIG__ = {{ reps100: {reps100}, reps200: {reps200}, hideUI: {hide_ui}, noAudio: {no_audio} }};\n{}",
                    measure::SCRIPT
                );
                builder = builder.initialization_script(&measure_script);
            }
            builder.build()?;

            if selftest {
                std::thread::spawn(|| {
                    std::thread::sleep(std::time::Duration::from_secs(45));
                    eprintln!("la autoprueba de UI no terminó a tiempo");
                    std::process::exit(2);
                });
            }
            if measure {
                std::thread::spawn(|| {
                    std::thread::sleep(std::time::Duration::from_secs(480));
                    eprintln!("la medición no terminó a tiempo");
                    std::process::exit(2);
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(error) = result {
        eprintln!("no se pudo iniciar MathMaster: {error}");
        std::process::exit(1);
    }
}
