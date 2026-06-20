#![allow(dead_code)]

use crate::models::song::{ArtistName, AudioPath, SongId, SongTitle};
use serde::{Deserialize, Serialize};

/// Time signature de un compás (ej. 3/4, 4/4, 6/8).
/// El denominador es la unidad de nota que cuenta el BPM
/// (4 => negras, 8 => corcheas).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

impl TimeSignature {
    /// Compás por defecto: 4/4.
    pub const DEFAULT: TimeSignature = TimeSignature {
        numerator: 4,
        denominator: 4,
    };

    pub fn new(numerator: u8, denominator: u8) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.numerator >= 1
            && self.denominator_is_power_of_two()
            && self.denominator >= 1
            && self.denominator <= 64
    }

    fn denominator_is_power_of_two(&self) -> bool {
        self.denominator != 0 && (self.denominator & (self.denominator - 1)) == 0
    }
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Punto de calibración de tempo. Define el BPM que rige a partir de
/// `offset_ms` (milisegundos desde el inicio del audio). El `time_signature`
/// es opcional; cuando es `None` se aplica el default 4/4. Se omite del JSON
/// cuando coincide con el default (skip_serializing_if).
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimingPoint {
    pub offset_ms: f64,
    pub bpm: f64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_time_signature_optional"
    )]
    pub time_signature: Option<TimeSignature>,
}

impl TimingPoint {
    pub fn new(offset_ms: f64, bpm: f64) -> Self {
        Self {
            offset_ms,
            bpm,
            time_signature: None,
        }
    }

    pub fn with_time_signature(offset_ms: f64, bpm: f64, time_signature: TimeSignature) -> Self {
        Self {
            offset_ms,
            bpm,
            time_signature: Some(time_signature),
        }
    }

    /// Devuelve el compás efectivo (el propio o el default 4/4).
    pub fn effective_time_signature(&self) -> TimeSignature {
        self.time_signature.clone().unwrap_or_default()
    }
}

/// Serializa `Option<TimeSignature>` omitiendo el default 4/4,
/// y al deserializar trata ausencia como `None`.
mod serde_time_signature_optional {
    use super::TimeSignature;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<TimeSignature>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(ts) => {
                if *ts == TimeSignature::DEFAULT {
                    serializer.serialize_none()
                } else {
                    ts.serialize(serializer)
                }
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<TimeSignature>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<TimeSignature>::deserialize(deserializer)?;
        Ok(opt.filter(|ts| *ts != TimeSignature::DEFAULT))
    }
}

/// Contenedor del archivo `songs/<uuid>.bassical.json`.
/// `tab` y `practice` son opcionales y se omiten del JSON cuando no existen,
/// para no acoplar Sprint 4 (calibración) con el modelo de tablatura (Sprint 5).
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BassicalTab {
    pub schema_version: u32,
    pub id: SongId,
    pub title: SongTitle,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<ArtistName>,
    pub audio_path: AudioPath,
    pub timing_points: Vec<TimingPoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab: Option<TabData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub practice: Option<PracticeData>,
}

/// Metadata de la canción embebida en el archivo `.bassical.json`.
/// Se extrae de `Song` al guardar calibración para que el archivo sea
/// autocontenido (identificable sin la biblioteca).
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalibrationMeta {
    pub id: SongId,
    pub title: SongTitle,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<ArtistName>,
    pub audio_path: AudioPath,
}

impl BassicalTab {
    pub const SCHEMA_VERSION: u32 = 1;

    /// Crea un contenedor solo con calibración (sin tab/practice).
    pub fn for_calibration(meta: CalibrationMeta, timing_points: Vec<TimingPoint>) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            id: meta.id,
            title: meta.title,
            artist: meta.artist,
            audio_path: meta.audio_path,
            timing_points,
            tab: None,
            practice: None,
        }
    }
}

/// Placeholder del modelo de tablatura (Sprint 5). Se declara para que el
/// esquema versionado esté completo, pero en Sprint 4 siempre es `None`
/// (omitido del JSON). Sprint 5 lo rellenará.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TabData {
    pub strings: u8,
    pub tuning: Vec<String>,
    pub measures: Vec<Measure>,
}

