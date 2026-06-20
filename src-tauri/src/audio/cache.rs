use moka::sync::Cache;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use super::engine::StreamingState;

const MAX_CACHE_BYTES: u64 = 200 * 1024 * 1024;

pub struct CachedAudio {
    pub peaks: Vec<f32>,
    pub samples: Arc<Vec<f32>>,
    pub duration_ms: f64,
    pub sample_rate: u32,
    pub channels: u16,
}

pub struct ActiveDecode {
    pub path: String,
    pub streaming: Arc<StreamingState>,
}

pub struct AudioCache {
    completed: Cache<String, Arc<CachedAudio>>,
    active: Mutex<Option<ActiveDecode>>,
}

impl AudioCache {
    pub fn new() -> Self {
        let completed = Cache::builder()
            .max_capacity(MAX_CACHE_BYTES)
            .weigher(|_key, value: &Arc<CachedAudio>| {
                let samples_bytes = value.samples.len() * std::mem::size_of::<f32>();
                let peaks_bytes = value.peaks.len() * std::mem::size_of::<f32>();
                let metadata_bytes = std::mem::size_of::<CachedAudio>();
                (samples_bytes + peaks_bytes + metadata_bytes) as u32
            })
            .build();

        Self {
            completed,
            active: Mutex::new(None),
        }
    }

    pub fn get(&self, path: &str) -> Option<Arc<CachedAudio>> {
        self.completed.get(path)
    }

    pub fn get_active(&self, path: &str) -> Option<Arc<ActiveDecode>> {
        let guard = self.active.lock().unwrap();
        guard.as_ref().and_then(|active| {
            if active.path == path {
                Some(Arc::new(ActiveDecode {
                    path: active.path.clone(),
                    streaming: active.streaming.clone(),
                }))
            } else {
                None
            }
        })
    }

    pub fn set_active(&self, path: String, streaming: Arc<StreamingState>) {
        let mut guard = self.active.lock().unwrap();
        *guard = Some(ActiveDecode { path, streaming });
    }

    pub fn promote_active_to_completed(&self) -> Result<(), String> {
        let mut guard = self.active.lock().unwrap();
        let Some(active) = guard.take() else {
            return Ok(());
        };

        if !active.streaming.is_done.load(Ordering::Acquire) {
            *guard = Some(active);
            return Err("La decodificación activa aún no ha terminado".to_string());
        }

        let samples = Arc::new(active.streaming.decoded_buffer.lock().unwrap().clone());
        let cached = Arc::new(CachedAudio {
            peaks: active.streaming.get_peaks(),
            duration_ms: active.streaming.get_duration_ms(),
            sample_rate: active.streaming.metadata.sample_rate,
            channels: active.streaming.metadata.channels,
            samples,
        });

        self.completed.insert(active.path, cached);
        Ok(())
    }
}
