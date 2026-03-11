use std::sync::Mutex;
use tauri::{Emitter, Manager};
use kvasir_core::{KvasFileTreeEntry, FileContent, AllFormats, JsonlInfo, JsonlEntry, TableData};

// ── Managed state for file-open-from-OS race condition ───────────────
struct PendingFile(Mutex<Option<String>>);

// ── Commands ─────────────────────────────────────────────────────────

#[tauri::command]
fn list_directory(directory: String, show_hidden: bool) -> Result<Vec<KvasFileTreeEntry>, String> {
    kvasir_core::list_directory(&directory, show_hidden)
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
fn read_jsonl_info(path: String) -> Result<JsonlInfo, String> {
    kvasir_core::read_jsonl_info(&path)
}

#[tauri::command]
fn read_jsonl_entry(path: String, index: usize) -> Result<JsonlEntry, String> {
    kvasir_core::read_jsonl_entry(&path, index)
}

#[tauri::command]
fn export_entry_as(content: String, format: String, source_name: String, index: usize) -> Result<String, String> {
    kvasir_core::export_entry_as(&content, &format, &source_name, index)
}

#[tauri::command]
fn read_table(path: String) -> Result<TableData, String> {
    kvasir_core::read_table(&path)
}

#[tauri::command]
fn export_table_csv(headers: Vec<String>, rows: Vec<Vec<String>>, source_path: String) -> Result<String, String> {
    kvasir_core::export_table_csv(headers, rows, &source_path)
}

#[tauri::command]
fn get_pending_file(state: tauri::State<PendingFile>) -> Option<String> {
    state.0.lock().ok()?.take()
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
            read_jsonl_info,
            read_jsonl_entry,
            export_entry_as,
            read_table,
            export_table_csv,
            get_pending_file,
        ])
        .build(tauri::generate_context!())
        .unwrap_or_else(|_| std::process::exit(1))
        .run(|app_handle, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = &event {
                for url in urls {
                    if let Ok(path) = url.to_file_path() {
                        let path_str = path.to_string_lossy().to_string();
                        let _ = app_handle.emit("open-file", &path_str);
                        if let Ok(mut pending) = app_handle.state::<PendingFile>().0.lock() {
                            *pending = Some(path_str);
                        }
                    }
                }
            }
        });
}
