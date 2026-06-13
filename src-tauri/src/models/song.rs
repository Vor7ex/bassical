use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub audio_path: String,
    pub audio_missing: bool,
    pub has_tab: bool,
    pub has_calibration: bool,
    pub preferred_speed: f64,
    pub last_position_ms: f64,
    pub created_at: String,
    pub updated_at: String,
}

impl Song {
    pub fn new(title: String, artist: Option<String>, audio_path: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            artist,
            audio_path,
            audio_missing: false,
            has_tab: false,
            has_calibration: false,
            preferred_speed: 1.0,
            last_position_ms: 0.0,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub songs: Vec<Song>,
}

impl Library {
    pub fn new() -> Self {
        Self { songs: Vec::new() }
    }
}