/// Placeholder de compás (Sprint 5).
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Measure {
    pub time_signature: TimeSignature,
    pub beats: Vec<Beat>,
}

/// Placeholder de beat (Sprint 5).
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Beat {
    pub duration: String,
    pub notes: Vec<Note>,
}

/// Placeholder de nota (Sprint 5).
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub string: u8,
    pub fret: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technique: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PracticeData {
    pub preferred_speed: f64,
    pub last_position_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tp(tp: &TimingPoint, offset_ms: f64, bpm: f64, ts: Option<TimeSignature>) {
        assert_eq!(tp.offset_ms, offset_ms);
        assert_eq!(tp.bpm, bpm);
        assert_eq!(tp.time_signature, ts);
    }

    fn assert_tab_identity(
        tab: &BassicalTab,
        id: &SongId,
        title: &SongTitle,
        artist: Option<&ArtistName>,
    ) {
        assert_eq!(&tab.id, id);
        assert_eq!(&tab.title, title);
        assert_eq!(tab.artist.as_ref(), artist);
    }

    fn assert_tab_optional(tab: &BassicalTab, has_tab: bool, has_practice: bool) {
        assert_eq!(tab.tab.is_some(), has_tab);
        assert_eq!(tab.practice.is_some(), has_practice);
    }

    fn assert_json_omits(json: &str, keys: &[&str]) {
        for key in keys {
            assert!(
                !json.contains(key),
                "expected {} omitted, got: {}",
                key,
                json
            );
        }
    }

    fn assert_json_contains(json: &str, keys: &[&str]) {
        for key in keys {
            assert!(json.contains(key), "expected {} in: {}", key, json);
        }
    }

    #[test]
    fn test_time_signature_default_is_4_4() {
        let ts = TimeSignature::default();
        assert_eq!(ts.numerator, 4);
        assert_eq!(ts.denominator, 4);
    }

    #[test]
    fn test_time_signature_is_valid_common() {
        for &(n, d) in &[(3, 4), (4, 4), (2, 4), (6, 8), (12, 8), (5, 4), (7, 8)] {
            assert!(
                TimeSignature::new(n, d).is_valid(),
                "{}/{} should be valid",
                n,
                d
            );
        }
    }

    #[test]
    fn test_time_signature_invalid_denominator_non_power_of_two() {
        for d in [3, 5, 6, 7] {
            assert!(
                !TimeSignature::new(4, d).is_valid(),
                "4/{} should be invalid",
                d
            );
        }
    }

    #[test]
    fn test_time_signature_invalid_zero() {
        assert!(!TimeSignature::new(0, 4).is_valid());
        assert!(!TimeSignature::new(4, 0).is_valid());
    }

    #[test]
    fn test_timing_point_new_no_time_signature() {
        let tp = TimingPoint::new(1240.0, 132.0);
        assert_tp(&tp, 1240.0, 132.0, None);
        assert_eq!(tp.effective_time_signature(), TimeSignature::DEFAULT);
    }

    #[test]
    fn test_timing_point_with_time_signature_uses_it() {
        let tp = TimingPoint::with_time_signature(0.0, 100.0, TimeSignature::new(3, 4));
        assert_eq!(tp.effective_time_signature(), TimeSignature::new(3, 4));
    }

    #[test]
    fn test_timing_point_serialization_omits_default_time_signature() {
        let tp = TimingPoint::new(1000.0, 120.0); // 4/4 default
        let json = serde_json::to_string(&tp).unwrap();
        assert!(
            !json.contains("timeSignature"),
            "default 4/4 must be omitted, got: {}",
            json
        );
    }

    #[test]
    fn test_timing_point_serialization_keeps_non_default_time_signature() {
        let tp = TimingPoint::with_time_signature(1000.0, 120.0, TimeSignature::new(6, 8));
        let json = serde_json::to_string(&tp).unwrap();
        assert!(json.contains("timeSignature"));
        assert!(json.contains("\"numerator\":6"));
        assert!(json.contains("\"denominator\":8"));
    }

    #[test]
    fn test_timing_point_serialization_roundtrip_with_time_signature() {
        let tp = TimingPoint::with_time_signature(2500.0, 90.5, TimeSignature::new(3, 4));
        let json = serde_json::to_string(&tp).unwrap();
        let back: TimingPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(back.offset_ms, 2500.0);
        assert_eq!(back.bpm, 90.5);
        assert_eq!(back.time_signature, Some(TimeSignature::new(3, 4)));
    }

    #[test]
    fn test_timing_point_serialization_roundtrip_without_time_signature() {
        let tp = TimingPoint::new(500.0, 140.0);
        let json = serde_json::to_string(&tp).unwrap();
        let back: TimingPoint = serde_json::from_str(&json).unwrap();
        assert!(back.time_signature.is_none());
        assert_eq!(back.effective_time_signature(), TimeSignature::DEFAULT);
    }

    #[test]
    fn test_timing_point_deserialization_explicit_default_becomes_none() {
        // Si el JSON trae explícitamente 4/4, se normaliza a None.
        let json = r#"{"offsetMs":0,"bpm":100,"timeSignature":{"numerator":4,"denominator":4}}"#;
        let tp: TimingPoint = serde_json::from_str(json).unwrap();
        assert!(tp.time_signature.is_none());
    }

    #[test]
    fn test_bassical_tab_for_calibration_omits_tab_and_practice() {
        let meta = CalibrationMeta {
            id: SongId::new("song-1".to_string()),
            title: SongTitle::new("Title".to_string()),
            artist: Some(ArtistName::new("Artist".to_string())),
            audio_path: AudioPath::new("/audio.mp3".to_string()),
        };
        let tab = BassicalTab::for_calibration(meta, vec![TimingPoint::new(0.0, 120.0)]);
        let json = serde_json::to_string_pretty(&tab).unwrap();
        assert_json_omits(&json, &["\"tab\"", "\"practice\""]);
        assert_json_contains(&json, &["schemaVersion", "timingPoints"]);
    }

    #[test]
    fn test_bassical_tab_for_calibration_roundtrip() {
        let meta = CalibrationMeta {
            id: SongId::new("uuid-1".to_string()),
            title: SongTitle::new("My Generation".to_string()),
            artist: Some(ArtistName::new("The Who".to_string())),
            audio_path: AudioPath::new("C:\\audio.mp3".to_string()),
        };
        let original = BassicalTab::for_calibration(
            meta,
            vec![
                TimingPoint::new(1240.0, 132.0),
                TimingPoint::with_time_signature(48300.0, 134.5, TimeSignature::new(3, 4)),
            ],
        );
        let json = serde_json::to_string(&original).unwrap();
        let back: BassicalTab = serde_json::from_str(&json).unwrap();
        assert_eq!(back.schema_version, 1);
        assert_eq!(back.timing_points.len(), 2);
        assert_tab_identity(
            &back,
            &SongId::new("uuid-1".to_string()),
            &SongTitle::new("My Generation".to_string()),
            Some(&ArtistName::new("The Who".to_string())),
        );
        assert_tp(&back.timing_points[0], 1240.0, 132.0, None);
        assert_tp(
            &back.timing_points[1],
            48300.0,
            134.5,
            Some(TimeSignature::new(3, 4)),
        );
        assert_tab_optional(&back, false, false);
    }

    #[test]
    fn test_bassical_tab_deserialization_without_optional_fields() {
        let json = r#"{
            "schemaVersion": 1,
            "id": "x",
            "title": "T",
            "audioPath": "/a.mp3",
            "timingPoints": [{"offsetMs": 0, "bpm": 100}]
        }"#;
        let tab: BassicalTab = serde_json::from_str(json).unwrap();
        assert_tab_identity(
            &tab,
            &SongId::new("x".to_string()),
            &SongTitle::new("T".to_string()),
            None,
        );
        assert_tab_optional(&tab, false, false);
        assert_eq!(tab.timing_points.len(), 1);
    }
}
