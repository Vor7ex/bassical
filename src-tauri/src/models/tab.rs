use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimingPoint {
    pub offset_ms: f64,
    pub bpm: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BassicalTab {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub audio_path: String,
    pub timing_points: Vec<TimingPoint>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PracticeData {
    pub preferred_speed: f64,
    pub last_position_ms: f64,
}
