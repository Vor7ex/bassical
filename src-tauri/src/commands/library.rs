use crate::models::song::{Library, Song};
use crate::persistence::storage;
use std::path::Path;

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

fn find_song_by_id<'a>(library: &'a Library, id: &str) -> Option<&'a Song> {
    library.songs.iter().find(|s| s.id == id)
}

fn find_song_by_id_mut<'a>(library: &'a mut Library, id: &str) -> Option<&'a mut Song> {
    library.songs.iter_mut().find(|s| s.id == id)
}

fn validate_audio_path(path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        return Err("El archivo de audio no existe".to_string());
    }
    Ok(())
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
pub fn add_song(title: String, artist: Option<String>, audio_path: String) -> Result<Song, String> {
    validate_audio_path(&audio_path)?;

    let mut library = load_library()?;
    let song = Song::new(title, artist, audio_path);
    library.songs.push(song.clone());
    save_library(&library)?;
    Ok(song)
}

#[tauri::command]
pub fn update_song(
    id: String,
    title: Option<String>,
    artist: Option<String>,
    audio_path: Option<String>,
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
        validate_audio_path(&path)?;
        song.audio_path = path;
        song.audio_missing = false;
    }
    song.updated_at = chrono::Utc::now().to_rfc3339();

    let updated_song = song.clone();
    save_library(&library)?;
    Ok(updated_song)
}

#[tauri::command]
pub fn delete_song(id: String) -> Result<(), String> {
    let mut library = load_library()?;
    library.songs.retain(|s| s.id != id);
    save_library(&library)?;
    Ok(())
}

#[tauri::command]
pub fn check_audio_exists(id: String) -> Result<bool, String> {
    let library = load_library()?;
    let song = find_song_by_id(&library, &id).ok_or_else(|| "Canción no encontrada".to_string())?;
    Ok(Path::new(&song.audio_path).exists())
}

#[tauri::command]
pub fn reassign_audio_path(id: String, new_path: String) -> Result<Song, String> {
    validate_audio_path(&new_path)?;

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
