// SPDX-License-Identifier: AGPL-3.0-or-later
//! Persistencia en disco del host de escritorio.
//!
//! Un único `store.json` con pares clave/valor (las mismas claves que usa la web en
//! `localStorage`). Al arrancar se inyecta como `window.__ANZAN_STORE__` para que la UI
//! pueda leer de forma síncrona; las escrituras se hacen con archivo temporal y renombrado
//! atómico. La web sigue usando `localStorage`.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// Tamaño máximo de una clave aceptada.
const MAX_KEY: usize = 64;
/// Tamaño máximo de un valor aceptado.
const MAX_VALUE: usize = 512 * 1024;

/// Almacén en memoria respaldado por `store.json`.
pub struct Store {
    dir: PathBuf,
    values: BTreeMap<String, String>,
}

impl Store {
    /// Carga el almacén del directorio de datos (vacío si no existe o está corrupto).
    #[must_use]
    pub fn load(dir: PathBuf) -> Self {
        let values = fs::read(dir.join("store.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<BTreeMap<String, String>>(&bytes).ok())
            .unwrap_or_default();
        Self { dir, values }
    }

    /// Script de inicialización que publica el snapshot antes de cargar la UI.
    #[must_use]
    pub fn init_script(&self) -> String {
        let snapshot = serde_json::to_string(&self.values).unwrap_or_else(|_| "{}".to_owned());
        format!("window.__ANZAN_STORE__ = {snapshot};")
    }

    /// Lee una clave.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }

    /// Escribe una clave y persiste el archivo.
    ///
    /// # Errors
    /// Devuelve un mensaje si la clave/valor no son válidos o si falla la escritura.
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
        validate(key, value)?;
        self.values.insert(key.to_owned(), value.to_owned());
        self.persist()
    }

    /// Borra una clave y persiste el archivo.
    ///
    /// # Errors
    /// Devuelve un mensaje si la clave no es válida o si falla la escritura.
    pub fn remove(&mut self, key: &str) -> Result<(), String> {
        validate(key, "")?;
        self.values.remove(key);
        self.persist()
    }

    fn persist(&self) -> Result<(), String> {
        fs::create_dir_all(&self.dir)
            .map_err(|error| format!("no se pudo crear {}: {error}", self.dir.display()))?;
        let target = self.dir.join("store.json");
        let temporary = self.dir.join("store.json.tmp");
        let bytes = serde_json::to_vec_pretty(&self.values)
            .map_err(|error| format!("no se pudo serializar el almacén: {error}"))?;
        fs::write(&temporary, bytes)
            .map_err(|error| format!("no se pudo escribir {}: {error}", temporary.display()))?;
        fs::rename(&temporary, &target)
            .map_err(|error| format!("no se pudo renombrar a {}: {error}", target.display()))?;
        Ok(())
    }
}

fn validate(key: &str, value: &str) -> Result<(), String> {
    if key.is_empty() || key.len() > MAX_KEY {
        return Err(format!("clave inválida (1..={MAX_KEY} bytes)"));
    }
    if !key.is_ascii() || key.contains(['/', '\\', '\0']) {
        return Err("clave con caracteres no permitidos".to_owned());
    }
    if value.len() > MAX_VALUE {
        return Err(format!("valor demasiado grande (> {MAX_VALUE} bytes)"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_keys_and_large_values() {
        assert!(validate("", "x").is_err());
        assert!(validate("anzan/../x", "x").is_err());
        assert!(validate("anzan.progress", "v1").is_ok());
        assert!(validate("anzan.progress", &"x".repeat(MAX_VALUE + 1)).is_err());
    }

    #[test]
    fn snapshot_is_a_javascript_object() {
        let mut store = Store {
            dir: std::env::temp_dir().join(format!("anzanlab-store-{}", std::process::id())),
            values: BTreeMap::new(),
        };
        assert!(store.set("anzan.progress", "v1,1,0,0,0,0,0").is_ok());
        let script = store.init_script();
        assert!(script.starts_with("window.__ANZAN_STORE__ = {"));
        assert!(script.contains("anzan.progress"));
    }
}
