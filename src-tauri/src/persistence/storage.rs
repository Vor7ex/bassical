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
}
