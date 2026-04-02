// RaccoonClaw OSS - macOS APP
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::thread;
use std::time::Duration;
use std::env;

fn main() {
    let home = env::var("HOME").unwrap_or_default();
    // Find the repo directory - support both original and linked paths
    let repo = env::var("REPO_DIR").unwrap_or_else(|_| {
        let path1 = format!("{}/Documents/RaccoonClaw", home);
        let path2 = format!("{}/Documents/033009RaccoonClaw-OSS", home);
        if std::path::Path::new(&path1).exists() { path1 } else { path2 }
    });

    let backend_cmd = format!(
        "cd '{}' && ./.venv-backend/bin/uvicorn Raccoon.backend.app.main:app --host 127.0.0.1 --port 7891",
        repo.replace("'", "'\"'\"'")
    );

    // Start backend in background
    let _ = Command::new("sh")
        .arg("-c")
        .arg(&backend_cmd)
        .spawn();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let repo_clone = repo.clone();
            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                // Wait for backend then navigate
                std::thread::spawn(move || {
                    for _ in 0..30 {
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
                    eprintln!("Backend did not start in 30s");
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
