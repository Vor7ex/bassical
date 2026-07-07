use crate::models::tab::TimingPoint;
use arc_swap::ArcSwap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

const BEAT_WAV: &[u8] = include_bytes!("../../../public/beat.wav");
const DOWNBEAT_WAV: &[u8] = include_bytes!("../../../public/downbeat.wav");

const CLICK_PEAK: f32 = 0.7;
const FADE_IN_MS: f64 = 1.0;
const FADE_OUT_MS: f64 = 5.0;

#[derive(Default)]
struct WavInfo {
    sample_rate: u32,
    bits: u16,
    channels: u16,
    data: Vec<u8>,
}

fn parse_wav(bytes: &[u8]) -> Option<WavInfo> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return None;
    }

    let mut info = WavInfo::default();
    let mut pos = 12usize;

    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32::from_le_bytes([
            bytes[pos + 4],
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
        ]) as usize;
        pos += 8;

        let end = (pos + size).min(bytes.len());
        let chunk = &bytes[pos..end];

        match id {
            b"fmt " if chunk.len() >= 16 => {
                info.channels = u16::from_le_bytes([chunk[2], chunk[3]]);
                info.sample_rate = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                info.bits = u16::from_le_bytes([chunk[14], chunk[15]]);
            }
            b"data" => {
                info.data = chunk.to_vec();
            }
            _ => {}
        }

        pos = end;
        if size % 2 == 1 {
            pos += 1;
        }
    }

    if info.sample_rate == 0 || info.bits == 0 || info.channels == 0 {
        return None;
    }
    Some(info)
}

fn decode_pcm_to_mono_f32(info: &WavInfo) -> Vec<f32> {
    let data = &info.data;
    let bytes_per_sample = (info.bits / 8) as usize;
    let frame_size = bytes_per_sample * info.channels as usize;
    let frame_count = data.len() / frame_size;

    let mut mono = Vec::with_capacity(frame_count);

    for f in 0..frame_count {
        let frame_start = f * frame_size;
        let mut acc = 0.0f32;
        for ch in 0..info.channels as usize {
            let off = frame_start + ch * bytes_per_sample;
            let sample = match info.bits {
                16 => {
                    let v = i16::from_le_bytes([data[off], data[off + 1]]);
                    v as f32 / i16::MAX as f32
                }
                24 => {
                    let raw = (data[off] as i32)
                        | ((data[off + 1] as i32) << 8)
                        | ((data[off + 2] as i32) << 16);
                    let val = if raw & 0x800000 != 0 {
                        raw | !0xFFFFFF
                    } else {
                        raw
                    };
                    val as f32 / 8388607.0f32
                }
                32 => f32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]),
                _ => 0.0,
            };
            acc += sample;
        }
        mono.push(acc / info.channels as f32);
    }

    mono
}

fn resample_linear(input: &[f32], src_rate: u32, dst_rate: u32) -> Vec<f32> {
    if src_rate == dst_rate || input.is_empty() {
        return input.to_vec();
    }
    let ratio = src_rate as f64 / dst_rate as f64;
    let out_len = (input.len() as f64 / ratio).floor() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f64 * ratio;
        let idx0 = src_pos.floor() as usize;
        let frac = src_pos - idx0 as f64;
        let idx1 = (idx0 + 1).min(input.len() - 1);
        let s0 = input[idx0] as f64;
        let s1 = input[idx1] as f64;
        out.push((s0 + (s1 - s0) * frac) as f32);
    }
    out
}

fn normalize_and_envelope(samples: &mut [f32], rate: u32) {
    let peak = samples.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
    if peak <= f32::EPSILON {
        return;
    }
    let norm = CLICK_PEAK / peak;
    for s in samples.iter_mut() {
        *s *= norm;
    }

    let fade_in_samples = (FADE_IN_MS / 1000.0 * rate as f64).round() as usize;
    let fade_out_samples = (FADE_OUT_MS / 1000.0 * rate as f64).round() as usize;
    let len = samples.len();

    if fade_in_samples > 0 && fade_in_samples < len {
        for (i, s) in samples.iter_mut().take(fade_in_samples).enumerate() {
            let gain = i as f32 / fade_in_samples as f32;
            *s *= gain;
        }
    }

    if fade_out_samples > 0 && fade_out_samples < len {
        let start = len - fade_out_samples;
        for (i, s) in samples[start..].iter_mut().enumerate() {
            let gain = 1.0 - (i as f32 / fade_out_samples as f32);
            *s *= gain;
        }
    }
}

