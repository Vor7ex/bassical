use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use crate::audio::decoder;
use crate::audio::engine::AudioEngine;

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
pub fn load_audio(path: String, state: State<AudioEngineState>) -> Result<AudioInfo, String> {
    let metadata = decoder::probe_file(&path)?;

    let mut engine = state.inner().0.lock().map_err(|e| e.to_string())?;
    let already_loaded = engine.is_current_path(&path);
    engine.start_decode(path, metadata.clone())?;

    let peaks = if already_loaded {
        engine.get_peaks()
    } else {
        Vec::new()
    };

    Ok(AudioInfo {
        duration_ms: metadata.duration_ms,
        sample_rate: metadata.sample_rate,
        channels: metadata.channels,
        peaks,
    })
}

#[tauri::command]
pub fn get_decode_progress(state: State<AudioEngineState>) -> Result<f64, String> {
    let engine = state.inner().0.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_decode_progress())
}

#[tauri::command]
pub fn get_decoded_peaks(state: State<AudioEngineState>) -> Result<Vec<f32>, String> {
    let engine = state.inner().0.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_peaks())
}

#[tauri::command]
pub fn play_audio(state: State<AudioEngineState>) -> Result<(), String> {
    let mut engine = state.inner().0.lock().map_err(|e| e.to_string())?;
    engine.play()
}

#[tauri::command]
pub fn pause_audio(state: State<AudioEngineState>) -> Result<(), String> {
    let mut engine = state.inner().0.lock().map_err(|e| e.to_string())?;
    engine.pause()
}

#[tauri::command]
pub fn seek_audio(position_ms: f64, state: State<AudioEngineState>) -> Result<(), String> {
    let mut engine = state.inner().0.lock().map_err(|e| e.to_string())?;
    engine.seek(position_ms)
}

#[tauri::command]
pub fn set_playback_speed(speed: f64, state: State<AudioEngineState>) -> Result<(), String> {
    let mut engine = state.inner().0.lock().map_err(|e| e.to_string())?;
    engine.set_speed(speed)
}

#[tauri::command]
pub fn get_audio_position(state: State<AudioEngineState>) -> Result<f64, String> {
    let engine = state.inner().0.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_position_ms())
}

#[tauri::command]
pub fn get_audio_duration(state: State<AudioEngineState>) -> Result<f64, String> {
    let engine = state.inner().0.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_duration_ms())
}

#[tauri::command]
pub fn is_audio_playing(state: State<AudioEngineState>) -> Result<bool, String> {
    let engine = state.inner().0.lock().map_err(|e| e.to_string())?;
    Ok(engine.is_playing())
}

#[tauri::command]
pub fn cache_audio(path: String, state: State<AudioEngineState>) -> Result<(), String> {
    let engine = state.inner().0.lock().map_err(|e| e.to_string())?;
    engine.store_in_cache(&path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_info_serialization() {
        let info = AudioInfo {
            duration_ms: 180000.0,
            sample_rate: 44100,
            channels: 2,
            peaks: vec![0.5, 0.8, 0.3],
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("durationMs"));
        assert!(json.contains("sampleRate"));
        assert!(json.contains("channels"));
        assert!(json.contains("peaks"));

        let deserialized: AudioInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.duration_ms, 180000.0);
        assert_eq!(deserialized.sample_rate, 44100);
        assert_eq!(deserialized.channels, 2);
        assert_eq!(deserialized.peaks.len(), 3);
    }
}
