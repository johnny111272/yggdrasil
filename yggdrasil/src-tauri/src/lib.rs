use std::sync::Mutex;
use tauri::{Emitter, Manager};

// ── Managed state for file-open-from-OS race condition ───────────────
struct PendingFile(Mutex<Option<String>>);

// ============================================================================
// Hlidskjalf commands (hlid_ prefix)
// ============================================================================

#[tauri::command]
async fn hlid_start_monitor(app_handle: tauri::AppHandle) -> Result<(), String> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    hlidskjalf_core::start_all(tx).await?;
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = app_handle.emit("datagram", &event);
        }
    });
    Ok(())
}

#[tauri::command]
fn hlid_speak(text: String) {
    hlidskjalf_core::speak(&text);
}

#[tauri::command]
fn hlid_open_in_editor(path: String, line: usize) -> Result<(), String> {
    common_core::open_in_editor(&path, line)
}

// ============================================================================
// Svalinn commands (sval_ prefix)
// ============================================================================

#[tauri::command]
fn sval_scan_directory(directory: String, include_tests: bool) -> Result<svalinn_core::ScanResult, String> {
    svalinn_core::scan_directory(&directory, include_tests)
}

#[tauri::command]
fn sval_open_in_editor(path: String, line: usize) -> Result<(), String> {
    svalinn_core::open_in_editor(&path, line)
}

#[tauri::command]
fn sval_run_saga(directory: String) -> Result<svalinn_core::SagaResult, String> {
    svalinn_core::run_saga(&directory)
}

#[tauri::command]
fn sval_list_qa_tree(directory: String) -> Result<Vec<svalinn_core::SvalFileTreeEntry>, String> {
    svalinn_core::list_qa_tree(&directory)
}

// ============================================================================
// Kvasir commands (kvas_ prefix)
// ============================================================================

#[tauri::command]
fn kvas_list_directory(directory: String, show_hidden: bool) -> Result<Vec<kvasir_core::KvasFileTreeEntry>, String> {
    kvasir_core::list_directory(&directory, show_hidden)
}

#[tauri::command]
fn kvas_read_file(path: String) -> Result<kvasir_core::FileContent, String> {
    kvasir_core::read_file(&path)
}

#[tauri::command]
fn kvas_open_in_editor(path: String, line: usize) -> Result<(), String> {
    kvasir_core::open_in_editor(&path, line)
}

#[tauri::command]
fn kvas_convert_to_all_formats(content: String, source_format: String) -> Result<kvasir_core::AllFormats, String> {
    kvasir_core::convert_to_all_formats(&content, &source_format)
}

#[tauri::command]
fn kvas_detect_data_format(path: String) -> Option<String> {
    kvasir_core::detect_data_format(&path)
}

#[tauri::command]
fn kvas_read_jsonl_info(path: String) -> Result<kvasir_core::JsonlInfo, String> {
    kvasir_core::read_jsonl_info(&path)
}

#[tauri::command]
fn kvas_read_jsonl_entry(path: String, index: usize) -> Result<kvasir_core::JsonlEntry, String> {
    kvasir_core::read_jsonl_entry(&path, index)
}

#[tauri::command]
fn kvas_export_entry_as(content: String, format: String, source_name: String, index: usize) -> Result<String, String> {
    kvasir_core::export_entry_as(&content, &format, &source_name, index)
}

#[tauri::command]
fn kvas_read_table(path: String) -> Result<kvasir_core::TableData, String> {
    kvasir_core::read_table(&path)
}

#[tauri::command]
fn kvas_export_table_csv(headers: Vec<String>, rows: Vec<Vec<String>>, source_path: String) -> Result<String, String> {
    kvasir_core::export_table_csv(headers, rows, &source_path)
}

// ============================================================================
// Ratatoskr commands (rata_ prefix)
// ============================================================================

#[tauri::command]
fn rata_list_directory(directory: String, show_hidden: bool) -> Result<Vec<ratatoskr_core::FileTreeEntry>, String> {
    ratatoskr_core::list_directory(&directory, show_hidden)
}

#[tauri::command]
fn rata_load_graph(path: String) -> Result<ratatoskr_core::GraphData, String> {
    ratatoskr_core::load_graph(&path)
}

#[tauri::command]
fn rata_save_graph(path: String, graph: ratatoskr_core::GraphData) -> Result<(), String> {
    ratatoskr_core::save_graph(&path, &graph)
}

#[tauri::command]
fn rata_get_graph_stats(graph: ratatoskr_core::GraphData) -> ratatoskr_core::GraphStats {
    ratatoskr_core::get_graph_stats(&graph)
}

#[tauri::command]
fn rata_generate_sample_graph() -> ratatoskr_core::GraphData {
    ratatoskr_core::generate_sample_graph()
}

// ============================================================================
// Shell-level commands (no prefix — not app-specific)
// ============================================================================

#[tauri::command]
fn get_pending_file(state: tauri::State<PendingFile>) -> Option<String> {
    state.0.lock().ok()?.take()
}

// ============================================================================
// App Entry
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(PendingFile(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            // Shell
            get_pending_file,
            // Hlidskjalf
            hlid_start_monitor,
            hlid_speak,
            hlid_open_in_editor,
            // Svalinn
            sval_scan_directory,
            sval_open_in_editor,
            sval_run_saga,
            sval_list_qa_tree,
            // Kvasir
            kvas_list_directory,
            kvas_read_file,
            kvas_open_in_editor,
            kvas_convert_to_all_formats,
            kvas_detect_data_format,
            kvas_read_jsonl_info,
            kvas_read_jsonl_entry,
            kvas_export_entry_as,
            kvas_read_table,
            kvas_export_table_csv,
            // Ratatoskr
            rata_list_directory,
            rata_load_graph,
            rata_save_graph,
            rata_get_graph_stats,
            rata_generate_sample_graph,
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