fn expand_to_channels(mono: &[f32], channels: usize) -> Vec<f32> {
    if channels == 1 {
        return mono.to_vec();
    }
    let mut out = Vec::with_capacity(mono.len() * channels);
    for &s in mono {
        for _ in 0..channels {
            out.push(s);
        }
    }
    out
}

fn build_click(bytes: &[u8], device_rate: u32, device_channels: usize) -> Arc<Vec<f32>> {
    let info = match parse_wav(bytes) {
        Some(i) => i,
        None => return Arc::new(Vec::new()),
    };
    let mono = decode_pcm_to_mono_f32(&info);
    let resampled = resample_linear(&mono, info.sample_rate, device_rate);
    let mut processed = resampled;
    normalize_and_envelope(&mut processed, device_rate);
    let expanded = expand_to_channels(&processed, device_channels);
    Arc::new(expanded)
}

pub struct ClickBank {
    pub hi: Arc<Vec<f32>>,
    pub lo: Arc<Vec<f32>>,
}

impl ClickBank {
    pub fn new(device_rate: u32, device_channels: usize) -> Self {
        Self {
            hi: build_click(DOWNBEAT_WAV, device_rate, device_channels),
            lo: build_click(BEAT_WAV, device_rate, device_channels),
        }
    }
}

#[derive(Debug, Clone)]
struct MetronomeBeat {
    sample_index: u64,
    is_downbeat: bool,
}

#[derive(Debug, Clone, Default)]
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
            return Self::default();
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
}

struct ActiveClick {
    samples: Arc<Vec<f32>>,
    cursor: usize,
    block_offset: usize,
}

pub struct MetronomeState {
    pub enabled: AtomicBool,
    pub click_bank: ArcSwap<ClickBank>,
    grid: ArcSwap<MetronomeGrid>,
    active: Mutex<Vec<ActiveClick>>,
    pub device_rate: AtomicU64,
    pub device_channels: AtomicU64,
    balance: AtomicU64,
}

impl MetronomeState {
    pub fn new(sample_rate: u32) -> Self {
        let channels = 2usize;
        Self::with_channels(sample_rate, channels)
    }

    pub fn with_channels(sample_rate: u32, channels: usize) -> Self {
        let bank = ClickBank::new(sample_rate, channels);
        let balance = 0.7f64;
        Self {
            enabled: AtomicBool::new(false),
            click_bank: ArcSwap::from_pointee(bank),
            grid: ArcSwap::from_pointee(MetronomeGrid::default()),
            active: Mutex::new(Vec::new()),
            device_rate: AtomicU64::new((sample_rate as f64).to_bits()),
            device_channels: AtomicU64::new(channels as u64),
            balance: AtomicU64::new(balance.to_bits()),
        }
    }

    pub fn reconfigure(&self, device_rate: u32, channels: usize) {
        let prev_rate = f64::from_bits(self.device_rate.load(Ordering::Relaxed)) as u32;
        let prev_ch = self.device_channels.load(Ordering::Relaxed) as usize;
        if prev_rate == device_rate && prev_ch == channels {
            return;
        }
        self.click_bank
            .store(Arc::new(ClickBank::new(device_rate, channels)));
        self.device_rate
            .store((device_rate as f64).to_bits(), Ordering::Relaxed);
        self.device_channels
            .store(channels as u64, Ordering::Relaxed);
        self.grid.store(Arc::new(MetronomeGrid::default()));
        self.active.lock().unwrap().clear();
    }

    pub fn set_grid(&self, tps: &[TimingPoint], total_frames: u64) {
        let rate = f64::from_bits(self.device_rate.load(Ordering::Relaxed)) as u32;
        let grid = MetronomeGrid::compute(tps, rate, total_frames);
        self.grid.store(Arc::new(grid));
        self.active.lock().unwrap().clear();
    }

    pub fn set_balance(&self, balance: f64) {
        let clamped = balance.clamp(0.0, 1.0);
        self.balance.store(clamped.to_bits(), Ordering::Relaxed);
    }

