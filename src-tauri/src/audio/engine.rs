use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use super::decoder::{AudioMetadata, StreamingDecoder};

const PEAK_BINS: usize = 2000;

struct SharedState {
    buffer: Mutex<Arc<Vec<f32>>>,
    buffer_frames: AtomicU64,
    total_frames: AtomicU64,
    source_rate: AtomicU32,
    channels: AtomicU16,
    device_rate: f64,
    speed: AtomicU64,
    position: AtomicU64,
    is_playing: AtomicBool,
    duration_ms: Mutex<f64>,
    peaks: Mutex<Vec<f32>>,
    peaks_ready: AtomicBool,
    current_path: Mutex<String>,
}

struct CacheEntry {
    samples: Arc<Vec<f32>>,
    metadata: AudioMetadata,
    peaks: Vec<f32>,
}

pub struct AudioEngine {
    shared: Arc<SharedState>,
    stream: Option<Stream>,
    decoder_handle: Option<thread::JoinHandle<()>>,
    stop_decode: Arc<AtomicBool>,
    cache: Mutex<HashMap<String, CacheEntry>>,
}

unsafe impl Send for AudioEngine {}

impl AudioEngine {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device_rate = host
            .default_output_device()
            .and_then(|d| d.default_output_config().ok())
            .map(|c| c.sample_rate().0 as f64)
            .unwrap_or(48000.0);

        AudioEngine {
            shared: Arc::new(SharedState {
                buffer: Mutex::new(Arc::new(Vec::new())),
                buffer_frames: AtomicU64::new(0),
                total_frames: AtomicU64::new(0),
                source_rate: AtomicU32::new(0),
                channels: AtomicU16::new(0),
                device_rate,
                speed: AtomicU64::new(1.0f64.to_bits()),
                position: AtomicU64::new(0),
                is_playing: AtomicBool::new(false),
                duration_ms: Mutex::new(0.0),
                peaks: Mutex::new(vec![0.0; PEAK_BINS]),
                peaks_ready: AtomicBool::new(false),
                current_path: Mutex::new(String::new()),
            }),
            stream: None,
            decoder_handle: None,
            stop_decode: Arc::new(AtomicBool::new(false)),
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn start_decode(&mut self, path: String, metadata: AudioMetadata) -> Result<(), String> {
        let is_same_path = {
            let current = self.shared.current_path.lock().unwrap();
            *current == path
        };

        if is_same_path && self.decoder_handle.is_some() {
            return Ok(());
        }

        self.stop_decode_stream();

        let cached_entry = self
            .cache
            .lock()
            .unwrap()
            .get(&path)
            .map(|e| (e.samples.clone(), e.metadata.clone(), e.peaks.clone()));

        if let Some((cached, meta, cached_peaks)) = cached_entry {
            {
                let mut buf = self.shared.buffer.lock().unwrap();
                *buf = cached;
            }
            self.shared.buffer_frames.store(
                self.shared.buffer.lock().unwrap().len() as u64 / meta.channels as u64,
                Ordering::Relaxed,
            );
            {
                let mut peaks = self.shared.peaks.lock().unwrap();
                *peaks = cached_peaks;
            }
            self.shared.peaks_ready.store(true, Ordering::Relaxed);
            self.apply_metadata(&meta);
            self.shared.position.store(0, Ordering::Relaxed);
            self.shared.is_playing.store(false, Ordering::Relaxed);
            *self.shared.current_path.lock().unwrap() = path;
            self.create_stream()?;
            return Ok(());
        }

        {
            let mut buf = self.shared.buffer.lock().unwrap();
            *buf = Arc::new(Vec::new());
        }
        {
            let mut peaks = self.shared.peaks.lock().unwrap();
            *peaks = vec![0.0; PEAK_BINS];
        }
        self.shared.buffer_frames.store(0, Ordering::Relaxed);
        self.shared.peaks_ready.store(false, Ordering::Relaxed);
        self.apply_metadata(&metadata);
        self.shared.position.store(0, Ordering::Relaxed);
        self.shared.is_playing.store(false, Ordering::Relaxed);
        *self.shared.current_path.lock().unwrap() = path.clone();
        self.create_stream()?;

        let shared = self.shared.clone();
        let stop = self.stop_decode.clone();

        let handle = thread::spawn(move || run_decode_thread(path, shared, stop));
        self.decoder_handle = Some(handle);
        Ok(())
    }

    fn apply_metadata(&self, meta: &AudioMetadata) {
        self.shared
            .source_rate
            .store(meta.sample_rate, Ordering::Relaxed);
        self.shared.channels.store(meta.channels, Ordering::Relaxed);
        self.shared
            .total_frames
            .store(meta.total_frames, Ordering::Relaxed);
        {
            let mut dur = self.shared.duration_ms.lock().unwrap();
            *dur = meta.duration_ms;
        }
        self.shared.speed.store(1.0f64.to_bits(), Ordering::Relaxed);
    }

    fn stop_decode_stream(&mut self) {
        self.stop_decode.store(true, Ordering::Relaxed);
        if let Some(handle) = self.decoder_handle.take() {
            let _ = handle.join();
        }
        self.stop_decode.store(false, Ordering::Relaxed);
    }

    pub fn get_decode_progress(&self) -> f64 {
        let total = self.shared.total_frames.load(Ordering::Relaxed);
        if total == 0 {
            return 1.0;
        }
        let decoded = self.shared.buffer_frames.load(Ordering::Relaxed);
        (decoded as f64 / total as f64).min(1.0)
    }

    pub fn is_decode_done(&self) -> bool {
        let total = self.shared.total_frames.load(Ordering::Relaxed);
        if total == 0 {
            return true;
        }
        self.shared.buffer_frames.load(Ordering::Relaxed) >= total
    }

    fn create_stream(&mut self) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "No se encontró un dispositivo de salida".to_string())?;

