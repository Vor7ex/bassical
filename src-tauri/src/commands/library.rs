use crate::models::song::{ArtistName, AudioPath, Library, Song, SongId, SongTitle};
use crate::persistence::storage;
use std::fs::File;
use std::path::Path;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTag};

const LIBRARY_FILE: &str = "library.json";

#[derive(serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SongMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<String>,
    pub genre: Option<String>,
}

fn apply_standard_tag(meta: &mut SongMetadata, std_tag: &StandardTag) {
    match std_tag {
        StandardTag::TrackTitle(s) => {
            if meta.title.is_none() {
                meta.title = Some(s.to_string());
            }
        }
        StandardTag::Artist(s) => {
            if meta.artist.is_none() {
                meta.artist = Some(s.to_string());
            }
        }
        StandardTag::Album(s) => {
            if meta.album.is_none() {
                meta.album = Some(s.to_string());
            }
        }
        StandardTag::ReleaseYear(y) => {
            if meta.year.is_none() {
                meta.year = Some(y.to_string());
            }
        }
        StandardTag::Genre(s) => {
            if meta.genre.is_none() {
                meta.genre = Some(s.to_string());
            }
        }
        _ => {}
    }
}

fn apply_tags_to_metadata(meta: &mut SongMetadata, tags: &[symphonia::core::meta::Tag]) {
    for tag in tags {
        if let Some(ref std_tag) = tag.std {
            apply_standard_tag(meta, std_tag);
        }
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn extract_metadata(file_path: String) -> Result<SongMetadata, String> {
    eprintln!("[extract_metadata] file_path: {}", file_path);
    let path = Path::new(&file_path);
    let src = File::open(path).map_err(|e| format!("No se pudo abrir el archivo: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
        eprintln!("[extract_metadata] extension: {}", ext);
    }

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let mut format = symphonia::default::get_probe()
        .probe(&hint, mss, fmt_opts, meta_opts)
        .map_err(|e| format!("Formato no soportado: {}", e))?;

    eprintln!("[extract_metadata] format probed successfully");

    let mut song_meta = SongMetadata::default();

    match format.metadata().current() {
        Some(rev) => {
            eprintln!(
                "[extract_metadata] metadata revision found, tags count: {}",
                rev.media.tags.len()
            );
            apply_tags_to_metadata(&mut song_meta, &rev.media.tags);
        }
        None => {
            eprintln!("[extract_metadata] NO metadata revision found");
        }
    }

    eprintln!("[extract_metadata] result: {:?}", song_meta);
    Ok(song_meta)
}

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SongUpdate {
    pub title: Option<SongTitle>,
    pub artist: Option<ArtistName>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u16>,
    pub tuning: Option<String>,
    pub bpm: Option<f64>,
    pub difficulty: Option<u8>,
    pub tags: Option<Vec<String>>,
    pub audio_path: Option<AudioPath>,
}

fn apply_opt_str(target: &mut Option<String>, value: Option<String>) {
    if let Some(v) = value {
        *target = if v.is_empty() { None } else { Some(v) };
    }
}

fn apply_updates(song: &mut Song, update: SongUpdate) -> Result<(), String> {
    if let Some(t) = update.title {
        song.title = t;
    }
    if let Some(a) = update.artist {
        song.artist = Some(a);
    }
    apply_opt_str(&mut song.album, update.album);
    apply_opt_str(&mut song.genre, update.genre);
    apply_opt_str(&mut song.tuning, update.tuning);
    song.year = update.year;
    song.bpm = update.bpm;
    song.difficulty = update.difficulty;
    if let Some(t) = update.tags {
        song.tags = t;
    }
    if let Some(path) = update.audio_path {
        if !path.exists() {
            return Err("El archivo de audio no existe".to_string());
        }
        song.audio_path = path;
        song.audio_missing = false;
    }
    song.updated_at = chrono::Utc::now().to_rfc3339();
    Ok(())
}

fn load_library() -> Result<Library, String> {
    match storage::read_json::<Library>(LIBRARY_FILE) {
        Ok(lib) => Ok(lib),
        Err(_) => Ok(Library::new()),
    }
}

fn save_library(library: &Library) -> Result<(), String> {
    storage::write_json(LIBRARY_FILE, library)
}

fn find_song_by_id<'a>(library: &'a Library, id: &SongId) -> Option<&'a Song> {
    library.songs.iter().find(|s| &s.id == id)
}

fn find_song_by_id_mut<'a>(library: &'a mut Library, id: &SongId) -> Option<&'a mut Song> {
    library.songs.iter_mut().find(|s| &s.id == id)
}

