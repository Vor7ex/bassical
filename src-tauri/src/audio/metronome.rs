use crate::models::tab::TimingPoint;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

const BEAT_WAV: &[u8] = include_bytes!("../../../public/beat.wav");
const DOWNBEAT_WAV: &[u8] = include_bytes!("../../../public/downbeat.wav");

pub struct ClickBank {
    pub hi: Arc<Vec<f32>>,
    pub lo: Arc<Vec<f32>>,
}

fn read_wav_mono_f32(bytes: &[u8]) -> Vec<f32> {
    let num_channels = u16::from_le_bytes([bytes[22], bytes[23]]) as usize;
    let bits = u16::from_le_bytes([bytes[34], bytes[35]]) as usize;

    let data = &bytes[44..];
    let frame_count = data.len() / (num_channels * bits / 8);
    let mut raw = Vec::with_capacity(frame_count);

    for i in 0..frame_count {
        let offset = i * num_channels * (bits / 8);
        let sample = match bits {
            16 => {
                let b = &data[offset..offset + 2];
                i16::from_le_bytes([b[0], b[1]]) as f32 / i16::MAX as f32
            }
            24 => {
                let mut val = (data[offset + 2] as i32) << 16
                    | (data[offset + 1] as i32) << 8
                    | (data[offset] as i32);
                if val & 0x800000 != 0 {
                    val |= !0xFFFFFF;
                }
                val as f32 / 8388607.0f32
            }
            _ => 0.0,
        };
        raw.push(sample);
    }

    if num_channels == 2 {
        (0..raw.len() / 2)
            .map(|i| (raw[i * 2] + raw[i * 2 + 1]) * 0.5)
            .collect()
    } else {
        raw
    }
}

impl ClickBank {
    pub fn new(_sample_rate: u32) -> Self {
        Self {
            hi: Arc::new(read_wav_mono_f32(DOWNBEAT_WAV)),
            lo: Arc::new(read_wav_mono_f32(BEAT_WAV)),
        }
    }
}

fn mix_click_samples(output: &mut [f32], click: &[f32], offset: usize) {
    for (i, &sample) in click.iter().enumerate() {
        let idx = offset + i;
        if idx < output.len() {
            output[idx] += sample;
        }
    }
}

#[derive(Debug, Clone)]
struct MetronomeBeat {
    sample_index: u64,
    is_downbeat: bool,
}

#[derive(Debug, Clone)]
pub struct MetronomeGrid {
    beats: Vec<MetronomeBeat>,
}

fn add_segment_beats(
    beats: &mut Vec<MetronomeBeat>,
    tp: &TimingPoint,
    next_offset_ms: f64,
    total_frames: u64,
    rate: f64,
) {
    let ts = tp
        .time_signature
        .clone()
        .unwrap_or(crate::models::tab::TimeSignature::DEFAULT);
    let beat_period = 60.0 * rate / tp.bpm;
    if beat_period <= 0.0 || tp.bpm <= 0.0 {
        return;
    }

    let first_beat_sample = (tp.offset_ms / 1000.0 * rate).round() as u64;
    let segment_end_sample = (next_offset_ms / 1000.0 * rate).round() as u64;
    let mut beat_idx: u64 = 0;

    loop {
        let current_sample =
            (first_beat_sample as f64 + beat_idx as f64 * beat_period).round() as u64;
        if current_sample >= segment_end_sample || current_sample >= total_frames {
            break;
        }
        beats.push(MetronomeBeat {
            sample_index: current_sample,
            is_downbeat: beat_idx.is_multiple_of(ts.numerator as u64),
        });
        beat_idx += 1;
    }
}

impl MetronomeGrid {
    fn compute(tps: &[TimingPoint], sample_rate: u32, total_frames: u64) -> Self {
        if tps.is_empty() {
            return Self { beats: Vec::new() };
        }

        let mut sorted = tps.to_vec();
        sorted.sort_by(|a, b| {
            a.offset_ms
                .partial_cmp(&b.offset_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let rate = sample_rate as f64;
        let mut beats = Vec::new();

        for seg_idx in 0..sorted.len() {
            let next_offset_ms = if seg_idx + 1 < sorted.len() {
                sorted[seg_idx + 1].offset_ms
            } else {
                total_frames as f64 / rate * 1000.0
            };
            add_segment_beats(
                &mut beats,
                &sorted[seg_idx],
                next_offset_ms,
                total_frames,
                rate,
            );
        }

        beats.sort_by_key(|b| b.sample_index);
        Self { beats }
    }

    fn mix_into(
        &self,
        output: &mut [f32],
        channels: usize,
        block_start_sample: u64,
        bank: &ClickBank,
    ) {
        let block_end = block_start_sample + (output.len() / channels) as u64;
        let first = self
            .beats
            .partition_point(|b| b.sample_index < block_start_sample);

        for beat in &self.beats[first..] {
            if beat.sample_index >= block_end {
                break;
            }
            let offset = (beat.sample_index - block_start_sample) as usize * channels;
            let click = if beat.is_downbeat { &bank.hi } else { &bank.lo };
            mix_click_samples(output, click, offset);
        }
    }
}

pub struct MetronomeState {
    pub enabled: AtomicBool,
    pub click_bank: ClickBank,
    grid: RwLock<Arc<MetronomeGrid>>,
    pub device_rate: AtomicU64,
}

impl MetronomeState {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            enabled: AtomicBool::new(false),
            click_bank: ClickBank::new(sample_rate),
            grid: RwLock::new(Arc::new(MetronomeGrid { beats: Vec::new() })),
            device_rate: AtomicU64::new((sample_rate as f64).to_bits()),
        }
    }

    pub fn set_grid(&self, tps: &[TimingPoint], total_frames: u64) {
        let rate = f64::from_bits(self.device_rate.load(Ordering::Relaxed)) as u32;
        let grid = MetronomeGrid::compute(tps, rate, total_frames);
        *self.grid.write().unwrap() = Arc::new(grid);
    }

    pub fn mix_enabled(&self, output: &mut [f32], channels: usize, current_sample: u64) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }
        let grid = self.grid.read().unwrap();
        grid.mix_into(output, channels, current_sample, &self.click_bank);
    }
}
