use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub struct CalibrationState {
    pub position: AtomicU64,
    pub speed: AtomicU64,
    pub is_full_buffer: AtomicBool,
    pub is_playing: AtomicBool,
    pub soundtouch_unprocessed: AtomicU64,
    pub ring_buffer_samples: AtomicU64,
    pub output_buffer_frames: AtomicU64,
    pub os_latency_ms: AtomicU64,
    pub device_rate: AtomicU64,
}

impl CalibrationState {
    pub fn new() -> Self {
        Self {
            position: AtomicU64::new(0),
            speed: AtomicU64::new(1.0f64.to_bits()),
            is_full_buffer: AtomicBool::new(false),
            is_playing: AtomicBool::new(false),
            soundtouch_unprocessed: AtomicU64::new(0),
            ring_buffer_samples: AtomicU64::new(0),
            output_buffer_frames: AtomicU64::new(0),
            os_latency_ms: AtomicU64::new(0f64.to_bits()),
            device_rate: AtomicU64::new(48000.0f64.to_bits()),
        }
    }

    pub fn audible_position_ms(&self) -> f64 {
        let position = f64::from_bits(self.position.load(Ordering::Relaxed));
        let dr = f64::from_bits(self.device_rate.load(Ordering::Relaxed));
        if dr <= 0.0 {
            return position;
        }
        let st = self.soundtouch_unprocessed.load(Ordering::Relaxed) as f64;
        let rb = self.ring_buffer_samples.load(Ordering::Relaxed) as f64;
        let ob = self.output_buffer_frames.load(Ordering::Relaxed) as f64;
        let os = f64::from_bits(self.os_latency_ms.load(Ordering::Relaxed));
        let retraso = (st + rb + ob) / dr * 1000.0 + os;
        (position - retraso).max(0.0)
    }

    pub fn set_position_ms(&self, ms: f64) {
        self.position.store(ms.to_bits(), Ordering::Relaxed);
    }

    pub fn set_speed(&self, speed: f64) {
        self.speed.store(speed.to_bits(), Ordering::Relaxed);
    }

    pub fn set_soundtouch_unprocessed(&self, frames: u64) {
        self.soundtouch_unprocessed.store(frames, Ordering::Relaxed);
    }

    pub fn set_ring_buffer_samples(&self, samples: u64) {
        self.ring_buffer_samples.store(samples, Ordering::Relaxed);
    }

    pub fn set_output_buffer_frames(&self, frames: u64) {
        self.output_buffer_frames.store(frames, Ordering::Relaxed);
    }

    pub fn set_is_full_buffer(&self, val: bool) {
        self.is_full_buffer.store(val, Ordering::Relaxed);
    }

    pub fn set_is_playing(&self, val: bool) {
        self.is_playing.store(val, Ordering::Relaxed);
    }

    pub fn set_device_rate(&self, rate: f64) {
        self.device_rate.store(rate.to_bits(), Ordering::Relaxed);
    }

    pub fn set_os_latency_ms(&self, ms: f64) {
        self.os_latency_ms.store(ms.to_bits(), Ordering::Relaxed);
    }
}
