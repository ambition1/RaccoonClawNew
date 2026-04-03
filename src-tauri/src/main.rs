// RaccoonClaw OSS - macOS APP
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::thread;
use std::time::Duration;

fn main() {
    // Start OpenClaw gateway (auto-starts the backend)
    let _ = Command::new("openclaw")
        .args(["gateway", "start"])
        .spawn();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                std::thread::spawn(move || {
                    // Wait for backend to be ready
                    for _ in 0..60 {
                        if Command::new("sh")
                            .args(["-c", "curl -s --max-time 1 http://localhost:7891 > /dev/null"])
                            .output()
                            .map(|o| o.status.success())
                            .unwrap_or(false)
                        {
                            let _ = w.eval("window.location.href = 'http://localhost:7891'");
                            return;
                        }
                        std::thread::sleep(Duration::from_secs(1));
                    }
                    // Fallback: open in default browser
                    let _ = Command::new("open")
                        .arg("http://localhost:7891")
                        .spawn();
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
