use std::fs::File;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub struct DecodedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_ms: f64,
}

#[derive(Clone)]
pub struct AudioMetadata {
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_ms: f64,
    pub total_frames: u64,
}

pub fn probe_file(path: &str) -> Result<AudioMetadata, String> {
    let file_path = Path::new(path);
    let src = File::open(file_path).map_err(|e| format!("No se pudo abrir el archivo: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());
    let hint = build_hint(file_path);

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| format!("Formato no soportado: {}", e))?;

    let format = probed.format;
    let track = format
        .default_track()
        .ok_or_else(|| "No se encontró una pista de audio".to_string())?;

    let codec_params = &track.codec_params;
    let sample_rate = codec_params.sample_rate.unwrap_or(44100);
    let channels = codec_params
        .channels
        .map(|ch| ch.count() as u16)
        .unwrap_or(2);

    let total_frames = codec_params.n_frames.unwrap_or(0);
    let duration_ms = if total_frames > 0 {
        let rate = sample_rate as f64;
        (total_frames as f64 / rate) * 1000.0
    } else {
        0.0
    };

    Ok(AudioMetadata {
        sample_rate,
        channels,
        duration_ms,
        total_frames,
    })
}

pub fn decode_file(path: &str) -> Result<DecodedAudio, String> {
    let file_path = Path::new(path);
    let src = File::open(file_path).map_err(|e| format!("No se pudo abrir el archivo: {}", e))?;

    let mss = MediaSourceStream::new(Box::new(src), Default::default());
    let hint = build_hint(file_path);

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| format!("Formato no soportado: {}", e))?;

    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or_else(|| "No se encontró una pista de audio".to_string())?;

    let codec_params = track.codec_params.clone();
    let sample_rate = codec_params.sample_rate.unwrap_or(44100);
    let channels = codec_params
        .channels
        .map(|ch| ch.count() as u16)
        .unwrap_or(2);

    let dec_opts: DecoderOptions = Default::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &dec_opts)
        .map_err(|e| format!("Codec no soportado: {}", e))?;

    let track_id = track.id;
    let duration = calc_track_duration(&track);

    let all_samples = decode_track(&mut format, &mut decoder, track_id)?;

    if all_samples.is_empty() {
        return Err("El archivo de audio está vacío".to_string());
    }

    let calculated_duration = if duration > 0.0 {
        duration
    } else {
        (all_samples.len() as f64 / (sample_rate as f64 * channels as f64)) * 1000.0
    };

    Ok(DecodedAudio {
        samples: all_samples,
        sample_rate,
        channels,
        duration_ms: calculated_duration,
    })
}

pub struct StreamingDecoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    channels: usize,
    done: bool,
}

unsafe impl Send for StreamingDecoder {}

impl StreamingDecoder {
    pub fn open(path: &str) -> Result<Self, String> {
        let file_path = Path::new(path);
        let src =
            File::open(file_path).map_err(|e| format!("No se pudo abrir el archivo: {}", e))?;
        let mss = MediaSourceStream::new(Box::new(src), Default::default());
        let hint = build_hint(file_path);

        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(|e| format!("Formato no soportado: {}", e))?;

        let mut format = probed.format;

        let (track_id, codec_params, channels) = {
            let track = format
                .default_track()
                .ok_or_else(|| "No se encontró una pista de audio".to_string())?;
            let cp = track.codec_params.clone();
            let ch = cp.channels.map(|c| c.count() as usize).unwrap_or(2);
            (track.id, cp, ch)
        };

        let dec_opts: DecoderOptions = Default::default();
        let decoder = symphonia::default::get_codecs()
            .make(&codec_params, &dec_opts)
            .map_err(|e| format!("Codec no soportado: {}", e))?;

        Ok(StreamingDecoder {
            format,
            decoder,
            track_id,
            channels,
            done: false,
        })
    }

    pub fn decode_chunk(&mut self, max_frames: usize) -> Result<Vec<f32>, String> {
        if self.done {
            return Ok(Vec::new());
        }

        let mut samples = Vec::new();

        while samples.len() / (self.channels * 4) < max_frames {
            let packet = match self.next_packet() {
                Some(p) => p,
                None => break,
            };
            self.decode_packet(&packet, &mut samples);
        }

        Ok(samples)
    }

