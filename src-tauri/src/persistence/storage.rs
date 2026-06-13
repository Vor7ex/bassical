use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn get_app_data_dir() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("Bassical");
    path
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
