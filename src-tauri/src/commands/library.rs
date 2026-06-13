use crate::models::song::{ArtistName, AudioPath, Library, Song, SongId, SongTitle};
use crate::persistence::storage;

const LIBRARY_FILE: &str = "library.json";

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
) -> Result<Song, String> {
    if !audio_path.exists() {
        return Err("El archivo de audio no existe".to_string());
    }

    let mut library = load_library()?;
    let song = Song::new(title, artist, audio_path);
    library.songs.push(song.clone());
    save_library(&library)?;
    Ok(song)
}

#[tauri::command]
pub fn update_song(
    id: SongId,
    title: Option<SongTitle>,
    artist: Option<ArtistName>,
    audio_path: Option<AudioPath>,
) -> Result<Song, String> {
    let mut library = load_library()?;
    let song = find_song_by_id_mut(&mut library, &id)
        .ok_or_else(|| "Canción no encontrada".to_string())?;

    if let Some(t) = title {
        song.title = t;
    }
    if let Some(a) = artist {
        song.artist = Some(a);
    }
    if let Some(path) = audio_path {
        if !path.exists() {
            return Err("El archivo de audio no existe".to_string());
        }
        song.audio_path = path;
        song.audio_missing = false;
    }
    song.updated_at = chrono::Utc::now().to_rfc3339();

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

    fn setup() -> PathBuf {
        let temp_dir = std::env::temp_dir().join(format!(
            "bassical_cmd_test_{:?}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&temp_dir).unwrap();
        storage::set_data_dir(temp_dir.clone());
        temp_dir
    }

    fn teardown(temp_dir: PathBuf) {
        storage::clear_data_dir();
        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_init_app_creates_dirs_and_library() {
        let temp_dir = setup();

        let result = init_app();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "App initialized");
        assert!(temp_dir.exists());
        assert!(temp_dir.join("songs").exists());
        assert!(temp_dir.join("library.json").exists());

        teardown(temp_dir);
    }

    #[test]
    fn test_init_app_idempotent() {
        let temp_dir = setup();

        assert!(init_app().is_ok());
        assert!(init_app().is_ok());

        teardown(temp_dir);
    }

    #[test]
    fn test_get_library_empty_after_init() {
        let temp_dir = setup();
        init_app().unwrap();

        let result = get_library();
        assert!(result.is_ok());
        assert!(result.unwrap().songs.is_empty());

        teardown(temp_dir);
    }

    #[test]
    fn test_get_library_creates_empty_if_missing() {
        let temp_dir = setup();

        let result = get_library();
        assert!(result.is_ok());
        assert!(result.unwrap().songs.is_empty());

        teardown(temp_dir);
    }

    #[test]
    fn test_add_song_success() {
        let temp_dir = setup();
        init_app().unwrap();

        let audio_file = temp_dir.join("test_add.mp3");
        fs::write(&audio_file, "fake audio").unwrap();

        let title = SongTitle::new("Test Song".to_string());
        let artist = Some(ArtistName::new("Test Artist".to_string()));
        let audio_path = AudioPath::new(audio_file.to_str().unwrap().to_string());

        let result = add_song(title, artist, audio_path);
        assert!(result.is_ok());

        let song = result.unwrap();
        assert_eq!(song.title.as_str(), "Test Song");
        assert_eq!(song.artist.unwrap().as_str(), "Test Artist");

        let library = get_library().unwrap();
        assert_eq!(library.songs.len(), 1);
        assert_eq!(library.songs[0].id, song.id);

        teardown(temp_dir);
    }

    #[test]
    fn test_add_song_no_artist() {
        let temp_dir = setup();
        init_app().unwrap();

        let audio_file = temp_dir.join("test_no_artist.mp3");
        fs::write(&audio_file, "fake audio").unwrap();

        let title = SongTitle::new("No Artist".to_string());
        let audio_path = AudioPath::new(audio_file.to_str().unwrap().to_string());

        let result = add_song(title, None, audio_path);
        assert!(result.is_ok());
        assert!(result.unwrap().artist.is_none());

        teardown(temp_dir);
    }

    #[test]
    fn test_add_song_nonexistent_audio_fails() {
        let temp_dir = setup();
        init_app().unwrap();

        let title = SongTitle::new("Bad Song".to_string());
        let audio_path = AudioPath::new("/nonexistent/file.mp3".to_string());

        let result = add_song(title, None, audio_path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "El archivo de audio no existe");

        let library = get_library().unwrap();
        assert!(library.songs.is_empty());

        teardown(temp_dir);
    }

    #[test]
    fn test_add_multiple_songs() {
        let temp_dir = setup();
        init_app().unwrap();

        for i in 0..3 {
            let audio_file = temp_dir.join(format!("test_{i}.mp3"));
            fs::write(&audio_file, "fake audio").unwrap();

            let title = SongTitle::new(format!("Song {i}"));
            let audio_path = AudioPath::new(audio_file.to_str().unwrap().to_string());
            add_song(title, None, audio_path).unwrap();
        }

        let library = get_library().unwrap();
        assert_eq!(library.songs.len(), 3);

        teardown(temp_dir);
    }

    #[test]
    fn test_update_song_title() {
        let temp_dir = setup();
        init_app().unwrap();

        let audio_file = temp_dir.join("test_update.mp3");
        fs::write(&audio_file, "fake audio").unwrap();

        let title = SongTitle::new("Original".to_string());
        let audio_path = AudioPath::new(audio_file.to_str().unwrap().to_string());
        let song = add_song(title, None, audio_path).unwrap();

        let new_title = SongTitle::new("Updated".to_string());
        let result = update_song(song.id.clone(), Some(new_title), None, None);
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.title.as_str(), "Updated");
        assert!(updated.artist.is_none());

        teardown(temp_dir);
    }

    #[test]
    fn test_update_song_artist() {
        let temp_dir = setup();
        init_app().unwrap();

        let audio_file = temp_dir.join("test_update_artist.mp3");
        fs::write(&audio_file, "fake audio").unwrap();

        let title = SongTitle::new("Song".to_string());
        let audio_path = AudioPath::new(audio_file.to_str().unwrap().to_string());
        let song = add_song(title, None, audio_path).unwrap();

        let new_artist = ArtistName::new("New Artist".to_string());
        let result = update_song(song.id.clone(), None, Some(new_artist), None);
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.artist.unwrap().as_str(), "New Artist");

        teardown(temp_dir);
    }

    #[test]
    fn test_update_song_audio_path() {
        let temp_dir = setup();
        init_app().unwrap();

        let audio_file1 = temp_dir.join("test1.mp3");
        let audio_file2 = temp_dir.join("test2.mp3");
        fs::write(&audio_file1, "audio 1").unwrap();
        fs::write(&audio_file2, "audio 2").unwrap();

        let title = SongTitle::new("Song".to_string());
        let audio_path = AudioPath::new(audio_file1.to_str().unwrap().to_string());
        let song = add_song(title, None, audio_path).unwrap();

        let new_path = AudioPath::new(audio_file2.to_str().unwrap().to_string());
        let result = update_song(song.id.clone(), None, None, Some(new_path));
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.audio_path.as_str(), audio_file2.to_str().unwrap());
        assert!(!updated.audio_missing);

        teardown(temp_dir);
    }

    #[test]
    fn test_update_song_nonexistent_audio_fails() {
        let temp_dir = setup();
        init_app().unwrap();

        let audio_file = temp_dir.join("test.mp3");
        fs::write(&audio_file, "fake audio").unwrap();

        let title = SongTitle::new("Song".to_string());
        let audio_path = AudioPath::new(audio_file.to_str().unwrap().to_string());
        let song = add_song(title, None, audio_path).unwrap();

        let bad_path = AudioPath::new("/nonexistent/file.mp3".to_string());
        let result = update_song(song.id.clone(), None, None, Some(bad_path));
        assert!(result.is_err());

        teardown(temp_dir);
    }

    #[test]
    fn test_update_song_not_found() {
        let temp_dir = setup();
        init_app().unwrap();

        let fake_id = SongId::new("nonexistent-id".to_string());
        let result = update_song(fake_id, Some(SongTitle::new("X".to_string())), None, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Canción no encontrada");

        teardown(temp_dir);
    }

    #[test]
    fn test_update_song_no_changes() {
        let temp_dir = setup();
        init_app().unwrap();

        let audio_file = temp_dir.join("test.mp3");
        fs::write(&audio_file, "fake audio").unwrap();

        let title = SongTitle::new("Song".to_string());
        let audio_path = AudioPath::new(audio_file.to_str().unwrap().to_string());
        let song = add_song(title, None, audio_path).unwrap();

        let result = update_song(song.id.clone(), None, None, None);
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.title.as_str(), "Song");
        assert!(updated.updated_at >= song.updated_at);

        teardown(temp_dir);
    }

    #[test]
    fn test_delete_song() {
        let temp_dir = setup();
        init_app().unwrap();

        let audio_file = temp_dir.join("test.mp3");
        fs::write(&audio_file, "fake audio").unwrap();

        let title = SongTitle::new("Delete Me".to_string());
        let audio_path = AudioPath::new(audio_file.to_str().unwrap().to_string());
        let song = add_song(title, None, audio_path).unwrap();

        assert_eq!(get_library().unwrap().songs.len(), 1);

        let result = delete_song(song.id);
        assert!(result.is_ok());
        assert!(get_library().unwrap().songs.is_empty());

        teardown(temp_dir);
    }

    #[test]
    fn test_delete_song_not_found_succeeds() {
        let temp_dir = setup();
        init_app().unwrap();

        let fake_id = SongId::new("nonexistent".to_string());
        let result = delete_song(fake_id);
        assert!(result.is_ok());

        teardown(temp_dir);
    }

    #[test]
    fn test_delete_one_of_many() {
        let temp_dir = setup();
        init_app().unwrap();

        let mut song_ids = Vec::new();
        for i in 0..3 {
            let audio_file = temp_dir.join(format!("test_{i}.mp3"));
            fs::write(&audio_file, "fake audio").unwrap();

            let title = SongTitle::new(format!("Song {i}"));
            let audio_path = AudioPath::new(audio_file.to_str().unwrap().to_string());
            let song = add_song(title, None, audio_path).unwrap();
            song_ids.push(song.id);
        }

        assert_eq!(get_library().unwrap().songs.len(), 3);

        delete_song(song_ids[1].clone()).unwrap();

        let library = get_library().unwrap();
        assert_eq!(library.songs.len(), 2);
        assert!(library.songs.iter().all(|s| s.id != song_ids[1]));

        teardown(temp_dir);
    }

    #[test]
    fn test_check_audio_exists_true() {
        let temp_dir = setup();
        init_app().unwrap();

        let audio_file = temp_dir.join("exists.mp3");
        fs::write(&audio_file, "fake audio").unwrap();

        let title = SongTitle::new("Song".to_string());
        let audio_path = AudioPath::new(audio_file.to_str().unwrap().to_string());
        let song = add_song(title, None, audio_path).unwrap();

        let result = check_audio_exists(song.id);
        assert!(result.is_ok());
        assert!(result.unwrap());

        teardown(temp_dir);
    }

    #[test]
    fn test_check_audio_exists_false_after_delete() {
        let temp_dir = setup();
        init_app().unwrap();

        let audio_file = temp_dir.join("will_delete.mp3");
        fs::write(&audio_file, "fake audio").unwrap();

        let title = SongTitle::new("Song".to_string());
        let audio_path = AudioPath::new(audio_file.to_str().unwrap().to_string());
        let song = add_song(title, None, audio_path).unwrap();

        fs::remove_file(&audio_file).unwrap();

        let result = check_audio_exists(song.id);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        teardown(temp_dir);
    }

    #[test]
    fn test_check_audio_exists_not_found() {
        let temp_dir = setup();
        init_app().unwrap();

        let fake_id = SongId::new("nonexistent".to_string());
        let result = check_audio_exists(fake_id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Canción no encontrada");

        teardown(temp_dir);
    }

    #[test]
    fn test_reassign_audio_path_success() {
        let temp_dir = setup();
        init_app().unwrap();

        let audio_file1 = temp_dir.join("old.mp3");
        let audio_file2 = temp_dir.join("new.mp3");
        fs::write(&audio_file1, "old audio").unwrap();
        fs::write(&audio_file2, "new audio").unwrap();

        let title = SongTitle::new("Song".to_string());
        let audio_path = AudioPath::new(audio_file1.to_str().unwrap().to_string());
        let song = add_song(title, None, audio_path).unwrap();

        let new_path = AudioPath::new(audio_file2.to_str().unwrap().to_string());
        let result = reassign_audio_path(song.id.clone(), new_path);
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.audio_path.as_str(), audio_file2.to_str().unwrap());
        assert!(!updated.audio_missing);

        teardown(temp_dir);
    }

    #[test]
    fn test_reassign_audio_path_nonexistent_fails() {
        let temp_dir = setup();
        init_app().unwrap();

        let audio_file = temp_dir.join("original.mp3");
        fs::write(&audio_file, "fake audio").unwrap();

        let title = SongTitle::new("Song".to_string());
        let audio_path = AudioPath::new(audio_file.to_str().unwrap().to_string());
        let song = add_song(title, None, audio_path).unwrap();

        let bad_path = AudioPath::new("/nonexistent/new.mp3".to_string());
        let result = reassign_audio_path(song.id, bad_path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "El archivo de audio no existe");

        teardown(temp_dir);
    }

    #[test]
    fn test_reassign_audio_path_not_found() {
        let temp_dir = setup();
        init_app().unwrap();

        let audio_file = temp_dir.join("new.mp3");
        fs::write(&audio_file, "fake audio").unwrap();

        let fake_id = SongId::new("nonexistent".to_string());
        let new_path = AudioPath::new(audio_file.to_str().unwrap().to_string());
        let result = reassign_audio_path(fake_id, new_path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Canción no encontrada");

        teardown(temp_dir);
    }

    #[test]
    fn test_full_workflow() {
        let temp_dir = setup();

        // Init
        init_app().unwrap();

        // Add song
        let audio_file = temp_dir.join("workflow.mp3");
        fs::write(&audio_file, "fake audio").unwrap();
        let title = SongTitle::new("Workflow Song".to_string());
        let artist = Some(ArtistName::new("Workflow Artist".to_string()));
        let audio_path = AudioPath::new(audio_file.to_str().unwrap().to_string());
        let song = add_song(title, artist, audio_path).unwrap();

        // Verify library
        let library = get_library().unwrap();
        assert_eq!(library.songs.len(), 1);

        // Update
        let new_title = SongTitle::new("Updated Workflow".to_string());
        let updated = update_song(song.id.clone(), Some(new_title), None, None).unwrap();
        assert_eq!(updated.title.as_str(), "Updated Workflow");

        // Check audio
        assert!(check_audio_exists(song.id.clone()).unwrap());

        // Reassign
        let new_audio = temp_dir.join("workflow_v2.mp3");
        fs::write(&new_audio, "new audio").unwrap();
        let new_path = AudioPath::new(new_audio.to_str().unwrap().to_string());
        let reassigned = reassign_audio_path(song.id.clone(), new_path).unwrap();
        assert_eq!(reassigned.audio_path.as_str(), new_audio.to_str().unwrap());

        // Delete
        delete_song(song.id).unwrap();
        assert!(get_library().unwrap().songs.is_empty());

        teardown(temp_dir);
    }
}
