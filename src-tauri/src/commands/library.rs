use crate::persistence::storage;

#[tauri::command]
pub fn init_app() -> Result<String, String> {
    storage::ensure_app_data_dir()?;
    Ok("App initialized".to_string())
}
