use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use crate::audio::decoder::probe_file;
use crate::audio::engine::{spawn_decoder_thread, AudioEngine};

pub struct AudioEngineState(pub Mutex<AudioEngine>);

unsafe impl Send for AudioEngineState {}
unsafe impl Sync for AudioEngineState {}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioInfo {
    pub duration_ms: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub peaks: Vec<f32>,
}

#[tauri::command]
pub fn load_audio(
    path: String,
    engine_state: State<AudioEngineState>,
) -> Result<AudioInfo, String> {
    let metadata = probe_file(&path)?;

    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    let device_rate = engine.device_rate();

    let streaming =
        crate::audio::engine::StreamingState::new(metadata.clone(), engine.channels(), device_rate);
    let streaming = std::sync::Arc::new(streaming);

    engine.set_current_stream(streaming.clone());
    drop(engine);

    spawn_decoder_thread(path, streaming.clone());

    Ok(AudioInfo {
        duration_ms: metadata.duration_ms,
        sample_rate: metadata.sample_rate,
        channels: metadata.channels,
        peaks: Vec::new(),
    })
}

#[tauri::command]
pub fn get_decode_progress(engine_state: State<AudioEngineState>) -> Result<f64, String> {
    let engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_decode_progress())
}

#[tauri::command]
pub fn get_decoded_peaks(engine_state: State<AudioEngineState>) -> Result<Vec<f32>, String> {
    let engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_peaks())
}

#[tauri::command]
pub fn play_audio(engine_state: State<AudioEngineState>) -> Result<(), String> {
    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    engine.play()
}

#[tauri::command]
pub fn pause_audio(engine_state: State<AudioEngineState>) -> Result<(), String> {
    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    engine.pause()
}

#[tauri::command]
pub fn seek_audio(position_ms: f64, engine_state: State<AudioEngineState>) -> Result<(), String> {
    let engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    engine.seek(position_ms)
}

#[tauri::command]
pub fn set_playback_speed(speed: f64, engine_state: State<AudioEngineState>) -> Result<(), String> {
    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    engine.set_speed(speed)
}

#[tauri::command]
pub fn get_audio_position(engine_state: State<AudioEngineState>) -> Result<f64, String> {
    let engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_position_ms())
}

#[tauri::command]
pub fn get_audio_duration(engine_state: State<AudioEngineState>) -> Result<f64, String> {
    let engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_duration_ms())
}

#[tauri::command]
pub fn is_audio_playing(engine_state: State<AudioEngineState>) -> Result<bool, String> {
    let engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    Ok(engine.is_playing())
}
