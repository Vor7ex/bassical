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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_song_id_new() {
        let id = SongId::new("test-id-123".to_string());
        assert_eq!(id.as_str(), "test-id-123");
        assert_eq!(format!("{}", id), "test-id-123");
    }

    #[test]
    fn test_song_id_from_string() {
        let id: SongId = "from-string".to_string().into();
        assert_eq!(id.as_str(), "from-string");
    }

    #[test]
    fn test_audio_path_new() {
        let path = AudioPath::new("/path/to/audio.mp3".to_string());
        assert_eq!(path.as_str(), "/path/to/audio.mp3");
        assert_eq!(format!("{}", path), "/path/to/audio.mp3");
    }

    #[test]
    fn test_audio_path_exists_returns_false_for_missing() {
        let path = AudioPath::new("/nonexistent/path/audio.mp3".to_string());
        assert!(!path.exists());
    }

    #[test]
    fn test_song_title_new() {
        let title = SongTitle::new("My Song".to_string());
        assert_eq!(title.as_str(), "My Song");
    }

    #[test]
    fn test_artist_name_new() {
        let artist = ArtistName::new("The Band".to_string());
        assert_eq!(artist.as_str(), "The Band");
    }

    #[test]
    fn test_song_new_sets_defaults() {
        let title = SongTitle::new("Test Song".to_string());
        let artist = Some(ArtistName::new("Test Artist".to_string()));
        let audio_path = AudioPath::new("/path/to/audio.mp3".to_string());

        let song = Song::new(title, artist, audio_path);

        assert!(!song.id.as_str().is_empty());
        assert_eq!(song.title.as_str(), "Test Song");
        assert_eq!(song.artist.unwrap().as_str(), "Test Artist");
        assert_eq!(song.audio_path.as_str(), "/path/to/audio.mp3");
        assert!(!song.audio_missing);
        assert!(!song.has_tab);
        assert!(!song.has_calibration);
        assert_eq!(song.preferred_speed, 1.0);
        assert_eq!(song.last_position_ms, 0.0);
        assert!(!song.created_at.is_empty());
        assert!(!song.updated_at.is_empty());
        assert_eq!(song.created_at, song.updated_at);
    }

    #[test]
    fn test_song_new_with_no_artist() {
        let title = SongTitle::new("No Artist Song".to_string());
        let audio_path = AudioPath::new("/path/to/audio.mp3".to_string());

        let song = Song::new(title, None, audio_path);

        assert!(song.artist.is_none());
    }

    #[test]
    fn test_song_new_generates_unique_ids() {
        let title1 = SongTitle::new("Song 1".to_string());
        let title2 = SongTitle::new("Song 2".to_string());
        let audio_path = AudioPath::new("/path/to/audio.mp3".to_string());

        let song1 = Song::new(title1, None, audio_path.clone());
        let song2 = Song::new(title2, None, audio_path);

        assert_ne!(song1.id, song2.id);
    }

    #[test]
    fn test_song_clone() {
        let title = SongTitle::new("Clone Me".to_string());
        let audio_path = AudioPath::new("/path/to/audio.mp3".to_string());
        let song = Song::new(title, None, audio_path);

        let cloned = song.clone();
        assert_eq!(song.id, cloned.id);
        assert_eq!(song.title.as_str(), cloned.title.as_str());
    }

    #[test]
    fn test_song_serialization_roundtrip() {
        let title = SongTitle::new("Serialize Me".to_string());
        let artist = Some(ArtistName::new("Artist".to_string()));
        let audio_path = AudioPath::new("/path/to/audio.mp3".to_string());
        let song = Song::new(title, artist, audio_path);

        let json = serde_json::to_string(&song).unwrap();
        let deserialized: Song = serde_json::from_str(&json).unwrap();

        assert_eq!(song.id, deserialized.id);
        assert_eq!(song.title.as_str(), deserialized.title.as_str());
        assert_eq!(
            song.artist.unwrap().as_str(),
            deserialized.artist.unwrap().as_str()
        );
    }

    #[test]
    fn test_library_new_is_empty() {
        let library = Library::new();
        assert!(library.songs.is_empty());
    }

    #[test]
    fn test_library_serialization_roundtrip() {
        let mut library = Library::new();
        let title = SongTitle::new("Library Song".to_string());
        let audio_path = AudioPath::new("/path/to/audio.mp3".to_string());
        library.songs.push(Song::new(title, None, audio_path));

        let json = serde_json::to_string(&library).unwrap();
        let deserialized: Library = serde_json::from_str(&json).unwrap();

        assert_eq!(library.songs.len(), deserialized.songs.len());
        assert_eq!(
            library.songs[0].title.as_str(),
            deserialized.songs[0].title.as_str()
        );
    }
}
