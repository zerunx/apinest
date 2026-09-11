mod commands;
mod db;
mod error;
mod grpc;
mod nest;

use tauri::Manager;

use commands::*;
use db::Db;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("app data dir: {e}"))?;
            let database = Db::open(data_dir).map_err(|e| e.to_string())?;
            app.manage(database);
            app.manage(WsHub::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_session,
            login,
            logout,
            test_connection,
            sync_now,
            clear_local_cache,
            list_local_workspaces,
            list_local_projects,
            get_local_tree,
            get_interface,
            list_local_environments,
            list_local_variables,
            get_env_vars,
            get_active_context,
            set_active_context,
            create_local_workspace,
            update_local_workspace,
            delete_local_workspace,
            create_local_project,
            update_local_project,
            delete_local_project,
            create_local_folder,
            create_local_interface,
            update_local_interface,
            parse_proto_source,
            import_grpc_proto,
            invoke_grpc,
            export_apinest,
            import_apinest,
            save_json_file,
            reorder_local_interfaces,
            move_reorder_local_interfaces,
            list_local_folders,
            rename_local_folder,
            delete_local_folder,
            rename_local_interface,
            delete_local_interface,
            create_local_environment,
            update_local_environment,
            delete_local_environment,
            create_local_variable,
            update_local_variable,
            delete_local_variable,
            list_system_dictionaries,
            list_workspace_dictionaries,
            create_workspace_dictionary,
            update_workspace_dictionary,
            delete_workspace_dictionary,
            import_system_dictionary,
            send_http_request,
            ws_connect,
            ws_send,
            ws_disconnect,
            list_request_history,
            get_setting_value,
            set_setting_value,
            get_db_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
