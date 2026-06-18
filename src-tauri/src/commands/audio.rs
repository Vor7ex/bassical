use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use crate::audio::decoder::probe_file;
use crate::audio::engine::{spawn_decoder_thread, AudioEngine, StreamingState};

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

fn setup_stream(
    path: String,
    engine: &mut AudioEngine,
    decode_immediately: bool,
) -> Result<AudioInfo, String> {
    let metadata = probe_file(&path)?;
    let device_rate = engine.device_rate();

    let streaming = StreamingState::new(metadata.clone(), engine.channels(), device_rate);
    if decode_immediately {
        streaming.set_decode_immediately(true);
    }
    let initial_peaks = streaming.get_peaks();
    let streaming = std::sync::Arc::new(streaming);

    engine.set_current_stream(streaming.clone());

    spawn_decoder_thread(path, streaming);

    Ok(AudioInfo {
        duration_ms: metadata.duration_ms,
        sample_rate: metadata.sample_rate,
        channels: metadata.channels,
        peaks: initial_peaks,
    })
}

#[tauri::command]
pub fn load_audio(
    path: String,
    engine_state: State<AudioEngineState>,
) -> Result<AudioInfo, String> {
    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    setup_stream(path, &mut engine, false)
}

#[tauri::command]
pub fn decode_audio(
    path: String,
    engine_state: State<AudioEngineState>,
) -> Result<AudioInfo, String> {
    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    setup_stream(path, &mut engine, true)
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

#[tauri::command]
pub fn start_playback(
    path: String,
    engine_state: State<AudioEngineState>,
) -> Result<AudioInfo, String> {
    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    let info = engine.start_playback(path)?;
    Ok(AudioInfo {
        duration_ms: info.duration_ms,
        sample_rate: info.sample_rate,
        channels: info.channels,
        peaks: vec![],
    })
}

#[tauri::command]
pub fn stop_playback(engine_state: State<AudioEngineState>) -> Result<(), String> {
    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    engine.stop_playback();
    Ok(())
}