        let supported_config = device
            .default_output_config()
            .map_err(|e| format!("Error de configuración de audio: {}", e))?;

        let config: StreamConfig = supported_config.config();
        let shared = self.shared.clone();

        let stream = match supported_config.sample_format() {
            SampleFormat::F32 => {
                device.build_output_stream(&config, create_f32_callback(shared), log_error, None)
            }
            SampleFormat::I16 => {
                device.build_output_stream(&config, create_i16_callback(shared), log_error, None)
            }
            fmt => return Err(format!("Formato de audio no soportado: {:?}", fmt)),
        }
        .map_err(|e| format!("Error al crear stream de audio: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Error al iniciar stream: {}", e))?;

        self.stream = Some(stream);
        Ok(())
    }

    pub fn play(&mut self) -> Result<(), String> {
        if self.shared.buffer_frames.load(Ordering::Acquire) == 0 {
            return Err("No hay audio cargado".to_string());
        }
        self.shared.is_playing.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), String> {
        self.shared.is_playing.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub fn seek(&mut self, position_ms: f64) -> Result<(), String> {
        let source_rate = self.shared.source_rate.load(Ordering::Relaxed) as f64;
        let channels = self.shared.channels.load(Ordering::Relaxed) as f64;
        let frame = (position_ms / 1000.0 * source_rate) as u64;
        self.shared
            .position
            .store(frame * channels as u64, Ordering::Relaxed);
        Ok(())
    }

    pub fn set_speed(&mut self, speed: f64) -> Result<(), String> {
        let clamped = speed.clamp(0.25, 1.0);
        self.shared
            .speed
            .store(clamped.to_bits(), Ordering::Relaxed);
        Ok(())
    }

    pub fn get_position_ms(&self) -> f64 {
        let pos = self.shared.position.load(Ordering::Relaxed);
        let channels = self.shared.channels.load(Ordering::Relaxed) as f64;
        let source_rate = self.shared.source_rate.load(Ordering::Relaxed) as f64;
        if source_rate > 0.0 && channels > 0.0 {
            (pos as f64 / channels / source_rate) * 1000.0
        } else {
            0.0
        }
    }

    pub fn get_duration_ms(&self) -> f64 {
        *self.shared.duration_ms.lock().unwrap()
    }

    pub fn is_playing(&self) -> bool {
        self.shared.is_playing.load(Ordering::Relaxed)
    }

    pub fn get_peaks(&self) -> Vec<f32> {
        self.shared.peaks.lock().unwrap().clone()
    }

    pub fn is_current_path(&self, path: &str) -> bool {
        *self.shared.current_path.lock().unwrap() == path
    }

    pub fn store_in_cache(&self, path: &str) {
        let current = self.shared.current_path.lock().unwrap().clone();
        if current != path {
            return;
        }

        let total = self.shared.total_frames.load(Ordering::Relaxed);
        if total == 0 {
            return;
        }
        let buffer = self.shared.buffer.lock().unwrap().clone();
        let peaks = self.shared.peaks.lock().unwrap().clone();
        let meta = AudioMetadata {
            sample_rate: self.shared.source_rate.load(Ordering::Relaxed),
            channels: self.shared.channels.load(Ordering::Relaxed),
            duration_ms: *self.shared.duration_ms.lock().unwrap(),
            total_frames: total,
        };
        self.cache.lock().unwrap().insert(
            path.to_string(),
            CacheEntry {
                samples: buffer,
                metadata: meta,
                peaks,
            },
        );
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.stop_decode_stream();
    }
}

