use soundtouch::{Setting, SoundTouch};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::calibration::CalibrationState;

const CHUNK_FRAMES: usize = 4096;
const PREFILL_CHUNKS: usize = 4;

pub struct FullBufferPlayback {
    decoded_samples: Arc<Vec<f32>>,
    sample_rate: u32,
    channels: usize,
    total_frames: usize,
    soundtouch: std::sync::Mutex<SoundTouch>,
    read_position: AtomicU64,
    output_position: AtomicU64,
    is_playing: AtomicBool,
    is_done: AtomicBool,
    flushed: AtomicBool,
    tempo: AtomicU64,
    duration_ms: f64,
    path: String,
}

unsafe impl Send for FullBufferPlayback {}

impl FullBufferPlayback {
    pub fn new(
        decoded_samples: Arc<Vec<f32>>,
        sample_rate: u32,
        channels: usize,
        initial_speed: f64,
    ) -> Self {
        let total_frames = decoded_samples.len() / channels;
        let duration_ms = if sample_rate > 0 {
            (total_frames as f64 / sample_rate as f64) * 1000.0
        } else {
            0.0
        };

        let mut st = SoundTouch::new();
        st.set_channels(channels as u32)
            .set_sample_rate(sample_rate)
            .set_tempo(initial_speed)
            .set_setting(Setting::UseQuickseek, 1);

        let mut initial_read = 0usize;
        let prefill_frames = (CHUNK_FRAMES * PREFILL_CHUNKS).min(total_frames);
        if prefill_frames > 0 {
            let chunk = &decoded_samples[..prefill_frames * channels];
            st.put_samples(chunk, prefill_frames);
            initial_read = prefill_frames;
        }

        Self {
            decoded_samples,
            sample_rate,
            channels,
            total_frames,
            soundtouch: std::sync::Mutex::new(st),
            read_position: AtomicU64::new(initial_read as u64),
            output_position: AtomicU64::new(0),
            is_playing: AtomicBool::new(false),
            is_done: AtomicBool::new(false),
            flushed: AtomicBool::new(false),
            tempo: AtomicU64::new(initial_speed.to_bits()),
            duration_ms,
            path: String::new(),
        }
    }

    pub fn set_path(&mut self, path: String) {
        self.path = path;
    }

    pub fn play(&self) {
        self.is_playing.store(true, Ordering::Relaxed);
    }

    pub fn pause(&self) {
        self.is_playing.store(false, Ordering::Relaxed);
    }

    pub fn seek_to_ms(&self, ms: f64) {
        let frame = ((ms / 1000.0) * self.sample_rate as f64) as u64;
        let clamped = frame.min(self.total_frames as u64);
        self.read_position.store(clamped, Ordering::Relaxed);
        self.is_done.store(false, Ordering::Relaxed);
        self.flushed.store(false, Ordering::Relaxed);

        let tempo = f64::from_bits(self.tempo.load(Ordering::Relaxed));
        let output_base = if tempo > 0.0 {
            (clamped as f64 / tempo) as u64
        } else {
            0
        };
        self.output_position.store(output_base, Ordering::Relaxed);

        if let Ok(mut st) = self.soundtouch.lock() {
            st.clear();
            self.prefill_from(&mut st, clamped as usize);
        }
    }

    pub fn set_tempo(&self, tempo: f64) {
        let old_tempo = f64::from_bits(self.tempo.load(Ordering::Relaxed));
        if (old_tempo - tempo).abs() < f64::EPSILON {
            return;
        }

        self.rescale_output_position(old_tempo, tempo);
        self.tempo.store(tempo.to_bits(), Ordering::Relaxed);

        if let Ok(mut st) = self.soundtouch.lock() {
            self.apply_tempo_transition(&mut st, old_tempo, tempo);
        }
    }

    fn rescale_output_position(&self, old_tempo: f64, new_tempo: f64) {
        if old_tempo <= 0.0 || new_tempo <= 0.0 {
            return;
        }
        let current_output = self.output_position.load(Ordering::Relaxed) as f64;
        let rescaled = (current_output * old_tempo / new_tempo) as u64;
        self.output_position.store(rescaled, Ordering::Relaxed);
    }

    fn apply_tempo_transition(&self, st: &mut SoundTouch, old_tempo: f64, new_tempo: f64) {
        let was_bypass = (old_tempo - 1.0).abs() < 1e-6;
        let is_bypass = (new_tempo - 1.0).abs() < 1e-6;

        if was_bypass && !is_bypass {
            st.clear();
            st.set_tempo(new_tempo);
            let read_pos = self.read_position.load(Ordering::Relaxed) as usize;
            self.prefill_from(st, read_pos);
        } else if !was_bypass && is_bypass {
            st.clear();
            let new_output = self.output_position.load(Ordering::Relaxed);
            self.read_position.store(new_output, Ordering::Relaxed);
        } else {
            st.set_tempo(new_tempo);
        }
    }

