use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::decoder::{probe_file, AudioMetadata, StreamingDecoder};

const PEAK_BINS: usize = 2000;
const RING_BUFFER_SECONDS: usize = 2;

type RbProd = ringbuf::CachingProd<Arc<HeapRb<f32>>>;
type RbCons = ringbuf::CachingCons<Arc<HeapRb<f32>>>;

pub struct StreamingState {
    #[allow(dead_code)]
    producer: Mutex<RbProd>,
    consumer: Mutex<RbCons>,
    pub metadata: AudioMetadata,
    decoded_frames: AtomicU64,
    peaks: Mutex<Vec<f32>>,
    is_done: AtomicBool,
    is_seeking: AtomicBool,
    seek_target_ms: AtomicU64,
    needs_buffer_swap: AtomicBool,
    is_playing: AtomicBool,
    decode_immediately: AtomicBool,
    push_to_buffer: AtomicBool,
    device_channels: usize,
    device_sample_rate: f64,
}

impl StreamingState {
    fn channels(&self) -> usize {
        self.device_channels
    }

    fn set_consumer(&self, consumer: RbCons) {
        let mut guard = self.consumer.lock().unwrap();
        *guard = consumer;
    }

    pub fn set_decode_immediately(&self, val: bool) {
        self.decode_immediately.store(val, Ordering::Release);
    }

    fn get_decode_progress(&self) -> f64 {
        let total = self.metadata.total_frames;
        if total == 0 {
            return 1.0;
        }
        let decoded = self.decoded_frames.load(Ordering::Relaxed);
        (decoded as f64 / total as f64).min(1.0)
    }

    pub fn get_peaks(&self) -> Vec<f32> {
        self.peaks.lock().unwrap().clone()
    }

    fn get_duration_ms(&self) -> f64 {
        self.metadata.duration_ms
    }

    pub fn new(metadata: AudioMetadata, device_channels: usize, device_sample_rate: f64) -> Self {
        Self {
            producer: Mutex::new(HeapRb::<f32>::new(1).split().0),
            consumer: Mutex::new(HeapRb::<f32>::new(1).split().1),
            metadata,
            decoded_frames: AtomicU64::new(0),
            peaks: Mutex::new(vec![0.0; PEAK_BINS]),
            is_done: AtomicBool::new(false),
            is_seeking: AtomicBool::new(false),
            seek_target_ms: AtomicU64::new(0),
            needs_buffer_swap: AtomicBool::new(false),
            is_playing: AtomicBool::new(false),
            decode_immediately: AtomicBool::new(false),
            push_to_buffer: AtomicBool::new(false),
            device_channels,
            device_sample_rate,
        }
    }

    #[allow(dead_code)]
    fn init_buffer(&self) {
        let capacity = self.metadata.sample_rate as usize * self.channels() * RING_BUFFER_SECONDS;
        let rb = HeapRb::<f32>::new(capacity);
        let (prod, cons) = rb.split();
        *self.producer.lock().unwrap() = prod;
        *self.consumer.lock().unwrap() = cons;
        self.decoded_frames.store(0, Ordering::Release);
        self.is_done.store(false, Ordering::Release);
    }
}

struct PlaybackState {
    position: AtomicU64,
    is_playing: AtomicBool,
    speed: AtomicU64,
    device_rate: f64,
    streaming: Mutex<Option<Arc<StreamingState>>>,
    playback: Mutex<Option<Arc<StreamingState>>>,
}

