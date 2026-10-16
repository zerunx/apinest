use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::db::{get_setting, set_setting, Db};
use crate::error::{AppError, AppResult};

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

#[tauri::command]
pub async fn export_local_database(app: tauri::AppHandle, db: State<'_, Db>) -> AppResult<bool> {
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let default_name = format!("apinest-backup-{stamp}.db");

    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_file_name(&default_name)
        .add_filter("ApiNest Database", &["db"])
        .save_file(move |file| {
            let _ = tx.send(file);
        });

    let Some(file) = rx.await.unwrap_or(None) else {
        return Ok(false);
    };
    let path = file
        .into_path()
        .map_err(|e| AppError::msg(format!("invalid path: {e}")))?;

    db.export_to(&path)?;
    Ok(true)
}

#[tauri::command]
pub async fn import_local_database(app: tauri::AppHandle, db: State<'_, Db>) -> AppResult<bool> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("ApiNest Database", &["db"])
        .add_filter("All files", &["*"])
        .pick_file(move |file| {
            let _ = tx.send(file);
        });

    let Some(file) = rx.await.unwrap_or(None) else {
        return Ok(false);
    };
    let path = file
        .into_path()
        .map_err(|e| AppError::msg(format!("invalid path: {e}")))?;

    db.import_from(&path)?;
    Ok(true)
}
