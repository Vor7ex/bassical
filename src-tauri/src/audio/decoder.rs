use std::fs::File;
use std::path::Path;
use symphonia::core::audio::{Audio, GenericAudioBufferRef};
use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::packet::Packet;
use symphonia::core::units::Time;

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

    let format = symphonia::default::get_probe()
        .probe(&hint, mss, fmt_opts, meta_opts)
        .map_err(|e| format!("Formato no soportado: {}", e))?;

    let track = format
        .default_track(TrackType::Audio)
        .ok_or_else(|| "No se encontró una pista de audio".to_string())?;

    let codec_params = track
        .codec_params
        .as_ref()
        .and_then(|cp| cp.audio())
        .ok_or_else(|| "No se encontraron parámetros de audio".to_string())?;

    let sample_rate = codec_params.sample_rate.unwrap_or(44100);
    let channels = codec_params
        .channels
        .as_ref()
        .map(|ch| ch.count() as u16)
        .unwrap_or(2);

    let (duration_ms, total_frames) =
        if let (Some(dur), Some(tb)) = (track.duration, track.time_base) {
            let frames = dur.get();
            let seconds = frames as f64 * tb.numer.get() as f64 / tb.denom.get() as f64;
            (seconds * 1000.0, frames)
        } else {
            (0.0, 0)
        };

    Ok(AudioMetadata {
        sample_rate,
        channels,
        duration_ms,
        total_frames,
    })
}

pub struct StreamingDecoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
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

        let format = symphonia::default::get_probe()
            .probe(&hint, mss, fmt_opts, meta_opts)
            .map_err(|e| format!("Formato no soportado: {}", e))?;

        let (track_id, codec_params, channels) = {
            let track = format
                .default_track(TrackType::Audio)
                .ok_or_else(|| "No se encontró una pista de audio".to_string())?;
            let cp = track
                .codec_params
                .clone()
                .ok_or_else(|| "No se encontraron parámetros de codec".to_string())?;
            let ch = cp
                .audio()
                .and_then(|a| a.channels.clone())
                .map(|c| c.count())
                .unwrap_or(2);
            (track.id, cp, ch)
        };

        let dec_opts: AudioDecoderOptions = Default::default();
        let audio_params = codec_params
            .audio()
            .ok_or_else(|| "No se encontraron parámetros de audio".to_string())?;
        let decoder = symphonia::default::get_codecs()
            .make_audio_decoder(audio_params, &dec_opts)
            .map_err(|e| format!("Codec no soportado: {}", e))?;

        Ok(StreamingDecoder {
            format,
            decoder,
            track_id,
            channels,
            done: false,
        })
    }

    pub fn seek_to_ms(&mut self, ms: f64) -> Result<(), String> {
        let time =
            Time::try_from_secs_f64(ms / 1000.0).ok_or_else(|| "Tiempo inválido".to_string())?;

        self.format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time,
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| format!("Seek error: {}", e))?;

        self.done = false;
        Ok(())
    }

    pub fn decode_chunk(&mut self, max_frames: usize) -> Result<Vec<f32>, String> {
        if self.done {
            return Ok(Vec::new());
        }

        let mut samples = Vec::new();

        while samples.len() / self.channels < max_frames {
            let packet = match self.next_packet() {
                Some(p) => p,
                None => break,
            };
            self.decode_packet(&packet, &mut samples);
        }

        Ok(samples)
    }

    fn next_packet(&mut self) -> Option<Packet> {
        match self.format.next_packet() {
            Ok(Some(p)) => Some(p),
            Ok(None) => {
                self.done = true;
                None
            }
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                self.done = true;
                None
            }
            Err(symphonia::core::errors::Error::DecodeError(e)) if e.contains("end of stream") => {
                self.done = true;
                None
            }
            Err(_) => {
                self.done = true;
                None
            }
        }
    }

    fn decode_packet(&mut self, packet: &Packet, samples: &mut Vec<f32>) {
        if packet.track_id != self.track_id {
            return;
        }

        if let Ok(decoded) = self.decoder.decode(packet) {
            if let GenericAudioBufferRef::F32(buf) = decoded {
                copy_f32_samples(&buf, samples);
            }
        }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    pub fn channels(&self) -> usize {
        self.channels
    }
}

fn copy_f32_samples(buf: &symphonia::core::audio::AudioBuffer<f32>, samples: &mut Vec<f32>) {
    let planes: Vec<&[f32]> = buf.iter_planes().collect();
    let num_frames = buf.frames();
    let num_channels = planes.len();
    for frame_idx in 0..num_frames {
        for ch in 0..num_channels {
            samples.push(planes[ch][frame_idx]);
        }
    }
}

fn build_hint(file_path: &Path) -> symphonia::core::formats::probe::Hint {
    let mut hint = symphonia::core::formats::probe::Hint::new();
    if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    hint
}
