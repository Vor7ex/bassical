use crate::audio::metronome::MetronomeState;
use crate::models::tab::TimingPoint;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::State;

pub struct MetronomeStateWrapper(pub Arc<MetronomeState>);

#[tauri::command]
pub fn toggle_metronome(metro: State<MetronomeStateWrapper>) -> Result<bool, String> {
    let prev = metro.0.enabled.load(Ordering::Relaxed);
    metro.0.enabled.store(!prev, Ordering::Relaxed);
    Ok(!prev)
}

#[tauri::command]
pub fn get_metronome_state(metro: State<MetronomeStateWrapper>) -> Result<bool, String> {
    Ok(metro.0.enabled.load(Ordering::Relaxed))
}

#[tauri::command]
pub fn set_metronome_grid(
    timing_points: Vec<TimingPoint>,
    duration_ms: f64,
    metro: State<MetronomeStateWrapper>,
) -> Result<(), String> {
    let rate = f64::from_bits(metro.0.device_rate.load(Ordering::Relaxed));
    let total_frames = (duration_ms / 1000.0 * rate).round() as u64;
    metro.0.set_grid(&timing_points, total_frames);
    Ok(())
}
