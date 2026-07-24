use tauri::State;

use crate::db::{get_setting, set_setting, Db};
use crate::error::AppResult;

#[tauri::command]
pub fn get_setting_value(db: State<'_, Db>, key: String) -> AppResult<Option<String>> {
    db.with_conn(|conn| get_setting(conn, &key))
}

#[tauri::command]
pub fn set_setting_value(db: State<'_, Db>, key: String, value: String) -> AppResult<()> {
    db.with_conn(|conn| set_setting(conn, &key, &value))
}

#[tauri::command]
pub fn get_db_path(db: State<'_, Db>) -> AppResult<String> {
    Ok(db.path.display().to_string())
}
