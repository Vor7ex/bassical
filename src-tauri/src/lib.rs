mod audio;
mod calibration;
mod commands;
mod models;
mod parser;
mod persistence;

use commands::audio::AudioEngineState;
use commands::library;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let engine = audio::engine::AudioEngine::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AudioEngineState(Mutex::new(engine)))
        .invoke_handler(tauri::generate_handler![
            library::init_app,
            library::get_library,
            library::get_library_with_status,
            library::add_song,
            library::update_song,
            library::delete_song,
            library::check_audio_exists,
            library::reassign_audio_path,
            library::extract_metadata,
            commands::audio::load_audio,
            commands::audio::decode_audio,
            commands::audio::get_decode_progress,
            commands::audio::get_decoded_peaks,
            commands::audio::play_audio,
            commands::audio::pause_audio,
            commands::audio::seek_audio,
            commands::audio::set_playback_speed,
            commands::audio::get_audio_position,
            commands::audio::get_audio_duration,
            commands::audio::is_audio_playing,
            commands::audio::start_playback,
            commands::audio::stop_playback,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
