#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

pub mod bridge;
pub mod domain;
pub mod runtime;
pub mod storage;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Olá, {name}! Vue está conectado ao Rust.")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let app = tauri::Builder::default()
        .manage(runtime::RuntimeState::default())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_local_data_dir()
                .map_err(|_| Box::new(storage::StorageError::Io) as Box<dyn std::error::Error>)?;
            let configured_runtime = runtime::RuntimeBuilder::new()
                .register(storage::StorageService::new(
                    data_dir.join(storage::DATABASE_FILE_NAME),
                ))
                .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)?
                .build();
            let state = app.state::<runtime::RuntimeState>();
            state
                .configure(configured_runtime)
                .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)?;
            let snapshot = state
                .start()
                .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)?;
            bridge::emit_bridge_status(app.handle(), &snapshot)
                .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            bridge::bridge_get_info,
            bridge::bridge_get_snapshot
        ])
        .build(tauri::generate_context!())?;

    app.run(|app_handle, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            if let Err(error) = app_handle.state::<runtime::RuntimeState>().shutdown() {
                eprintln!("Pulse runtime shutdown failed: {error}");
            }
        }
    });

    Ok(())
}
