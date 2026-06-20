use soundtouch::{Setting, SoundTouch};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

const CHUNK_FRAMES: usize = 4096;
const PREFILL_CHUNKS: usize = 4;

pub struct FullBufferPlayback {
    decoded_samples: Vec<f32>,
    sample_rate: u32,
    channels: usize,
    total_frames: usize,
    soundtouch: std::sync::Mutex<SoundTouch>,
    read_position: AtomicU64,
    is_playing: AtomicBool,
    is_done: AtomicBool,
    flushed: AtomicBool,
    tempo: AtomicU64,
    duration_ms: f64,
}

unsafe impl Send for FullBufferPlayback {}

impl FullBufferPlayback {
    pub fn new(
        decoded_samples: Vec<f32>,
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
            is_playing: AtomicBool::new(false),
            is_done: AtomicBool::new(false),
            flushed: AtomicBool::new(false),
            tempo: AtomicU64::new(initial_speed.to_bits()),
            duration_ms,
        }
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

        if let Ok(mut st) = self.soundtouch.lock() {
            st.clear();
            self.prefill_from(&mut st, clamped as usize);
        }
    }

    pub fn set_tempo(&self, tempo: f64) {
        self.tempo.store(tempo.to_bits(), Ordering::Relaxed);
        if let Ok(mut st) = self.soundtouch.lock() {
            st.set_tempo(tempo);
        }
    }

    pub fn feed_and_receive(&self, frames_needed: usize) -> Vec<f32> {
        let ch = self.channels;
        let mut output = vec![0.0f32; frames_needed * ch];
        let mut offset = 0usize;

        let Ok(mut st) = self.soundtouch.lock() else {
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

        self.mark_done_if_empty(&mut st, offset, frames_needed);
        output
    }

    pub fn get_position_ms(&self) -> f64 {
        let frame = self.read_position.load(Ordering::Relaxed);
        if self.sample_rate > 0 {
            let ms = (frame as f64 / self.sample_rate as f64) * 1000.0;
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
