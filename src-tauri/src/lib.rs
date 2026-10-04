// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Reports the engine crate's version — proves the workspace wiring
/// from the UI all the way down into `crates/polyku-engine`.
#[tauri::command]
fn engine_version() -> String {
    polyku_engine::version().to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, engine_version])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
