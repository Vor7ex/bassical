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

/// Computa peaks (amplitud máxima) para un rango temporal del audio cacheado.
/// Usado para zoom milimétrico del waveform: cuando el viewport se estrecha,
/// los 2000 bins del overview no tienen suficiente resolución, así que se
/// recalculan peaks desde los samples completos para el rango visible.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RangeQuery {
    pub start_ms: f64,
    pub end_ms: f64,
    pub num_bins: u32,
}

#[tauri::command]
pub fn get_peaks_in_range(
    path: String,
    range: RangeQuery,
    cache_state: State<AudioCacheState>,
) -> Result<Vec<f32>, String> {
    if range.num_bins == 0 {
        return Ok(vec![]);
    }
    let cache = cache_state.inner().0.clone();
    let cached = cache
        .get(&path)
        .ok_or_else(|| "Audio no decodificado aún".to_string())?;

    compute_peaks_in_range(&cached.samples, cached.sample_rate, cached.channels, &range)
}

/// Computa peaks por rango desde un buffer de samples intercalados.
/// Extraída como función pura para testear sin depender del cache.
fn compute_peaks_in_range(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    query: &RangeQuery,
) -> Result<Vec<f32>, String> {
    if query.num_bins == 0 {
        return Ok(vec![]);
    }
    let ch = channels as usize;
    if ch == 0 || sample_rate == 0 {
        return Err("Sample rate o canales inválidos".to_string());
    }
    let sr = sample_rate as f64;
    let total_samples = samples.len();

    let start_frame = (((query.start_ms / 1000.0) * sr).round() as usize).saturating_mul(ch);
    let end_frame = (((query.end_ms / 1000.0) * sr).round() as usize).saturating_mul(ch);

    let start = start_frame.min(total_samples);
    let end = end_frame.min(total_samples);

    if start >= end {
        return Ok(vec![]);
    }

    let range = end - start;
    let num_bins_us = query.num_bins as usize;
    let bin_size = (range / num_bins_us).max(1);

    let mut peaks = Vec::with_capacity(num_bins_us);
    for i in 0..num_bins_us {
        let bin_start = start + i * bin_size;
        let bin_end = (bin_start + bin_size).min(end);
        if bin_start >= bin_end {
            peaks.push(0.0);
            continue;
        }
        let max = samples[bin_start..bin_end]
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);
        peaks.push(max);
    }
    Ok(peaks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(start_ms: f64, end_ms: f64, num_bins: u32) -> RangeQuery {
        RangeQuery {
            start_ms,
            end_ms,
            num_bins,
        }
    }

    fn make_test_samples(num_frames: usize, channels: usize) -> Vec<f32> {
        (0..num_frames)
            .flat_map(|f| {
                let val = (f as f32 / 10.0).sin();
                (0..channels).map(move |_| val).collect::<Vec<f32>>()
            })
            .collect()
    }

    #[test]
    fn test_compute_peaks_basic() {
        let samples: Vec<f32> = (0..100).map(|i| (i as f32 / 10.0).sin()).collect();
        let peaks = compute_peaks_in_range(&samples, 1000, 1, &range(0.0, 100.0, 10)).unwrap();
        assert_eq!(peaks.len(), 10);
        assert!(peaks.iter().all(|&p| p >= 0.0 && p <= 1.0));
    }

    #[test]
    fn test_compute_peaks_empty_range() {
        let samples = vec![0.5, 0.3, 0.8, 0.1];
        let peaks = compute_peaks_in_range(&samples, 1000, 1, &range(0.0, 0.0, 10)).unwrap();
        assert!(peaks.is_empty());
    }

    #[test]
    fn test_compute_peaks_zero_bins() {
        let samples = vec![0.5, 0.3];
        let peaks = compute_peaks_in_range(&samples, 1000, 1, &range(0.0, 1.0, 0)).unwrap();
        assert!(peaks.is_empty());
    }

    #[test]
    fn test_compute_peaks_clamps_to_samples_len() {
        let samples: Vec<f32> = (0..10).map(|i| (i as f32).abs()).collect();
        let peaks = compute_peaks_in_range(&samples, 1000, 1, &range(0.0, 1000.0, 5)).unwrap();
        assert_eq!(peaks.len(), 5);
    }

    #[test]
    fn test_compute_peaks_stereo() {
        let samples = make_test_samples(100, 2);
        let peaks = compute_peaks_in_range(&samples, 1000, 2, &range(0.0, 100.0, 10)).unwrap();
        assert_eq!(peaks.len(), 10);
    }

    #[test]
    fn test_compute_peaks_finds_max_amplitude() {
        let mut samples: Vec<f32> = vec![0.1; 100];
        samples[50] = 0.95;
        let peaks = compute_peaks_in_range(&samples, 1000, 1, &range(0.0, 100.0, 2)).unwrap();
        assert_eq!(peaks.len(), 2);
        assert!((peaks[0] - 0.1).abs() < 0.001);
        assert!((peaks[1] - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_compute_peaks_invalid_sample_rate() {
        let samples = vec![0.5];
        let result = compute_peaks_in_range(&samples, 0, 1, &range(0.0, 1.0, 10));
        assert!(result.is_err());
    }
}