fn log_error(err: cpal::StreamError) {
    eprintln!("Error en el stream de audio: {}", err);
}

fn should_swap_buffer(decoded: usize, last_swap: usize, interval: usize) -> bool {
    decoded > 0 && (last_swap == 0 || decoded - last_swap >= interval)
}

struct PeakUpdateContext {
    channels: usize,
    offset: usize,
    total_est: usize,
}

fn update_peaks_incremental(peaks: &mut [f32], chunk: &[f32], ctx: &PeakUpdateContext) {
    if ctx.total_est == 0 {
        return;
    }
    let frames = chunk.len() / ctx.channels;
    for frame in 0..frames {
        let mono: f32 = (0..ctx.channels)
            .map(|c| chunk[frame * ctx.channels + c].abs())
            .sum::<f32>()
            / ctx.channels as f32;
        let bin = ((ctx.offset + frame) * PEAK_BINS / ctx.total_est).min(PEAK_BINS - 1);
        if mono > peaks[bin] {
            peaks[bin] = mono;
        }
    }
}

enum DecodeStatus {
    Decoded(Vec<f32>),
    EndOfStream,
    Error(String),
}

fn decode_next(decoder: &mut StreamingDecoder) -> DecodeStatus {
    match decoder.decode_chunk(8192) {
        Ok(c) if c.is_empty() => DecodeStatus::EndOfStream,
        Ok(c) => DecodeStatus::Decoded(c),
        Err(e) => DecodeStatus::Error(e.to_string()),
    }
}

struct DecodeLoop {
    local_samples: Vec<f32>,
    decoded_frames: usize,
    last_swap_frames: usize,
    channels: usize,
    total_est: usize,
    swap_interval: usize,
    shared: Arc<SharedState>,
}

impl DecodeLoop {
    fn new(shared: Arc<SharedState>) -> Self {
        let channels = shared.channels.load(Ordering::Relaxed) as usize;
        let total_est = shared.total_frames.load(Ordering::Relaxed) as usize;
        let sample_rate = shared.source_rate.load(Ordering::Relaxed) as usize;
        let swap_interval = (sample_rate * channels).max(44100 * 2);

        DecodeLoop {
            local_samples: Vec::new(),
            decoded_frames: 0,
            last_swap_frames: 0,
            channels,
            total_est,
            swap_interval,
            shared,
        }
    }

    fn process_chunk(&mut self, chunk: &[f32]) {
        let chunk_frames = chunk.len() / self.channels;

        {
            let mut peaks = self.shared.peaks.lock().unwrap();
            update_peaks_incremental(
                &mut peaks,
                chunk,
                &PeakUpdateContext {
                    channels: self.channels,
                    offset: self.decoded_frames,
                    total_est: self.total_est,
                },
            );
        }

        self.local_samples.extend_from_slice(chunk);
        self.decoded_frames += chunk_frames;

        if should_swap_buffer(
            self.decoded_frames,
            self.last_swap_frames,
            self.swap_interval,
        ) {
            let mut buf = self.shared.buffer.lock().unwrap();
            *buf = Arc::new(self.local_samples.clone());
            self.last_swap_frames = self.decoded_frames;
        }

        self.shared
            .buffer_frames
            .store(self.decoded_frames as u64, Ordering::Release);
    }

    fn finish(self) {
        let mut buf = self.shared.buffer.lock().unwrap();
        *buf = Arc::new(self.local_samples);
        self.shared.peaks_ready.store(true, Ordering::Release);
    }

