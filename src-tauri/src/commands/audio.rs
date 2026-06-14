use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use crate::audio::cache::AudioCacheManager;
use crate::audio::engine::AudioEngine;

pub struct AudioEngineState(pub Mutex<AudioEngine>);
pub struct AudioCacheState(pub AudioCacheManager);

unsafe impl Send for AudioEngineState {}
unsafe impl Sync for AudioEngineState {}
unsafe impl Send for AudioCacheState {}
unsafe impl Sync for AudioCacheState {}

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
    cache_state: State<AudioCacheState>,
) -> Result<AudioInfo, String> {
    let cached = cache_state.0.get_or_load(&path)?;
    
    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    engine.set_current_audio(cached.clone());

    let peaks = cached.peaks.read().unwrap().clone();

    Ok(AudioInfo {
        duration_ms: cached.metadata.duration_ms,
        sample_rate: cached.metadata.sample_rate,
        channels: cached.metadata.channels,
        peaks,
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
    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
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

#[tauri::command]
pub fn cache_audio(_path: String) -> Result<(), String> {
    // Ya no hace nada, el caché se maneja automáticamente en el backend
    Ok(())
}
