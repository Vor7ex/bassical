use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use super::cache::CachedAudio;

struct PlaybackState {
    current_audio: Mutex<Option<Arc<CachedAudio>>>,
    position: AtomicU64,
    is_playing: AtomicBool,
    speed: AtomicU64, // f64 in bits
    device_rate: f64,
}

pub struct AudioEngine {
    state: Arc<PlaybackState>,
    stream: Option<Stream>,
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

        let state = Arc::new(PlaybackState {
            current_audio: Mutex::new(None),
            position: AtomicU64::new(0),
            is_playing: AtomicBool::new(false),
            speed: AtomicU64::new(1.0f64.to_bits()),
            device_rate,
        });

        let mut engine = AudioEngine { state, stream: None };
        if let Err(e) = engine.create_stream() {
            eprintln!("Failed to create initial stream: {}", e);
        }
        engine
    }

    pub fn set_current_audio(&mut self, audio: Arc<CachedAudio>) {
        let mut current = self.state.current_audio.lock().unwrap();
        *current = Some(audio);
        self.state.position.store(0, Ordering::Relaxed);
        self.state.is_playing.store(false, Ordering::Relaxed);
        self.state.speed.store(1.0f64.to_bits(), Ordering::Relaxed);
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
        let state = self.state.clone();

        let stream = match supported_config.sample_format() {
            SampleFormat::F32 => {
                device.build_output_stream(&config, create_f32_callback(state), log_error, None)
            }
            SampleFormat::I16 => {
                device.build_output_stream(&config, create_i16_callback(state), log_error, None)
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
        self.state.is_playing.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), String> {
        self.state.is_playing.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub fn seek(&mut self, position_ms: f64) -> Result<(), String> {
        let audio_opt = self.state.current_audio.lock().unwrap();
        if let Some(audio) = audio_opt.as_ref() {
            let source_rate = audio.metadata.sample_rate as f64;
            let channels = audio.metadata.channels as f64;
            let frame = (position_ms / 1000.0 * source_rate) as u64;
            self.state
                .position
                .store(frame * channels as u64, Ordering::Relaxed);
        }
        Ok(())
    }

    pub fn set_speed(&mut self, speed: f64) -> Result<(), String> {
        let clamped = speed.clamp(0.25, 1.0);
        self.state.speed.store(clamped.to_bits(), Ordering::Relaxed);
        Ok(())
    }

    pub fn get_position_ms(&self) -> f64 {
        let audio_opt = self.state.current_audio.lock().unwrap();
        if let Some(audio) = audio_opt.as_ref() {
            let pos = self.state.position.load(Ordering::Relaxed);
            let channels = audio.metadata.channels as f64;
            let source_rate = audio.metadata.sample_rate as f64;
            if source_rate > 0.0 && channels > 0.0 {
                return (pos as f64 / channels / source_rate) * 1000.0;
            }
        }
        0.0
    }

    pub fn get_duration_ms(&self) -> f64 {
        let audio_opt = self.state.current_audio.lock().unwrap();
        if let Some(audio) = audio_opt.as_ref() {
            audio.metadata.duration_ms
        } else {
            0.0
        }
    }

    pub fn get_decode_progress(&self) -> f64 {
        let audio_opt = self.state.current_audio.lock().unwrap();
        if let Some(audio) = audio_opt.as_ref() {
            let total = audio.metadata.total_frames;
            if total == 0 {
                return 1.0;
            }
            let decoded = audio.decoded_frames.load(Ordering::Relaxed);
            (decoded as f64 / total as f64).min(1.0)
        } else {
            0.0
        }
    }

    pub fn get_peaks(&self) -> Vec<f32> {
        let audio_opt = self.state.current_audio.lock().unwrap();
        if let Some(audio) = audio_opt.as_ref() {
            audio.peaks.read().unwrap().clone()
        } else {
            Vec::new()
        }
    }

    pub fn is_playing(&self) -> bool {
        self.state.is_playing.load(Ordering::Relaxed)
    }
}

fn log_error(err: cpal::StreamError) {
    eprintln!("Error en el stream de audio: {}", err);
}

macro_rules! create_audio_callback {
    ($state:expr, $sample_type:ty, $zero:expr, $convert:expr) => {{
        let s = $state.clone();
        move |data: &mut [$sample_type], _: &cpal::OutputCallbackInfo| {
            if !s.is_playing.load(Ordering::Relaxed) {
                data.fill($zero);
                return;
            }

            let audio_opt = {
                let guard = s.current_audio.lock().unwrap();
                guard.clone()
            };

            let audio = match audio_opt {
                Some(a) => a,
                None => {
                    data.fill($zero);
                    return;
                }
            };

            let src_rate = audio.metadata.sample_rate as f64;
            let dev_rate = s.device_rate;
            let ch = audio.metadata.channels as usize;
            let speed = f64::from_bits(s.speed.load(Ordering::Relaxed));
            let step = (src_rate * speed) / dev_rate;

            let decoded_frames = audio.decoded_frames.load(Ordering::Acquire) as usize;
            let max_idx = decoded_frames * ch;

            let mut pos = s.position.load(Ordering::Relaxed) as f64 / ch as f64;
            let out_frames = data.len() / ch;
            let mut filled = 0usize;

            // Intentamos leer el buffer. Si está bloqueado (escritura muy rápida), silenciamos temporalmente.
            let samples = match audio.samples.try_read() {
                Ok(guard) => guard,
                Err(_) => {
                    data.fill($zero);
                    return;
                }
            };

            while filled < out_frames {
                let src_idx = pos as usize;
                let buf_idx = src_idx * ch;

                if buf_idx >= max_idx {
                    let remaining = out_frames - filled;
                    for sample in &mut data[filled * ch..(filled + remaining) * ch] {
                        *sample = $zero;
                    }
                    if audio.is_done.load(Ordering::Acquire) {
                        s.is_playing.store(false, Ordering::Relaxed);
                    }
                    break;
                }

                let frac = (pos - src_idx as f64) as f32;
                let next_idx = ((src_idx + 1) * ch).min(max_idx.saturating_sub(ch));

                let base = filled * ch;
                // Verificamos de nuevo por seguridad (no debería pasar de max_idx)
                if next_idx + ch <= samples.len() {
                    for c in 0..ch {
                        let s0 = samples[buf_idx + c];
                        let s1 = samples[next_idx + c];
                        data[base + c] = $convert(s0 + (s1 - s0) * frac);
                    }
                } else {
                    for c in 0..ch {
                        data[base + c] = $zero;
                    }
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
    state: Arc<PlaybackState>,
) -> impl FnMut(&mut [f32], &cpal::OutputCallbackInfo) + Send + 'static {
    create_audio_callback!(state, f32, 0.0, |v: f32| v)
}

fn create_i16_callback(
    state: Arc<PlaybackState>,
) -> impl FnMut(&mut [i16], &cpal::OutputCallbackInfo) + Send + 'static {
    create_audio_callback!(state, i16, 0, |v: f32| (v * i16::MAX as f32) as i16)
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}