pub struct AudioPlaybackInfo {
    pub duration_ms: f64,
    pub sample_rate: u32,
    pub channels: u16,
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
            position: AtomicU64::new(0),
            is_playing: AtomicBool::new(false),
            speed: AtomicU64::new(1.0f64.to_bits()),
            device_rate,
            streaming: Mutex::new(None),
            playback: Mutex::new(None),
        });

        let mut engine = AudioEngine {
            state,
            stream: None,
        };
        if let Err(e) = engine.create_stream() {
            eprintln!("Failed to create initial stream: {}", e);
        }
        engine
    }

    pub fn set_current_stream(&mut self, streaming: Arc<StreamingState>) {
        self.state.position.store(0, Ordering::Relaxed);
        self.state.speed.store(1.0f64.to_bits(), Ordering::Relaxed);
        *self.state.streaming.lock().unwrap() = Some(streaming);
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
        let pb = self.state.playback.lock().unwrap();
        if let Some(ref s) = *pb {
            s.is_playing.store(true, Ordering::Relaxed);
        } else if let Some(ref s) = *self.state.streaming.lock().unwrap() {
            s.is_playing.store(true, Ordering::Relaxed);
        }
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), String> {
        self.state.is_playing.store(false, Ordering::Relaxed);
        let pb = self.state.playback.lock().unwrap();
        if let Some(ref s) = *pb {
            s.is_playing.store(false, Ordering::Relaxed);
        } else if let Some(ref s) = *self.state.streaming.lock().unwrap() {
            s.is_playing.store(false, Ordering::Relaxed);
        }
        Ok(())
    }

    pub fn seek(&self, position_ms: f64) -> Result<(), String> {
        let streaming = {
            let pb = self.state.playback.lock().unwrap();
            if let Some(ref s) = *pb {
                s.clone()
            } else {
                let guard = self.state.streaming.lock().unwrap();
                match *guard {
                    Some(ref s) => s.clone(),
                    None => return Err("No hay audio cargado".to_string()),
                }
            }
        };

        streaming
            .seek_target_ms
            .store(position_ms.to_bits(), Ordering::Release);
        streaming.is_seeking.store(true, Ordering::Release);

        let frame = (position_ms / 1000.0 * streaming.metadata.sample_rate as f64) as u64;
        self.state
            .position
            .store(frame * streaming.channels() as u64, Ordering::Relaxed);

        Ok(())
    }

    pub fn set_speed(&mut self, speed: f64) -> Result<(), String> {
        let clamped = speed.clamp(0.25, 1.0);
        self.state.speed.store(clamped.to_bits(), Ordering::Relaxed);
        Ok(())
    }

    pub fn get_position_ms(&self) -> f64 {
        let streaming = {
            let guard = self.state.streaming.lock().unwrap();
            guard.clone()
        };
        if let Some(s) = streaming {
            let pos = self.state.position.load(Ordering::Relaxed);
            let ch = s.channels() as f64;
            let rate = s.metadata.sample_rate as f64;
            if rate > 0.0 && ch > 0.0 {
                return (pos as f64 / ch / rate) * 1000.0;
            }
        }
        0.0
    }

    pub fn get_duration_ms(&self) -> f64 {
        let streaming = {
            let guard = self.state.streaming.lock().unwrap();
            guard.clone()
        };
        streaming.map(|s| s.get_duration_ms()).unwrap_or(0.0)
    }

    pub fn get_decode_progress(&self) -> f64 {
        let streaming = {
            let guard = self.state.streaming.lock().unwrap();
            guard.clone()
        };
        streaming.map(|s| s.get_decode_progress()).unwrap_or(0.0)
    }

    pub fn get_peaks(&self) -> Vec<f32> {
        let streaming = {
            let guard = self.state.streaming.lock().unwrap();
            guard.clone()
        };
        streaming.map(|s| s.get_peaks()).unwrap_or_default()
    }

    pub fn is_playing(&self) -> bool {
        self.state.is_playing.load(Ordering::Relaxed)
    }

    pub fn device_rate(&self) -> f64 {
        self.state.device_rate
    }

    pub fn channels(&self) -> usize {
        let host = cpal::default_host();
        host.default_output_device()
            .and_then(|d| d.default_output_config().ok())
            .map(|c| c.channels() as usize)
            .unwrap_or(2)
    }

    pub fn start_playback(&mut self, path: String) -> Result<AudioPlaybackInfo, String> {
        let existing = self.state.playback.lock().unwrap().clone();
        if let Some(ref pb) = existing {
            if !pb.is_done.load(Ordering::Acquire) {
                pb.is_playing.store(true, Ordering::Relaxed);
                self.state.is_playing.store(true, Ordering::Relaxed);
                return Ok(AudioPlaybackInfo {
                    duration_ms: pb.metadata.duration_ms,
                    sample_rate: pb.metadata.sample_rate,
                    channels: pb.metadata.channels,
                });
            }
        }

        let metadata = probe_file(&path)?;
        let device_rate = self.state.device_rate;
        let streaming = Arc::new(StreamingState::new(
            metadata.clone(),
            self.channels(),
            device_rate,
        ));
        streaming.push_to_buffer.store(true, Ordering::Release);
        streaming.is_playing.store(true, Ordering::Release);

        let info = AudioPlaybackInfo {
            duration_ms: metadata.duration_ms,
            sample_rate: metadata.sample_rate,
            channels: metadata.channels,
        };

        *self.state.playback.lock().unwrap() = Some(streaming.clone());
        self.state.position.store(0, Ordering::Relaxed);
        self.state.speed.store(1.0f64.to_bits(), Ordering::Relaxed);
        self.state.is_playing.store(true, Ordering::Relaxed);

        spawn_decoder_thread(path, streaming);
        Ok(info)
    }

    pub fn stop_playback(&mut self) {
        if let Some(pb) = self.state.playback.lock().unwrap().take() {
            pb.is_playing.store(false, Ordering::Relaxed);
        }
        self.state.is_playing.store(false, Ordering::Relaxed);
    }
}

