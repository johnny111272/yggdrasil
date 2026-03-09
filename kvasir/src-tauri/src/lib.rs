use std::sync::Mutex;
use tauri::{Emitter, Manager};
use kvasir_core::{KvasFileTreeEntry, FileContent, AllFormats};

// ── Managed state for file-open-from-OS race condition ───────────────
struct PendingFile(Mutex<Option<String>>);

// ── Commands ─────────────────────────────────────────────────────────

#[tauri::command]
fn list_directory(directory: String) -> Result<Vec<KvasFileTreeEntry>, String> {
    kvasir_core::list_directory(&directory)
}

#[tauri::command]
fn read_file(path: String) -> Result<FileContent, String> {
    kvasir_core::read_file(&path)
}

#[tauri::command]
fn open_in_editor(path: String, line: usize) -> Result<(), String> {
    kvasir_core::open_in_editor(&path, line)
}

#[tauri::command]
fn convert_to_all_formats(content: String, source_format: String) -> Result<AllFormats, String> {
    kvasir_core::convert_to_all_formats(&content, &source_format)
}

#[tauri::command]
fn detect_data_format(path: String) -> Option<String> {
    kvasir_core::detect_data_format(&path)
}

#[tauri::command]
fn get_pending_file(state: tauri::State<PendingFile>) -> Option<String> {
    state.0.lock().unwrap().take()
}

// ── App Entry ────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(PendingFile(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            list_directory,
            read_file,
            open_in_editor,
            convert_to_all_formats,
            detect_data_format,
            get_pending_file,
        ])
        .build(tauri::generate_context!())
        .expect("Kvasir failed to build")
        .run(|app_handle, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = &event {
                for url in urls {
                    if let Ok(path) = url.to_file_path() {
                        let path_str = path.to_string_lossy().to_string();
                        *app_handle.state::<PendingFile>().0.lock().unwrap() =
                            Some(path_str.clone());
                        let _ = app_handle.emit("open-file", &path_str);
                    }
                }
            }
        });
}
