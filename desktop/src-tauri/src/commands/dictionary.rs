use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::auth::current_server_id;
use crate::db::{next_local_id, now_iso, Db};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
pub struct SystemDictionaryItem {
    pub key: String,
    pub name: String,
    pub field_type: String,
    pub description: String,
    pub default_value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DictionaryEntryRow {
    pub id: i64,
    pub workspace_id: i64,
    pub name: String,
    pub field_type: String,
    pub description: String,
    pub default_value: String,
    pub source_system_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDictionaryArgs {
    pub workspace_id: i64,
    pub name: String,
    pub field_type: String,
    pub description: Option<String>,
    pub default_value: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDictionaryArgs {
    pub id: i64,
    pub workspace_id: i64,
    pub name: String,
    pub field_type: String,
    pub description: Option<String>,
    pub default_value: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImportSystemDictionaryArgs {
    pub workspace_id: i64,
    pub system_key: String,
}

fn system_dictionaries() -> Vec<SystemDictionaryItem> {
    vec![
        SystemDictionaryItem {
            key: "page".into(),
            name: "page".into(),
            field_type: "integer".into(),
            description: "页码".into(),
            default_value: "1".into(),
        },
        SystemDictionaryItem {
            key: "page_size".into(),
            name: "pageSize".into(),
            field_type: "integer".into(),
            description: "每页数量".into(),
            default_value: "20".into(),
        },
        SystemDictionaryItem {
            key: "keyword".into(),
            name: "keyword".into(),
            field_type: "string".into(),
            description: "搜索关键词".into(),
            default_value: String::new(),
        },
        SystemDictionaryItem {
            key: "id".into(),
            name: "id".into(),
            field_type: "integer".into(),
            description: "主键 ID".into(),
            default_value: String::new(),
        },
        SystemDictionaryItem {
            key: "status".into(),
            name: "status".into(),
            field_type: "integer".into(),
            description: "状态".into(),
            default_value: "0".into(),
        },
        SystemDictionaryItem {
            key: "authorization".into(),
            name: "Authorization".into(),
            field_type: "string".into(),
            description: "鉴权请求头".into(),
            default_value: "Bearer ".into(),
        },
        SystemDictionaryItem {
            key: "content_type".into(),
            name: "Content-Type".into(),
            field_type: "string".into(),
            description: "内容类型".into(),
            default_value: "application/json".into(),
        },
        SystemDictionaryItem {
            key: "timestamp".into(),
            name: "timestamp".into(),
            field_type: "integer".into(),
            description: "时间戳".into(),
            default_value: String::new(),
        },
        SystemDictionaryItem {
            key: "file".into(),
            name: "file".into(),
            field_type: "file".into(),
            description: "上传文件".into(),
            default_value: String::new(),
        },
    ]
}

fn normalize_field_type(raw: &str) -> AppResult<String> {
    let t = raw.trim().to_lowercase();
    match t.as_str() {
        "string" | "integer" | "number" | "boolean" | "file" => Ok(t),
        "" => Ok("string".into()),
        _ => Err(AppError::msg("unsupported field type")),
    }
}

#[tauri::command]
pub fn list_system_dictionaries() -> AppResult<Vec<SystemDictionaryItem>> {
    Ok(system_dictionaries())
}

#[tauri::command]
pub fn list_workspace_dictionaries(
    db: State<'_, Db>,
    workspace_id: i64,
) -> AppResult<Vec<DictionaryEntryRow>> {
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, workspace_id, name, field_type, description, default_value, source_system_key
             FROM dictionary_entries
             WHERE server_id = ?1 AND workspace_id = ?2
             ORDER BY sort_order ASC, id ASC",
        )?;
        let rows = stmt
            .query_map(params![server_id, workspace_id], |r| {
                Ok(DictionaryEntryRow {
                    id: r.get(0)?,
                    workspace_id: r.get(1)?,
                    name: r.get(2)?,
                    field_type: r.get(3)?,
                    description: r.get(4)?,
                    default_value: r.get(5)?,
                    source_system_key: r.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}

#[tauri::command]
pub fn create_workspace_dictionary(
    db: State<'_, Db>,
    args: CreateDictionaryArgs,
) -> AppResult<DictionaryEntryRow> {
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::msg("name is required"));
    }
    let field_type = normalize_field_type(&args.field_type)?;
    let description = args.description.unwrap_or_default().trim().to_string();
    let default_value = args.default_value.unwrap_or_default();
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let dup: bool = conn
            .query_row(
                "SELECT 1 FROM dictionary_entries
                 WHERE server_id = ?1 AND workspace_id = ?2 AND name = ?3",
                params![server_id, args.workspace_id, name],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if dup {
            return Err(AppError::msg("name already exists"));
        }
        let id = next_local_id(conn)?;
        let now = now_iso();
        let sort_order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM dictionary_entries
                 WHERE server_id = ?1 AND workspace_id = ?2",
                params![server_id, args.workspace_id],
                |r| r.get(0),
            )
            .unwrap_or(1);
        conn.execute(
            "INSERT INTO dictionary_entries(
               id, server_id, workspace_id, name, field_type, description, default_value,
               source_system_key, sort_order, synced_at, is_local
             ) VALUES(?1,?2,?3,?4,?5,?6,?7,NULL,?8,?9,1)",
            params![
                id,
                server_id,
                args.workspace_id,
                name,
                field_type,
                description,
                default_value,
                sort_order,
                now
            ],
        )?;
        Ok(DictionaryEntryRow {
            id,
            workspace_id: args.workspace_id,
            name,
            field_type,
            description,
            default_value,
            source_system_key: None,
        })
    })
}

#[tauri::command]
pub fn update_workspace_dictionary(
    db: State<'_, Db>,
    args: UpdateDictionaryArgs,
) -> AppResult<DictionaryEntryRow> {
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::msg("name is required"));
    }
    let field_type = normalize_field_type(&args.field_type)?;
    let description = args.description.unwrap_or_default().trim().to_string();
    let default_value = args.default_value.unwrap_or_default();
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let dup: bool = conn
            .query_row(
                "SELECT 1 FROM dictionary_entries
                 WHERE server_id = ?1 AND workspace_id = ?2 AND name = ?3 AND id != ?4",
                params![server_id, args.workspace_id, name, args.id],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if dup {
            return Err(AppError::msg("name already exists"));
        }
        let now = now_iso();
        let n = conn.execute(
            "UPDATE dictionary_entries SET
               name = ?1, field_type = ?2, description = ?3, default_value = ?4,
               synced_at = ?5, is_local = 1
             WHERE server_id = ?6 AND workspace_id = ?7 AND id = ?8",
            params![
                name,
                field_type,
                description,
                default_value,
                now,
                server_id,
                args.workspace_id,
                args.id
            ],
        )?;
        if n == 0 {
            return Err(AppError::msg("dictionary entry not found"));
        }
        let source_system_key: Option<String> = conn
            .query_row(
                "SELECT source_system_key FROM dictionary_entries
                 WHERE server_id = ?1 AND id = ?2",
                params![server_id, args.id],
                |r| r.get(0),
            )
            .ok();
        Ok(DictionaryEntryRow {
            id: args.id,
            workspace_id: args.workspace_id,
            name,
            field_type,
            description,
            default_value,
            source_system_key,
        })
    })
}

