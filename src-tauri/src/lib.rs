#[cfg_attr(mobile, tauri::mobile_entry_point)]
mod commands;
use commands::*;
use tauri::generate_handler;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(generate_handler![
          create_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