#[tauri::command]
pub fn init_app() -> Result<String, String> {
    storage::ensure_app_data_dir()?;
    if storage::read_json::<Library>(LIBRARY_FILE).is_err() {
        save_library(&Library::new())?;
    }
    Ok("App initialized".to_string())
}

#[tauri::command]
pub fn get_library() -> Result<Library, String> {
    load_library()
}

#[tauri::command]
pub fn add_song(
    title: SongTitle,
    artist: Option<ArtistName>,
    audio_path: AudioPath,
    album: Option<String>,
    genre: Option<String>,
    year: Option<u16>,
) -> Result<Song, String> {
    if !audio_path.exists() {
        return Err("El archivo de audio no existe".to_string());
    }

    let mut library = load_library()?;
    let mut song = Song::new(title, artist, audio_path);
    song.album = album;
    song.genre = genre;
    song.year = year;
    library.songs.push(song.clone());
    save_library(&library)?;
    Ok(song)
}

#[tauri::command]
pub fn update_song(id: SongId, update: SongUpdate) -> Result<Song, String> {
    let mut library = load_library()?;
    let song = find_song_by_id_mut(&mut library, &id)
        .ok_or_else(|| "Canción no encontrada".to_string())?;

    apply_updates(song, update)?;

    let updated_song = song.clone();
    save_library(&library)?;
    Ok(updated_song)
}

#[tauri::command]
pub fn delete_song(id: SongId) -> Result<(), String> {
    let mut library = load_library()?;
    library.songs.retain(|s| s.id != id);
    save_library(&library)?;
    Ok(())
}

#[tauri::command]
pub fn check_audio_exists(id: SongId) -> Result<bool, String> {
    let library = load_library()?;
    let song = find_song_by_id(&library, &id).ok_or_else(|| "Canción no encontrada".to_string())?;
    Ok(song.audio_path.exists())
}

#[tauri::command]
pub fn get_library_with_status() -> Result<Library, String> {
    let mut library = load_library()?;
    for song in &mut library.songs {
        song.audio_missing = !song.audio_path.exists();
    }
    save_library(&library)?;
    Ok(library)
}

