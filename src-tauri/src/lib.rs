mod audio;
mod calibration;
mod commands;
mod models;
mod parser;
mod persistence;

use crate::calibration::CalibrationState;
use audio::metronome::MetronomeState;
use commands::audio::{AudioCacheState, AudioEngineState};
use commands::calibration::CalibrationTapState;
use commands::library;
use commands::metronome::MetronomeStateWrapper;
use std::sync::{Arc, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let calib = Arc::new(CalibrationState::new());
    let metronome = Arc::new(MetronomeState::new(48000));
    let cache = Arc::new(audio::cache::AudioCache::new());
    let engine = audio::engine::AudioEngine::new(cache.clone(), calib.clone(), metronome.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AudioEngineState(Mutex::new(engine)))
        .manage(AudioCacheState::new(cache))
        .manage(CalibrationTapState(calib))
        .manage(MetronomeStateWrapper(metronome))
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
            commands::audio::activate_full_buffer_playback,
            commands::audio::is_full_buffer_ready,
            commands::audio::set_song_volume,
            commands::calibration::get_calibration,
            commands::calibration::save_timing_points,
            commands::calibration::clear_calibration,
            commands::calibration::record_calibration_tap,
            commands::metronome::toggle_metronome,
            commands::metronome::get_metronome_state,
            commands::metronome::set_metronome_grid,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
