// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod dto;
mod game;
mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // All IPC commands (incl. `engine_version`) are registered in
    // commands::register — a second invoke_handler call here would replace,
    // not extend, the command list.
    commands::register(tauri::Builder::default())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