    fn run(&mut self, decoder: &mut StreamingDecoder, stop: &AtomicBool) {
        while !stop.load(Ordering::Relaxed) {
            match decode_next(decoder) {
                DecodeStatus::Decoded(chunk) => self.process_chunk(&chunk),
                DecodeStatus::EndOfStream => break,
                DecodeStatus::Error(e) => {
                    eprintln!("Error de decodificación: {}", e);
                    break;
                }
            }
        }
    }
}

fn run_decode_thread(path: String, shared: Arc<SharedState>, stop: Arc<AtomicBool>) {
    let mut decoder = match StreamingDecoder::open(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error abriendo decoder: {}", e);
            return;
        }
    };

    let mut decode_loop = DecodeLoop::new(shared);
    decode_loop.run(&mut decoder, &stop);
    decode_loop.finish();
}

macro_rules! create_audio_callback {
    ($shared:expr, $sample_type:ty, $zero:expr, $convert:expr) => {{
        let s = $shared.clone();
        move |data: &mut [$sample_type], _: &cpal::OutputCallbackInfo| {
            if !s.is_playing.load(Ordering::Relaxed) {
                data.fill($zero);
                return;
            }

            let total_frames = s.buffer_frames.load(Ordering::Acquire);
            if total_frames == 0 {
                data.fill($zero);
                return;
            }

            let buffer = {
                let guard = s.buffer.lock().unwrap();
                Arc::clone(&guard)
            };
            let src_rate = s.source_rate.load(Ordering::Relaxed) as f64;
            let dev_rate = s.device_rate;
            let ch = s.channels.load(Ordering::Relaxed) as usize;
            let speed = f64::from_bits(s.speed.load(Ordering::Relaxed));
            let step = (src_rate * speed) / dev_rate;
            let buf_frames = buffer.len() / ch;
            let mut pos = s.position.load(Ordering::Relaxed) as f64 / ch as f64;

            let out_frames = data.len() / ch;
            let mut filled = 0usize;

            while filled < out_frames {
                let src_idx = pos as usize;
                if src_idx >= buf_frames {
                    let remaining = out_frames - filled;
                    for sample in &mut data[filled * ch..(filled + remaining) * ch] {
                        *sample = $zero;
                    }
                    s.is_playing.store(false, Ordering::Relaxed);
                    break;
                }

                let frac = (pos - src_idx as f64) as f32;
                let next = (src_idx + 1).min(buf_frames - 1);

                let base = filled * ch;
                for c in 0..ch {
                    let s0 = buffer[src_idx * ch + c];
                    let s1 = buffer[next * ch + c];
                    data[base + c] = $convert(s0 + (s1 - s0) * frac);
                }

                filled += 1;
                pos += step;
            }

            s.position
                .store((pos * ch as f64) as u64, Ordering::Relaxed);
        }
    }};
}

fn create_f32_callback(
    shared: Arc<SharedState>,
) -> impl FnMut(&mut [f32], &cpal::OutputCallbackInfo) + Send + 'static {
    create_audio_callback!(shared, f32, 0.0, |v: f32| v)
}

fn create_i16_callback(
    shared: Arc<SharedState>,
) -> impl FnMut(&mut [i16], &cpal::OutputCallbackInfo) + Send + 'static {
    create_audio_callback!(shared, i16, 0, |v: f32| (v * i16::MAX as f32) as i16)
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_atomic_roundtrip() {
        let speeds: [f64; 4] = [0.25, 0.5, 0.75, 1.0];
        for &s in &speeds {
            let bits = s.to_bits();
            let back = f64::from_bits(bits);
            assert!((s - back).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_position_ms_conversion() {
        let sr: f64 = 44100.0;
        let ch: f64 = 2.0;
        let one_sec_frames: f64 = sr * ch;
        let ms = (one_sec_frames / ch / sr) * 1000.0;
        assert!((ms - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_step_calculation() {
        let src_rate: f64 = 44100.0;
        let dev_rate: f64 = 48000.0;
        let speed: f64 = 1.0;
        let step = (src_rate * speed) / dev_rate;
        assert!((step - 0.91875).abs() < 0.001);

        let step_half = (src_rate * 0.5) / dev_rate;
        assert!((step_half - 0.459375).abs() < 0.001);
    }
}