    pub fn get_balance(&self) -> f64 {
        f64::from_bits(self.balance.load(Ordering::Relaxed))
    }

    pub fn mix_block(
        &self,
        output: &mut [f32],
        channels: usize,
        block_start_sample: u64,
        click_gain: f32,
        speed: f64,
    ) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }
        if click_gain <= 0.0 {
            return;
        }

        let block_frames = output.len() / channels;
        if block_frames == 0 {
            return;
        }
        let song_frames = block_frames as f64 * speed;
        let block_end = block_start_sample + song_frames.round() as u64;

        let grid = self.grid.load();
        let bank = self.click_bank.load();
        let first = grid
            .beats
            .partition_point(|b| b.sample_index < block_start_sample);

        let mut active = self.active.lock().unwrap();

        for beat in &grid.beats[first..] {
            if beat.sample_index >= block_end {
                break;
            }
            let click = if beat.is_downbeat {
                bank.hi.clone()
            } else {
                bank.lo.clone()
            };
            if click.is_empty() {
                continue;
            }
            let song_offset = (beat.sample_index as f64 - block_start_sample as f64).max(0.0);
            let device_offset_frames = (song_offset / speed).round() as usize;
            let block_offset = device_offset_frames * channels;
            if block_offset >= output.len() {
                continue;
            }
            active.push(ActiveClick {
                samples: click,
                cursor: 0,
                block_offset,
            });
        }

        let mut i = 0;
        while i < active.len() {
            let block_offset = active[i].block_offset;
            let cursor = active[i].cursor;
            let samples = active[i].samples.clone();
            let remaining_click = samples.len() - cursor;
            let remaining_block = output.len() - block_offset;
            let n = remaining_click.min(remaining_block);

            for k in 0..n {
                let v = samples[cursor + k] * click_gain;
                let idx = block_offset + k;
                let mixed = output[idx] + v;
                output[idx] = mixed.clamp(-1.0, 1.0);
            }
            active[i].cursor += n;
            active[i].block_offset = 0;

            if active[i].cursor >= active[i].samples.len() {
                active.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_click(len: usize, channels: usize) -> Vec<f32> {
        let mut v = Vec::with_capacity(len * channels);
        for i in 0..len {
            let s = (CLICK_PEAK * (1.0 - i as f32 / len as f32)).max(0.0);
            for _ in 0..channels {
                v.push(s);
            }
        }
        v
    }

    fn state_with_click(rate: u32, channels: usize, click: Vec<f32>) -> MetronomeState {
        let s = MetronomeState::with_channels(rate, channels);
        s.click_bank.store(Arc::new(ClickBank {
            hi: Arc::new(click),
            lo: Arc::new(Vec::new()),
        }));
        s.enabled.store(true, Ordering::Relaxed);
        s.set_balance(1.0);
        s
    }

    fn set_single_beat(state: &MetronomeState, sample_index: u64, downbeat: bool) {
        state.grid.store(Arc::new(MetronomeGrid {
            beats: vec![MetronomeBeat {
                sample_index,
                is_downbeat: downbeat,
            }],
        }));
    }

    fn run_blocks(
        state: &MetronomeState,
        channels: usize,
        block_frames: usize,
        n_blocks: usize,
        start: u64,
        click_gain: f32,
    ) -> Vec<f32> {
        let mut out = Vec::new();
        let mut s = start;
        for _ in 0..n_blocks {
            let mut buf = vec![0.0f32; block_frames * channels];
            state.mix_block(&mut buf, channels, s, click_gain, 1.0);
            out.extend_from_slice(&buf);
            s += block_frames as u64;
        }
        out
    }

    #[test]
    fn test_click_reproduced_across_blocks() {
        let channels = 2;
        let rate = 48000;
        let click = synthetic_click(5000, channels);
        let click_len = click.len();
        let state = state_with_click(rate, channels, click.clone());
        set_single_beat(&state, 100, true);

        let block_frames = 240;
        let needed_blocks = (100 + click_len / channels) / block_frames + 2;
        let out = run_blocks(
            &state,
            channels,
            block_frames,
            needed_blocks as usize,
            0,
            1.0,
        );

        let start = 100 * channels;
        let end = start + click_len;
        assert!(end <= out.len(), "output too short");
        let reproduced = &out[start..end];
        for i in 0..click_len {
            assert!(
                (reproduced[i] - click[i]).abs() < 1e-5,
                "sample {} differs: reproduced={} click={}",
                i,
                reproduced[i],
                click[i]
            );
        }
    }

    #[test]
    fn test_beat_at_last_frame_of_block_continues_next_block() {
        let channels = 1;
        let click = synthetic_click(300, channels);
        let click_len = click.len();
        let state = state_with_click(48000, channels, click.clone());
        let block_frames = 240;
        set_single_beat(&state, (block_frames - 1) as u64, true);

        let combined = run_blocks(&state, channels, block_frames, 3, 0, 1.0);

        let start = (block_frames - 1) * channels;
        let end = start + click_len;
        assert!(
            end <= combined.len(),
            "combined output too short: need {} got {}",
            end,
            combined.len()
        );
        let reproduced = &combined[start..end];
        for i in 0..click_len {
            assert!(
                (reproduced[i] - click[i]).abs() < 1e-5,
                "tail sample {} differs: reproduced={} click={}",
                i,
                reproduced[i],
                click[i]
            );
        }
    }

    #[test]
    fn test_channels_duplicated_not_interleaved() {
        let channels = 2;
        let click = synthetic_click(50, channels);
        let state = state_with_click(48000, channels, click.clone());
        set_single_beat(&state, 10, true);

        let out = run_blocks(&state, channels, 256, 4, 0, 1.0);
        let start = 10 * channels;
        let l = out[start];
        let r = out[start + 1];
        let l2 = out[start + 2];
        let r2 = out[start + 3];
        assert_eq!(l, r, "L and R for click frame 0 must match");
        assert_eq!(l2, r2, "L and R for click frame 1 must match");
        assert_ne!(l, l2, "click must not collapse L/R into a single channel");
    }

    #[test]
    fn test_never_clips_when_song_and_click_max() {
        let channels = 2;
        let click = synthetic_click(2000, channels);
        let state = state_with_click(48000, channels, click.clone());
        set_single_beat(&state, 0, true);

        let block_frames = 512;
        let mut out = vec![0.95f32; block_frames * channels];
        state.mix_block(&mut out, channels, 0, 1.0, 1.0);

        let max_abs = out.iter().fold(0.0f32, |a, &v| a.max(v.abs()));
        assert!(max_abs <= 1.0 + 1e-6, "output clipped: max={}", max_abs);
    }

    #[test]
    fn test_disabled_metronome_is_noop() {
        let channels = 1;
        let click = synthetic_click(100, channels);
        let state = state_with_click(48000, channels, click);
        state.enabled.store(false, Ordering::Relaxed);
        set_single_beat(&state, 0, true);

        let mut out = vec![0.0f32; 256];
        state.mix_block(&mut out, channels, 0, 1.0, 1.0);
        assert!(out.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_balance_zero_mutes_click() {
        let channels = 1;
        let click = synthetic_click(100, channels);
        let state = state_with_click(48000, channels, click);
        state.set_balance(0.0);
        set_single_beat(&state, 0, true);

        let out = run_blocks(&state, channels, 256, 1, 0, 0.0);
        assert!(out.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_resample_linear_doubles_length_for_double_rate() {
        let input = vec![0.0, 1.0, 0.5, 0.0];
        let out = resample_linear(&input, 48000, 96000);
        assert_eq!(out.len(), 8);
    }

    #[test]
    fn test_parse_wav_roundtrip_headers() {
        let info = parse_wav(BEAT_WAV).expect("beat.wav must parse");
        assert_eq!(info.sample_rate, 48000);
        assert_eq!(info.bits, 16);
        assert_eq!(info.channels, 1);
        assert!(!info.data.is_empty());
    }

    #[test]
    fn test_grid_compute_single_tp() {
        let tps = vec![TimingPoint::new(0.0, 120.0)];
        let grid = MetronomeGrid::compute(&tps, 48000, 48000 * 5);
        assert!(!grid.beats.is_empty());
        assert!(grid.beats[0].is_downbeat);
        for b in &grid.beats {
            assert!(b.sample_index < 48000 * 5);
        }
    }
}
