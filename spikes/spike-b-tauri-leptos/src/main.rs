use std::io::Write;

#[tauri::command]
fn ping(seq: u64) -> u64 {
    seq
}

#[tauri::command]
fn report(json: String, app: tauri::AppHandle) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spikes/results");
    let _ = std::fs::create_dir_all(&dir);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = dir.join(format!("spike-b-{stamp}.json"));
    if std::fs::write(&path, &json).is_ok() {
        println!("SPIKE_B_REPORT {}", path.display());
    }
    let _ = std::io::stdout().flush();
    app.exit(0);
}

#[tauri::command]
fn fail(msg: String, app: tauri::AppHandle) {
    eprintln!("SPIKE_B_ERROR: {msg}");
    app.exit(1);
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ping, report, fail])
        .run(tauri::generate_context!())
        .expect("error al arrancar Tauri");
}