    fn next_packet(&mut self) -> Option<symphonia::core::formats::Packet> {
        match self.format.next_packet() {
            Ok(p) => Some(p),
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                self.done = true;
                None
            }
            Err(symphonia::core::errors::Error::DecodeError(ref e))
                if e.contains("end of stream") =>
            {
                self.done = true;
                None
            }
            Err(_) => {
                self.done = true;
                None
            }
        }
    }

    fn decode_packet(&mut self, packet: &symphonia::core::formats::Packet, samples: &mut Vec<f32>) {
        if packet.track_id() != self.track_id {
            return;
        }

        if let Ok(decoded) = self.decoder.decode(packet) {
            let spec = *decoded.spec();
            let num_frames = decoded.frames() as u64;
            let mut buf = SampleBuffer::<f32>::new(num_frames, spec);
            buf.copy_interleaved_ref(decoded);
            samples.extend_from_slice(buf.samples());
        }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }
}

fn build_hint(file_path: &Path) -> Hint {
    let mut hint = Hint::new();
    if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    hint
}

fn calc_track_duration(track: &symphonia::core::formats::Track) -> f64 {
    track
        .codec_params
        .time_base
        .zip(track.codec_params.n_frames)
        .map(|(tb, n_frames)| {
            let time = tb.calc_time(n_frames);
            time.seconds as f64 * 1000.0 + time.frac * 1000.0
        })
        .unwrap_or(0.0)
}

fn decode_track(
    format: &mut Box<dyn FormatReader>,
    decoder: &mut Box<dyn Decoder>,
    track_id: u32,
) -> Result<Vec<f32>, String> {
    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(symphonia::core::errors::Error::DecodeError(ref e))
                if e.contains("end of stream") =>
            {
                break;
            }
            Err(err) => {
                return Err(format!("Error de decodificación: {}", err));
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                let num_frames = decoded.frames() as u64;
                let mut buf = SampleBuffer::<f32>::new(num_frames, spec);
                buf.copy_interleaved_ref(decoded);
                all_samples.extend_from_slice(buf.samples());
            }
            Err(symphonia::core::errors::Error::IoError(_)) => continue,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(err) => {
                return Err(format!("Error de decodificación: {}", err));
            }
        }
    }

    Ok(all_samples)
}

pub fn extract_peaks(samples: &[f32], channels: u16, num_points: usize) -> Vec<f32> {
    if samples.is_empty() || num_points == 0 {
        return vec![0.0; num_points];
    }

    let channels = channels as usize;
    let mono_samples: Vec<f32> = if channels == 1 {
        samples.to_vec()
    } else {
        samples
            .chunks(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    };

    let total = mono_samples.len();
    let segment_size = (total / num_points).max(1);

    (0..num_points)
        .map(|i| {
            let start = i * segment_size;
            let end = (start + segment_size).min(total);
            if start >= total {
                return 0.0;
            }
            mono_samples[start..end]
                .iter()
                .map(|s| s.abs())
                .fold(0.0f32, f32::max)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_peaks_empty_input() {
        let peaks = extract_peaks(&[], 2, 100);
        assert_eq!(peaks.len(), 100);
        assert!(peaks.iter().all(|&p| p == 0.0));
    }

    #[test]
    fn test_extract_peaks_zero_points() {
        let samples = vec![0.5; 1000];
        let peaks = extract_peaks(&samples, 2, 0);
        assert!(peaks.is_empty());
    }

    #[test]
    fn test_extract_peaks_correct_length() {
        let samples = vec![0.5; 44100];
        let peaks = extract_peaks(&samples, 1, 2000);
        assert_eq!(peaks.len(), 2000);
    }

    #[test]
    fn test_extract_peaks_normalized_range() {
        let mut samples = vec![0.0f32; 44100];
        samples[100] = 0.8;
        samples[200] = -0.6;
        samples[300] = 0.3;
        let peaks = extract_peaks(&samples, 1, 10);
        assert!(peaks.iter().all(|&p| (0.0..=1.0).contains(&p)));
    }

    #[test]
    fn test_extract_peaks_max_value() {
        let mut samples = vec![0.0f32; 44100];
        samples[500] = 0.9;
        let peaks = extract_peaks(&samples, 1, 100);
        assert!(peaks.iter().any(|&p| (p - 0.9).abs() < 0.01));
    }

    #[test]
    fn test_extract_peaks_stereo_to_mono() {
        let stereo_samples: Vec<f32> = (0..2000)
            .map(|i| if i % 2 == 0 { 0.8 } else { 0.4 })
            .collect();
        let peaks = extract_peaks(&stereo_samples, 2, 10);
        assert_eq!(peaks.len(), 10);
        for peak in &peaks {
            assert!((*peak - 0.6).abs() < 0.01);
        }
    }
}