#[tauri::command]
pub fn delete_workspace_dictionary(
    db: State<'_, Db>,
    workspace_id: i64,
    entry_id: i64,
) -> AppResult<()> {
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let n = conn.execute(
            "DELETE FROM dictionary_entries
             WHERE server_id = ?1 AND workspace_id = ?2 AND id = ?3",
            params![server_id, workspace_id, entry_id],
        )?;
        if n == 0 {
            return Err(AppError::msg("dictionary entry not found"));
        }
        Ok(())
    })
}

#[tauri::command]
pub fn import_system_dictionary(
    db: State<'_, Db>,
    args: ImportSystemDictionaryArgs,
) -> AppResult<DictionaryEntryRow> {
    let item = system_dictionaries()
        .into_iter()
        .find(|x| x.key == args.system_key)
        .ok_or_else(|| AppError::msg("system dictionary not found"))?;
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let already: bool = conn
            .query_row(
                "SELECT 1 FROM dictionary_entries
                 WHERE server_id = ?1 AND workspace_id = ?2 AND source_system_key = ?3",
                params![server_id, args.workspace_id, item.key],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if already {
            return Err(AppError::msg("system dictionary already imported"));
        }
        let name_dup: bool = conn
            .query_row(
                "SELECT 1 FROM dictionary_entries
                 WHERE server_id = ?1 AND workspace_id = ?2 AND name = ?3",
                params![server_id, args.workspace_id, item.name],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if name_dup {
            return Err(AppError::msg("a custom field with the same name already exists"));
        }
        let id = next_local_id(conn)?;
        let now = now_iso();
        let sort_order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM dictionary_entries
                 WHERE server_id = ?1 AND workspace_id = ?2",
                params![server_id, args.workspace_id],
                |r| r.get(0),
            )
            .unwrap_or(1);
        conn.execute(
            "INSERT INTO dictionary_entries(
               id, server_id, workspace_id, name, field_type, description, default_value,
               source_system_key, sort_order, synced_at, is_local
             ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,1)",
            params![
                id,
                server_id,
                args.workspace_id,
                item.name,
                item.field_type,
                item.description,
                item.default_value,
                item.key,
                sort_order,
                now
            ],
        )?;
        Ok(DictionaryEntryRow {
            id,
            workspace_id: args.workspace_id,
            name: item.name,
            field_type: item.field_type,
            description: item.description,
            default_value: item.default_value,
            source_system_key: Some(item.key),
        })
    })
}

pub fn delete_workspace_dictionaries(
    conn: &rusqlite::Connection,
    server_id: i64,
    workspace_id: i64,
) -> AppResult<()> {
    conn.execute(
        "DELETE FROM dictionary_entries WHERE server_id = ?1 AND workspace_id = ?2",
        params![server_id, workspace_id],
    )?;
    Ok(())
}
