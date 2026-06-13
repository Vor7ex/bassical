use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SongId(String);

impl SongId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SongId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SongId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AudioPath(String);

impl AudioPath {
    pub fn new(path: String) -> Self {
        Self(path)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn exists(&self) -> bool {
        PathBuf::from(&self.0).exists()
    }
}

impl fmt::Display for AudioPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AudioPath {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SongTitle(String);

impl SongTitle {
    pub fn new(title: String) -> Self {
        Self(title)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SongTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SongTitle {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArtistName(String);

impl ArtistName {
    pub fn new(name: String) -> Self {
        Self(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArtistName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ArtistName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: SongId,
    pub title: SongTitle,
    pub artist: Option<ArtistName>,
    pub audio_path: AudioPath,
    pub audio_missing: bool,
    pub has_tab: bool,
    pub has_calibration: bool,
    pub preferred_speed: f64,
    pub last_position_ms: f64,
    pub created_at: String,
    pub updated_at: String,
}

impl Song {
    pub fn new(title: SongTitle, artist: Option<ArtistName>, audio_path: AudioPath) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: SongId::new(uuid::Uuid::new_v4().to_string()),
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