fn log_error(err: cpal::StreamError) {
    eprintln!("Error en el stream de audio: {}", err);
}

macro_rules! create_streaming_callback {
    ($state:expr, $sample_type:ty, $zero:expr, $convert:expr) => {{
        let s = $state.clone();
        move |data: &mut [$sample_type], _: &cpal::OutputCallbackInfo| {
            if !s.is_playing.load(Ordering::Relaxed) {
                data.fill($zero);
                return;
            }

            let streaming = {
                let pb = s.playback.lock().unwrap();
                if let Some(ref st) = *pb {
                    st.clone()
                } else {
                    drop(pb);
                    let guard = s.streaming.lock().unwrap();
                    match *guard {
                        Some(ref st) => st.clone(),
                        None => {
                            data.fill($zero);
                            return;
                        }
                    }
                }
            };

            if streaming.is_seeking.load(Ordering::Acquire) {
                data.fill($zero);
                return;
            }

            let mut consumer = streaming.consumer.lock().unwrap();

            for sample in data.iter_mut() {
                *sample = match consumer.try_pop() {
                    Some(s) => $convert(s),
                    None => {
                        if streaming.is_done.load(Ordering::Acquire) {
                            s.is_playing.store(false, Ordering::Relaxed);
                        }
                        $zero
                    }
                };
            }

            let ch = streaming.channels() as f64;
            let rate = streaming.metadata.sample_rate as f64;
            let frames_out = data.len() as f64 / ch;
            let src_step = rate / s.device_rate;
            let speed = f64::from_bits(s.speed.load(Ordering::Relaxed));
            let current_pos = s.position.load(Ordering::Relaxed) as f64;
            let src_frames = current_pos / ch;
            let new_src_frames = src_frames + frames_out * src_step * speed;
            s.position
                .store((new_src_frames * ch) as u64, Ordering::Relaxed);
        }
    }};
}

fn create_f32_callback(
    state: Arc<PlaybackState>,
) -> impl FnMut(&mut [f32], &cpal::OutputCallbackInfo) + Send + 'static {
    create_streaming_callback!(state, f32, 0.0, |v: f32| v)
}