#[tauri::command]
pub fn reassign_audio_path(id: SongId, new_path: AudioPath) -> Result<Song, String> {
    if !new_path.exists() {
        return Err("El archivo de audio no existe".to_string());
    }

    let mut library = load_library()?;
    let song = find_song_by_id_mut(&mut library, &id)
        .ok_or_else(|| "Canción no encontrada".to_string())?;

    song.audio_path = new_path;
    song.audio_missing = false;
    song.updated_at = chrono::Utc::now().to_rfc3339();

    let updated_song = song.clone();
    save_library(&library)?;
    Ok(updated_song)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::storage;
    use std::fs;
    use std::path::PathBuf;

    fn make_temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "bassical_cmd_test_{:?}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&dir).unwrap();
        storage::set_data_dir(dir.clone());
        dir
    }

    fn cleanup(dir: &PathBuf) {
        storage::clear_data_dir();
        fs::remove_dir_all(dir).ok();
    }

    fn make_song(dir: &PathBuf, name: &str) -> Song {
        let audio_file = dir.join(format!("{name}.mp3"));
        fs::write(&audio_file, "fake audio").unwrap();
        let title = SongTitle::new(name.to_string());
        let audio_path = AudioPath::new(audio_file.to_str().unwrap().to_string());
        add_song(title, None, audio_path).unwrap()
    }

    fn make_update<F: FnOnce(&mut SongUpdate)>(f: F) -> SongUpdate {
        let mut u = SongUpdate::default();
        f(&mut u);
        u
    }

    #[test]
    fn test_init_app_creates_dirs_and_library() {
        let dir = make_temp_dir();
        assert_eq!(init_app().unwrap(), "App initialized");
        assert!(dir.exists());
        assert!(dir.join("songs").exists());
        assert!(dir.join("library.json").exists());
        cleanup(&dir);
    }

    #[test]
    fn test_init_app_idempotent() {
        let dir = make_temp_dir();
        assert!(init_app().is_ok());
        assert!(init_app().is_ok());
        cleanup(&dir);
    }

    #[test]
    fn test_get_library_empty_after_init() {
        let dir = make_temp_dir();
        init_app().unwrap();
        assert!(get_library().unwrap().songs.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn test_get_library_creates_empty_if_missing() {
        let dir = make_temp_dir();
        assert!(get_library().unwrap().songs.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn test_add_song_success() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let audio_file = dir.join("test.mp3");
        fs::write(&audio_file, "data").unwrap();
        let song = add_song(
            SongTitle::new("Test".to_string()),
            Some(ArtistName::new("Artist".to_string())),
            AudioPath::new(audio_file.to_str().unwrap().to_string()),
        )
        .unwrap();
        assert_eq!(song.title.as_str(), "Test");
        assert_eq!(song.artist.unwrap().as_str(), "Artist");
        assert_eq!(get_library().unwrap().songs.len(), 1);
        cleanup(&dir);
    }

    #[test]
    fn test_add_song_no_artist() {
        let dir = make_temp_dir();
        init_app().unwrap();
        assert!(make_song(&dir, "Song").artist.is_none());
        cleanup(&dir);
    }

    #[test]
    fn test_add_song_nonexistent_audio_fails() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let result = add_song(
            SongTitle::new("Bad".to_string()),
            None,
            AudioPath::new("/nonexistent/file.mp3".to_string()),
        );
        assert!(result.is_err());
        assert!(get_library().unwrap().songs.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn test_add_multiple_songs() {
        let dir = make_temp_dir();
        init_app().unwrap();
        for i in 0..3 {
            make_song(&dir, &format!("Song {i}"));
        }
        assert_eq!(get_library().unwrap().songs.len(), 3);
        cleanup(&dir);
    }

    #[test]
    fn test_update_song_title() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Original");
        let result = update_song(
            song.id.clone(),
            make_update(|u| {
                u.title = Some(SongTitle::new("Updated".to_string()));
            }),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().title.as_str(), "Updated");
        cleanup(&dir);
    }

    #[test]
    fn test_update_song_artist() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        let result = update_song(
            song.id.clone(),
            make_update(|u| {
                u.artist = Some(ArtistName::new("New Artist".to_string()));
            }),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().artist.unwrap().as_str(), "New Artist");
        cleanup(&dir);
    }

    #[test]
    fn test_update_song_audio_path() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        let new_file = dir.join("new.mp3");
        fs::write(&new_file, "new").unwrap();
        let result = update_song(
            song.id.clone(),
            make_update(|u| {
                u.audio_path = Some(AudioPath::new(new_file.to_str().unwrap().to_string()));
            }),
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().audio_path.as_str(),
            new_file.to_str().unwrap()
        );
        cleanup(&dir);
    }

    #[test]
    fn test_update_song_nonexistent_audio_fails() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        let result = update_song(
            song.id.clone(),
            make_update(|u| {
                u.audio_path = Some(AudioPath::new("/nonexistent/file.mp3".to_string()));
            }),
        );
        assert!(result.is_err());
        cleanup(&dir);
    }

    #[test]
    fn test_update_song_not_found() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let result = update_song(
            SongId::new("no-id".to_string()),
            SongUpdate {
                title: Some(SongTitle::new("X".to_string())),
                ..SongUpdate::default()
            },
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Canción no encontrada");
        cleanup(&dir);
    }

    #[test]
    fn test_update_song_no_changes() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        let result = update_song(song.id.clone(), SongUpdate::default());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().title.as_str(), "Song");
        cleanup(&dir);
    }

    #[test]
    fn test_delete_song() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Delete Me");
        assert_eq!(get_library().unwrap().songs.len(), 1);
        assert!(delete_song(song.id).is_ok());
        assert!(get_library().unwrap().songs.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn test_delete_song_not_found_succeeds() {
        let dir = make_temp_dir();
        init_app().unwrap();
        assert!(delete_song(SongId::new("x".to_string())).is_ok());
        cleanup(&dir);
    }

    #[test]
    fn test_delete_one_of_many() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let mut ids = Vec::new();
        for i in 0..3 {
            ids.push(make_song(&dir, &format!("Song {i}")).id);
        }
        delete_song(ids[1].clone()).unwrap();
        let lib = get_library().unwrap();
        assert_eq!(lib.songs.len(), 2);
        assert!(lib.songs.iter().all(|s| s.id != ids[1]));
        cleanup(&dir);
    }

    #[test]
    fn test_check_audio_exists_true() {
        let dir = make_temp_dir();
        init_app().unwrap();
        assert!(check_audio_exists(make_song(&dir, "Song").id).unwrap());
        cleanup(&dir);
    }

    #[test]
    fn test_check_audio_exists_false_after_delete() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        fs::remove_file(dir.join("Song.mp3")).unwrap();
        assert!(!check_audio_exists(song.id).unwrap());
        cleanup(&dir);
    }

    #[test]
    fn test_check_audio_exists_not_found() {
        let dir = make_temp_dir();
        init_app().unwrap();
        assert!(check_audio_exists(SongId::new("x".to_string())).is_err());
        cleanup(&dir);
    }

    #[test]
    fn test_reassign_audio_path_success() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        let new_file = dir.join("new.mp3");
        fs::write(&new_file, "new").unwrap();
        let updated = reassign_audio_path(
            song.id,
            AudioPath::new(new_file.to_str().unwrap().to_string()),
        )
        .unwrap();
        assert_eq!(updated.audio_path.as_str(), new_file.to_str().unwrap());
        assert!(!updated.audio_missing);
        cleanup(&dir);
    }

    #[test]
    fn test_reassign_audio_path_nonexistent_fails() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        assert!(
            reassign_audio_path(song.id, AudioPath::new("/nonexistent.mp3".to_string())).is_err()
        );
        cleanup(&dir);
    }

    #[test]
    fn test_reassign_audio_path_not_found() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let f = dir.join("new.mp3");
        fs::write(&f, "data").unwrap();
        assert!(reassign_audio_path(
            SongId::new("no".to_string()),
            AudioPath::new(f.to_str().unwrap().to_string()),
        )
        .is_err());
        cleanup(&dir);
    }

    #[test]
    fn test_get_library_with_status_marks_missing() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        assert!(!get_library_with_status().unwrap().songs[0].audio_missing);
        fs::remove_file(dir.join("Song.mp3")).unwrap();
        let lib = get_library_with_status().unwrap();
        assert!(lib.songs[0].audio_missing);
        assert_eq!(lib.songs[0].id, song.id);
        cleanup(&dir);
    }

    #[test]
    fn test_get_library_with_status_empty() {
        let dir = make_temp_dir();
        init_app().unwrap();
        assert!(get_library_with_status().unwrap().songs.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn test_full_workflow() {
        let dir = make_temp_dir();
        init_app().unwrap();

        let song = make_song(&dir, "Workflow Song");
        assert_eq!(get_library().unwrap().songs.len(), 1);

        let updated = update_song(
            song.id.clone(),
            make_update(|u| {
                u.title = Some(SongTitle::new("Updated Workflow".to_string()));
            }),
        )
        .unwrap();
        assert_eq!(updated.title.as_str(), "Updated Workflow");

        assert!(check_audio_exists(song.id.clone()).unwrap());

        let new_audio = dir.join("workflow_v2.mp3");
        fs::write(&new_audio, "new").unwrap();
        let reassigned = reassign_audio_path(
            song.id.clone(),
            AudioPath::new(new_audio.to_str().unwrap().to_string()),
        )
        .unwrap();
        assert_eq!(reassigned.audio_path.as_str(), new_audio.to_str().unwrap());

        delete_song(song.id).unwrap();
        assert!(get_library().unwrap().songs.is_empty());
        cleanup(&dir);
    }
}
