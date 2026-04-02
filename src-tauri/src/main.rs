// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Open browser to the RaccoonClaw UI
            let url = "http://localhost:7891";
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.eval(&format!(
                    "window.location.href = '{}'",
                    url
                ));
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