fn create_i16_callback(
    state: Arc<PlaybackState>,
) -> impl FnMut(&mut [i16], &cpal::OutputCallbackInfo) + Send + 'static {
    create_streaming_callback!(state, i16, 0, |v: f32| { (v * i16::MAX as f32) as i16 })
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn resample_chunk(input: &[f32], source_channels: usize, ratio: f32) -> Vec<f32> {
    let source_frames = input.len() / source_channels;
    if source_frames == 0 || ratio == 0.0 {
        return Vec::new();
    }

    let out_frames = (source_frames as f32 / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(out_frames * source_channels);

    for out_idx in 0..out_frames {
        let src_pos = out_idx as f32 * ratio;
        let idx0 = src_pos as usize;
        let frac = src_pos - idx0 as f32;
        let idx1 = (idx0 + 1).min(source_frames - 1);

        for ch in 0..source_channels {
            let s0 = input[idx0 * source_channels + ch];
            let s1 = input[idx1 * source_channels + ch];
            output.push(s0 + (s1 - s0) * frac);
        }
    }

    output
}

struct PeakContext {
    total_source_frames: usize,
    channels: usize,
}

fn update_peaks_incremental(
    peaks: &mut [f32],
    chunk: &[f32],
    offset_source_frames: usize,
    ctx: &PeakContext,
) {
    if ctx.total_source_frames == 0 || chunk.is_empty() {
        return;
    }
    let frames = chunk.len() / ctx.channels;
    for frame in 0..frames {
        let mono: f32 = (0..ctx.channels)
            .map(|c| chunk[frame * ctx.channels + c].abs())
            .sum::<f32>()
            / ctx.channels as f32;
        let src_frame = offset_source_frames + frame;
        let bin = (src_frame * PEAK_BINS / ctx.total_source_frames).min(PEAK_BINS - 1);
        if mono > peaks[bin] {
            peaks[bin] = mono;
        }
    }
}

fn handle_seek_if_requested(streaming: &StreamingState, decoder: &mut StreamingDecoder) -> u64 {
    if !streaming.is_seeking.load(Ordering::Acquire) {
        return 0;
    }
    let ms = f64::from_bits(streaming.seek_target_ms.load(Ordering::Acquire));
    let _ = decoder.seek_to_ms(ms);
    streaming.needs_buffer_swap.store(true, Ordering::Release);

    let sample_rate = streaming.metadata.sample_rate as f64;
    let channels = streaming.channels() as f64;
    (ms / 1000.0 * sample_rate * channels) as u64
}

struct DecodeParams {
    source_channels: usize,
    ratio: f32,
    source_frames_decoded: u64,
}

fn push_with_backpressure(
    streaming: &StreamingState,
    prod: &mut ringbuf::CachingProd<Arc<HeapRb<f32>>>,
    samples: &[f32],
) {
    let mut offset = 0;
    while offset < samples.len() {
        if streaming.needs_buffer_swap.load(Ordering::Acquire)
            || streaming.is_seeking.load(Ordering::Acquire)
        {
            break;
        }
        let pushed = prod.push_slice(&samples[offset..]);
        if pushed == 0 {
            thread::sleep(Duration::from_millis(1));
        } else {
            offset += pushed;
        }
    }
}

fn process_decoded_chunk(
    streaming: &StreamingState,
    prod: &mut ringbuf::CachingProd<Arc<HeapRb<f32>>>,
    chunk: &[f32],
    params: &mut DecodeParams,
) {
    let resampled = resample_chunk(chunk, params.source_channels, params.ratio);
    let total_source = streaming.metadata.total_frames;

    if !streaming.push_to_buffer.load(Ordering::Acquire) && total_source > 0 {
        let mut peaks = streaming.peaks.lock().unwrap();
        let ctx = PeakContext {
            total_source_frames: total_source as usize,
            channels: streaming.channels(),
        };
        update_peaks_incremental(
            &mut peaks,
            &resampled,
            params.source_frames_decoded as usize,
            &ctx,
        );
    }

    if streaming.push_to_buffer.load(Ordering::Acquire)
        || !streaming.decode_immediately.load(Ordering::Acquire)
    {
        push_with_backpressure(streaming, prod, &resampled);
    }

    let chunk_frames = chunk.len() / params.source_channels;
    params.source_frames_decoded += chunk_frames as u64;
    streaming
        .decoded_frames
        .store(params.source_frames_decoded, Ordering::Release);
}

fn swap_buffer_on_seek(
    streaming: &StreamingState,
    prod: &mut ringbuf::CachingProd<Arc<HeapRb<f32>>>,
) {
    if !streaming.needs_buffer_swap.load(Ordering::Acquire) {
        return;
    }
    streaming.needs_buffer_swap.store(false, Ordering::Release);
    streaming.decoded_frames.store(0, Ordering::Release);
    streaming.is_done.store(false, Ordering::Release);
    let capacity =
        streaming.metadata.sample_rate as usize * streaming.channels() * RING_BUFFER_SECONDS;
    let rb = HeapRb::<f32>::new(capacity);
    let (new_prod, new_cons) = rb.split();
    *prod = new_prod;
    streaming.set_consumer(new_cons);
    streaming.is_seeking.store(false, Ordering::Release);
}

pub fn spawn_decoder_thread(path: String, streaming: Arc<StreamingState>) {
    thread::spawn(move || {
        run_decoder_thread(path, streaming);
    });
}

fn run_decoder_thread(path: String, streaming: Arc<StreamingState>) {
    let mut decoder = match StreamingDecoder::open(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error abriendo archivo: {}", e);
            streaming.is_done.store(true, Ordering::Release);
            return;
        }
    };

    let capacity =
        streaming.metadata.sample_rate as usize * streaming.channels() * RING_BUFFER_SECONDS;
    let rb = HeapRb::<f32>::new(capacity);
    let (mut prod, cons) = rb.split();

    streaming.set_consumer(cons);

    let source_channels = decoder.channels();
    let src_rate = streaming.metadata.sample_rate as f64;
    let dev_rate = streaming.device_sample_rate;
    let ratio = (src_rate / dev_rate) as f32;

    let mut params = DecodeParams {
        source_channels,
        ratio,
        source_frames_decoded: 0,
    };

    loop {
        while !streaming.is_playing.load(Ordering::Acquire) {
            if streaming.decode_immediately.load(Ordering::Acquire) {
                break;
            }
            if streaming.is_seeking.load(Ordering::Acquire) {
                break;
            }
            if streaming.is_done.load(Ordering::Acquire) {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }

        if !streaming.decode_immediately.load(Ordering::Acquire) {
            params.source_frames_decoded = handle_seek_if_requested(&streaming, &mut decoder);
            swap_buffer_on_seek(&streaming, &mut prod);
        }

        match decoder.decode_chunk(4096) {
            Ok(chunk) if chunk.is_empty() => break,
            Ok(chunk) => {
                process_decoded_chunk(&streaming, &mut prod, &chunk, &mut params);
            }
            Err(e) => {
                eprintln!("Error de decodificación: {}", e);
                break;
            }
        }

        if decoder.is_done() {
            break;
        }
    }

    streaming.is_done.store(true, Ordering::Release);
}
