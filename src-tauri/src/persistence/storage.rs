#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

thread_local! {
    static CUSTOM_DATA_DIR: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

pub fn set_data_dir(path: PathBuf) {
    CUSTOM_DATA_DIR.with(|dir| {
        *dir.borrow_mut() = Some(path);
    });
}

pub fn clear_data_dir() {
    CUSTOM_DATA_DIR.with(|dir| {
        *dir.borrow_mut() = None;
    });
}

fn get_app_data_dir() -> PathBuf {
    CUSTOM_DATA_DIR.with(|dir| {
        if let Some(path) = dir.borrow().clone() {
            return path;
        }
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("Bassical");
        path
    })
}

pub fn ensure_app_data_dir() -> Result<(), String> {
    let dir = get_app_data_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(dir.join("songs")).map_err(|e| e.to_string())?;
    Ok(())
}

/// Ruta del directorio `songs/` dentro de AppData.
pub fn songs_dir() -> PathBuf {
    get_app_data_dir().join("songs")
}

/// Ruta del archivo `.bassical.json` de una canción: `songs/<id>.bassical.json`.
pub fn song_file(song_id: &str) -> PathBuf {
    songs_dir().join(format!("{song_id}.bassical.json"))
}

/// Lee el `.bassical.json` de una canción. Devuelve `Ok(None)` si el archivo
/// no existe (canción sin calibración/tab aún), error solo si existe pero
/// está corrupto.
pub fn read_song<T: for<'de> Deserialize<'de>>(song_id: &str) -> Result<Option<T>, String> {
    let path = song_file(song_id);
    match fs::read_to_string(&path) {
        Ok(content) => {
            let parsed = serde_json::from_str(&content).map_err(|e| e.to_string())?;
            Ok(Some(parsed))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Escribe el `.bassical.json` de una canción (atomic-ish: pretty JSON).
pub fn write_song<T: Serialize>(song_id: &str, data: &T) -> Result<(), String> {
    let path = song_file(song_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}

/// Elimina el `.bassical.json` de una canción. No falla si no existe.
pub fn delete_song_file(song_id: &str) -> Result<(), String> {
    let path = song_file(song_id);
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn read_json<T: for<'de> Deserialize<'de>>(filename: &str) -> Result<T, String> {
    let path = get_app_data_dir().join(filename);
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub fn write_json<T: Serialize>(filename: &str, data: &T) -> Result<(), String> {
    let path = get_app_data_dir().join(filename);
    let content = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn setup() -> PathBuf {
        let temp_dir = std::env::temp_dir().join(format!(
            "bassical_test_{:?}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&temp_dir).unwrap();
        set_data_dir(temp_dir.clone());
        temp_dir
    }

    fn teardown(temp_dir: PathBuf) {
        clear_data_dir();
        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_ensure_app_data_dir_creates_structure() {
        let temp_dir = setup();
        let result = ensure_app_data_dir();
        assert!(result.is_ok());
        assert!(temp_dir.exists());
        assert!(temp_dir.join("songs").exists());
        teardown(temp_dir);
    }

    #[test]
    fn test_write_read_json_roundtrip() {
        let temp_dir = setup();
        fs::create_dir_all(&temp_dir).unwrap();

        let data = serde_json::json!({"key": "value", "number": 42});
        let result = write_json("test_roundtrip.json", &data);
        assert!(result.is_ok());

        let read_data: serde_json::Value = read_json("test_roundtrip.json").unwrap();
        assert_eq!(read_data, data);

        teardown(temp_dir);
    }

    #[test]
    fn test_read_json_missing_file() {
        let temp_dir = setup();
        let result: Result<serde_json::Value, String> = read_json("nonexistent.json");
        assert!(result.is_err());
        teardown(temp_dir);
    }

    #[test]
    fn test_read_json_invalid_content() {
        let temp_dir = setup();
        let path = temp_dir.join("invalid.json");
        fs::write(&path, "not valid json{{{").unwrap();

        let result: Result<serde_json::Value, String> = read_json("invalid.json");
        assert!(result.is_err());
        teardown(temp_dir);
    }

    #[test]
    fn test_song_file_path_under_songs_dir() {
        let temp_dir = setup();
        let path = song_file("abc-123");
        assert!(path.starts_with(temp_dir.join("songs")));
        assert_eq!(path.file_name().unwrap(), "abc-123.bassical.json");
        teardown(temp_dir);
    }

    #[test]
    fn test_read_song_returns_none_when_missing() {
        let temp_dir = setup();
        ensure_app_data_dir().unwrap();
        let result: Option<serde_json::Value> = read_song("no-such-id").unwrap();
        assert!(result.is_none());
        teardown(temp_dir);
    }

    #[test]
    fn test_write_then_read_song_roundtrip() {
        let temp_dir = setup();
        ensure_app_data_dir().unwrap();
        let data = serde_json::json!({"schemaVersion": 1, "id": "s1", "timingPoints": []});
        write_song("s1", &data).unwrap();
        let back: Option<serde_json::Value> = read_song("s1").unwrap();
        assert_eq!(back, Some(data));
        assert!(temp_dir.join("songs").join("s1.bassical.json").exists());
        teardown(temp_dir);
    }

    #[test]
    fn test_read_song_error_on_corrupt() {
        let temp_dir = setup();
        ensure_app_data_dir().unwrap();
        fs::write(
            temp_dir.join("songs").join("bad.bassical.json"),
            "{ not json",
        )
        .unwrap();
        let result: Result<Option<serde_json::Value>, String> = read_song("bad");
        assert!(result.is_err());
        teardown(temp_dir);
    }

    #[test]
    fn test_write_song_creates_songs_dir_if_missing() {
        let temp_dir = setup();
        // No llamamos a ensure_app_data_dir; write_song debe crear el padre.
        let data = serde_json::json!({"id": "s2"});
        write_song("s2", &data).unwrap();
        assert!(temp_dir.join("songs").join("s2.bassical.json").exists());
        teardown(temp_dir);
    }

    #[test]
    fn test_delete_song_file_idempotent() {
        let temp_dir = setup();
        ensure_app_data_dir().unwrap();
        // No existe: no error.
        assert!(delete_song_file("ghost").is_ok());
        // Existe: lo borra.
        let data = serde_json::json!({"id": "s3"});
        write_song("s3", &data).unwrap();
        assert!(delete_song_file("s3").is_ok());
        assert!(!temp_dir.join("songs").join("s3.bassical.json").exists());
        // Ya borrado: no error.
        assert!(delete_song_file("s3").is_ok());
        teardown(temp_dir);
    }
}
