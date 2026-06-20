use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;

use crate::audio::cache::AudioCache;
use crate::audio::decoder::probe_file;
use crate::audio::engine::{spawn_decoder_thread, AudioEngine, StreamingState};

pub struct AudioEngineState(pub Mutex<AudioEngine>);

unsafe impl Send for AudioEngineState {}
unsafe impl Sync for AudioEngineState {}

pub struct AudioCacheState(pub Arc<AudioCache>);

unsafe impl Send for AudioCacheState {}
unsafe impl Sync for AudioCacheState {}

impl AudioCacheState {
    pub fn new(cache: Arc<AudioCache>) -> Self {
        Self(cache)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioInfo {
    pub duration_ms: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub peaks: Vec<f32>,
    pub complete: bool,
}

fn audio_info_from_playback(
    info: crate::audio::engine::AudioPlaybackInfo,
    peaks: Vec<f32>,
    complete: bool,
) -> AudioInfo {
    AudioInfo {
        duration_ms: info.duration_ms,
        sample_rate: info.sample_rate,
        channels: info.channels,
        peaks,
        complete,
    }
}

struct StreamSetup {
    decode_immediately: bool,
    register_active: bool,
}

fn setup_stream(
    path: String,
    engine: &mut AudioEngine,
    cache: &AudioCache,
    setup: StreamSetup,
) -> Result<AudioInfo, String> {
    let metadata = probe_file(&path)?;
    let device_rate = engine.device_rate();

    let streaming = StreamingState::new(metadata.clone(), engine.channels(), device_rate);
    if setup.decode_immediately {
        streaming.set_decode_immediately(true);
    }
    let streaming = std::sync::Arc::new(streaming);

    engine.set_current_stream(streaming.clone(), path.clone());
    if setup.register_active {
        cache.set_active(path.clone(), streaming.clone());
    }

    spawn_decoder_thread(path, streaming);

    Ok(AudioInfo {
        duration_ms: metadata.duration_ms,
        sample_rate: metadata.sample_rate,
        channels: metadata.channels,
        peaks: vec![],
        complete: false,
    })
}

fn resolve_audio(
    path: String,
    engine: &mut AudioEngine,
    cache: &AudioCache,
    decode_immediately: bool,
) -> Result<AudioInfo, String> {
    if let Some(cached) = cache.get(&path) {
        let (info, peaks) = engine.load_from_cache(path, cached)?;
        return Ok(audio_info_from_playback(info, peaks, true));
    }

    if let Some(active) = cache.get_active(&path) {
        let (info, _peaks) = engine.attach_to_active_decode(path, active)?;
        return Ok(audio_info_from_playback(info, vec![], false));
    }

    setup_stream(
        path,
        engine,
        cache,
        StreamSetup {
            decode_immediately,
            register_active: true,
        },
    )
}

#[tauri::command]
pub fn load_audio(
    path: String,
    autoplay: Option<bool>,
    engine_state: State<AudioEngineState>,
    cache_state: State<AudioCacheState>,
) -> Result<AudioInfo, String> {
    let cache = cache_state.inner().0.clone();
    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    let info = setup_stream(
        path,
        &mut engine,
        &cache,
        StreamSetup {
            decode_immediately: false,
            register_active: false,
        },
    )?;
    if autoplay.unwrap_or(false) {
        engine.play()?;
    }
    Ok(info)
}

#[tauri::command]
pub fn decode_audio(
    path: String,
    engine_state: State<AudioEngineState>,
    cache_state: State<AudioCacheState>,
) -> Result<AudioInfo, String> {
    let cache = cache_state.inner().0.clone();
    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    resolve_audio(path, &mut engine, &cache, true)
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
    position_ms: Option<f64>,
    engine_state: State<AudioEngineState>,
) -> Result<AudioInfo, String> {
    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    let info = engine.start_playback(path)?;
    if let Some(pos) = position_ms {
        let _ = engine.seek(pos);
    }
    Ok(AudioInfo {
        duration_ms: info.duration_ms,
        sample_rate: info.sample_rate,
        channels: info.channels,
        peaks: vec![],
        complete: false,
    })
}

#[tauri::command]
pub fn stop_playback(engine_state: State<AudioEngineState>) -> Result<(), String> {
    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    engine.stop_playback();
    Ok(())
}

#[tauri::command]
pub fn activate_full_buffer_playback(engine_state: State<AudioEngineState>) -> Result<(), String> {
    let mut engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    engine.switch_to_full_buffer_playback()
}

#[tauri::command]
pub fn is_full_buffer_ready(engine_state: State<AudioEngineState>) -> Result<bool, String> {
    let engine = engine_state.inner().0.lock().map_err(|e| e.to_string())?;
    Ok(engine.is_full_buffer_ready())
}
