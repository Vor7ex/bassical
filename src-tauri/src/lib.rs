mod audio;
mod calibration;
mod commands;
mod models;
mod parser;
mod persistence;

use commands::library;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            library::init_app,
            library::get_library,
            library::get_library_with_status,
            library::add_song,
            library::update_song,
            library::delete_song,
            library::check_audio_exists,
            library::reassign_audio_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