    pub fn feed_and_receive(&self, frames_needed: usize, calib: &CalibrationState) -> Vec<f32> {
        let ch = self.channels;
        let mut output = vec![0.0f32; frames_needed * ch];

        if self.is_bypass() {
            self.feed_direct(&mut output, frames_needed, calib);
            return output;
        }

        let mut offset = 0usize;

        let Ok(mut st) = self.soundtouch.lock() else {
            calib.set_soundtouch_unprocessed(0);
            return output;
        };

        while offset < frames_needed {
            let ready = st.num_samples() as usize;
            if ready > 0 {
                let to_read = (frames_needed - offset).min(ready);
                let buf = &mut output[offset * ch..(offset + to_read) * ch];
                st.receive_samples(buf, to_read);
                offset += to_read;
                if offset >= frames_needed {
                    break;
                }
            }

            if !self.feed_or_flush(&mut st) {
                break;
            }
        }

        calib.set_soundtouch_unprocessed(st.num_unprocessed_samples() as u64);
        self.output_position
            .fetch_add(offset as u64, Ordering::Relaxed);
        self.mark_done_if_empty(&mut st, offset, frames_needed);
        output
    }

    fn feed_direct(&self, output: &mut [f32], frames_needed: usize, calib: &CalibrationState) {
        let ch = self.channels;
        let read_pos = self.read_position.load(Ordering::Relaxed) as usize;
        let remaining = self.total_frames.saturating_sub(read_pos);
        let to_copy = frames_needed.min(remaining);
        let start = read_pos * ch;
        let sample_count = to_copy * ch;
        output[..sample_count].copy_from_slice(&self.decoded_samples[start..start + sample_count]);
        self.read_position
            .fetch_add(to_copy as u64, Ordering::Relaxed);
        self.output_position
            .fetch_add(to_copy as u64, Ordering::Relaxed);
        calib.set_soundtouch_unprocessed(0);
        if read_pos + to_copy >= self.total_frames {
            self.is_done.store(true, Ordering::Relaxed);
        }
    }

    fn is_bypass(&self) -> bool {
        let t = f64::from_bits(self.tempo.load(Ordering::Relaxed));
        (t - 1.0).abs() < 1e-6
    }

    pub fn get_position_ms(&self) -> f64 {
        let tempo = f64::from_bits(self.tempo.load(Ordering::Relaxed));
        let output_frames = self.output_position.load(Ordering::Relaxed) as f64;
        let source_frames = output_frames * tempo;
        if self.sample_rate > 0 {
            let ms = (source_frames / self.sample_rate as f64) * 1000.0;
            ms.min(self.duration_ms)
        } else {
            0.0
        }
    }

    pub fn is_done(&self) -> bool {
        self.is_done.load(Ordering::Relaxed)
    }

    pub fn duration_ms(&self) -> f64 {
        self.duration_ms
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn tempo_atomic(&self) -> u64 {
        self.tempo.load(Ordering::Relaxed)
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    fn prefill_from(&self, st: &mut SoundTouch, from_frame: usize) {
        let remaining = self.total_frames.saturating_sub(from_frame);
        let prefill = (CHUNK_FRAMES * PREFILL_CHUNKS).min(remaining);
        if prefill > 0 {
            let start = from_frame * self.channels;
            let end = start + prefill * self.channels;
            let chunk = &self.decoded_samples[start..end];
            st.put_samples(chunk, prefill);
            self.read_position
                .store((from_frame + prefill) as u64, Ordering::Relaxed);
        }
    }

    fn feed_or_flush(&self, st: &mut SoundTouch) -> bool {
        let ch = self.channels;
        let read_pos = self.read_position.load(Ordering::Relaxed) as usize;
        let remaining = self.total_frames.saturating_sub(read_pos);

        if remaining == 0 {
            if self.flushed.load(Ordering::Relaxed) {
                return false;
            }
            st.flush();
            self.flushed.store(true, Ordering::Relaxed);
            return true;
        }

        let frames_to_feed = CHUNK_FRAMES.min(remaining);
        let start = read_pos * ch;
        let end = start + frames_to_feed * ch;
        let chunk = &self.decoded_samples[start..end];
        st.put_samples(chunk, frames_to_feed);
        self.read_position
            .store((read_pos + frames_to_feed) as u64, Ordering::Relaxed);
        true
    }

    fn mark_done_if_empty(&self, st: &mut SoundTouch, offset: usize, frames_needed: usize) {
        if !self.flushed.load(Ordering::Relaxed) {
            return;
        }
        let ready = st.num_samples() as usize;
        if ready == 0 && offset < frames_needed {
            self.is_done.store(true, Ordering::Relaxed);
        }
    }
}
