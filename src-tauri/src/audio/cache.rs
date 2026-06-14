use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use super::decoder::{probe_file, AudioMetadata, StreamingDecoder};

const PEAK_BINS: usize = 2000;
const MAX_CACHED_SONGS: usize = 3;

pub struct CachedAudio {
    pub path: String,
    pub metadata: AudioMetadata,
    pub samples: RwLock<Vec<f32>>,
    pub peaks: RwLock<Vec<f32>>,
    pub decoded_frames: AtomicU64,
    pub is_done: AtomicBool,
    pub error: Mutex<Option<String>>,
}

pub struct AudioCacheManager {
    cache: Mutex<HashMap<String, Arc<CachedAudio>>>,
    lru: Mutex<VecDeque<String>>,
}

impl AudioCacheManager {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            lru: Mutex::new(VecDeque::new()),
        }
    }

    pub fn get_or_load(&self, path: &str) -> Result<Arc<CachedAudio>, String> {
        // Check if already in cache
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(path) {
                // Update LRU
                let mut lru = self.lru.lock().unwrap();
                lru.retain(|p| p != path);
                lru.push_back(path.to_string());
                return Ok(Arc::clone(cached));
            }
        }

        // Not in cache, we need to load it
        let metadata = probe_file(path)?;
        
        let total_samples = metadata.total_frames as usize * metadata.channels as usize;
        // Si no se pudo determinar el total de frames, reservamos para ~3 minutos (stereo a 44.1kHz)
        let initial_capacity = if total_samples > 0 {
            total_samples
        } else {
            44100 * 2 * 180
        };

        let cached = Arc::new(CachedAudio {
            path: path.to_string(),
            metadata: metadata.clone(),
            samples: RwLock::new(Vec::with_capacity(initial_capacity)),
            peaks: RwLock::new(vec![0.0; PEAK_BINS]),
            decoded_frames: AtomicU64::new(0),
            is_done: AtomicBool::new(false),
            error: Mutex::new(None),
        });

        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(path.to_string(), Arc::clone(&cached));
            
            let mut lru = self.lru.lock().unwrap();
            lru.push_back(path.to_string());
            
            if cache.len() > MAX_CACHED_SONGS {
                if let Some(oldest) = lru.pop_front() {
                    cache.remove(&oldest);
                }
            }
        }

        let path_clone = path.to_string();
        let cached_clone = Arc::clone(&cached);

        thread::spawn(move || {
            run_decode_thread(path_clone, cached_clone);
        });

        Ok(cached)
    }
}

impl Default for AudioCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

fn run_decode_thread(path: String, cached: Arc<CachedAudio>) {
    let mut decoder = match StreamingDecoder::open(&path) {
        Ok(d) => d,
        Err(e) => {
            *cached.error.lock().unwrap() = Some(e);
            cached.is_done.store(true, Ordering::Release);
            return;
        }
    };

    let channels = cached.metadata.channels as usize;
    let total_est = cached.metadata.total_frames as usize;

    loop {
        match decoder.decode_chunk(8192) {
            Ok(chunk) => {
                if chunk.is_empty() {
                    break;
                }

                let chunk_frames = chunk.len() / channels;
                let current_frames = cached.decoded_frames.load(Ordering::Relaxed) as usize;

                // Actualizar picos
                if total_est > 0 {
                    let mut peaks = cached.peaks.write().unwrap();
                    update_peaks_incremental(&mut peaks, &chunk, channels, current_frames, total_est);
                }

                // Escribir muestras (Lock ultra rápido gracias a la pre-asignación)
                {
                    let mut samples = cached.samples.write().unwrap();
                    samples.extend_from_slice(&chunk);
                }

                cached.decoded_frames.fetch_add(chunk_frames as u64, Ordering::Release);
            }
            Err(e) => {
                *cached.error.lock().unwrap() = Some(e);
                break;
            }
        }

        if decoder.is_done() {
            break;
        }
    }

    cached.is_done.store(true, Ordering::Release);
}

fn update_peaks_incremental(
    peaks: &mut [f32],
    chunk: &[f32],
    channels: usize,
    offset: usize,
    total_est: usize,
) {
    if total_est == 0 {
        return;
    }
    let frames = chunk.len() / channels;
    for frame in 0..frames {
        let mono: f32 = (0..channels)
            .map(|c| chunk[frame * channels + c].abs())
            .sum::<f32>()
            / channels as f32;
        let bin = ((offset + frame) * PEAK_BINS / total_est).min(PEAK_BINS - 1);
        if mono > peaks[bin] {
            peaks[bin] = mono;
        }
    }
}
