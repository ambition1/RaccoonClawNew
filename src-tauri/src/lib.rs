use std::env;
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

// Attempts to connect to port 7891 to check if the backend is ready.
fn wait_for_backend(timeout_secs: u64) -> bool {
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
    while std::time::Instant::now() < deadline {
        if TcpStream::connect("127.0.0.1:7891").is_ok() {
            return true;
        }
        thread::sleep(Duration::from_millis(250));
    }
    false
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Determine the backend entry point.
    // In development (debug builds) we launch via `openclaw` if available.
    // In the macOS app bundle the server is at:
    //   Contents/Resources/dashboard/server.py
    //   (The whole app bundle is at the repo root so we walk up from src-tauri.)
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();

    // From src-tauri/, go up two levels to reach the repo root.
    let repo_root = exe_dir
        .join("../../../..")
        .canonicalize()
        .unwrap_or_else(|_| exe_dir.clone());

    let server_py = repo_root.join("dashboard").join("server.py");
    let backend_arg = server_py.to_string_lossy().to_string();

    let python_bin = env::var("PYTHON_BIN")
        .unwrap_or_else(|_| "python3".to_string());

    println!("[RaccoonClaw] Starting backend: {} {}", python_bin, backend_arg);
    let child = Command::new(&python_bin)
        .arg(&backend_arg)
        .arg("--port")
        .arg("7891")
        .arg("--host")
        .arg("127.0.0.1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let backendPid = match child {
        Ok(c) => {
            println!("[RaccoonClaw] Backend started with PID {}", c.id());
            c.id()
        }
        Err(e) => {
            eprintln!("[RaccoonClaw] Failed to start backend: {}", e);
            0
        }
    };

    // Wait up to 30 seconds for the backend to become ready.
    let backend_ready = wait_for_backend(30);
    if backend_ready {
        println!("[RaccoonClaw] Backend ready at http://127.0.0.1:7891");
    } else {
        eprintln!(
            "[RaccoonClaw] WARNING: backend did not respond within 30s – window will load anyway"
        );
    }

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .setup(move |_app| {
            // Backend is already running (started above).
            // The webview is configured via tauri.conf.json to load http://127.0.0.1:7891.
            println!("[RaccoonClaw] Tauri app setup complete");
            Ok(())
        })
        .on_window_event(move |_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                println!("[RaccoonClaw] Window closing – stopping backend (PID {})", backendPid);
                if backendPid != 0 {
                    #[cfg(target_os = "macos")]
                    {
                        let _ = Command::new("pkill")
                            .arg("-f")
                            .arg("dashboard/server.py")
                            .arg("--port")
                            .arg("7891")
                            .output();
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        let _ = Command::new("taskkill")
                            .arg("/F")
                            .arg("/IM")
                            .arg("python.exe")
                            .output();
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
