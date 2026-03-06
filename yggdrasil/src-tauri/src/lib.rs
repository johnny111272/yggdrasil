use tauri::Emitter;

// ============================================================================
// Hlidskjalf commands (hlid_ prefix)
// ============================================================================

#[tauri::command]
async fn hlid_start_listener(app: tauri::AppHandle) -> Result<(), String> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    hlidskjalf_core::start_listener(tx).await?;
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = app.emit("hook-event", &event);
        }
    });
    Ok(())
}

#[tauri::command]
fn hlid_speak(text: String) {
    hlidskjalf_core::speak(&text);
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
fn sval_list_directory(directory: String) -> Result<Vec<svalinn_core::SvalFileTreeEntry>, String> {
    svalinn_core::list_directory(&directory)
}

// ============================================================================
// Kvasir commands (kvas_ prefix)
// ============================================================================

#[tauri::command]
fn kvas_list_directory(directory: String) -> Result<Vec<kvasir_core::KvasFileTreeEntry>, String> {
    kvasir_core::list_directory(&directory)
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
fn kvas_convert_all_formats(content: String, source_format: String) -> Result<kvasir_core::AllFormats, String> {
    kvasir_core::convert_all_formats(&content, &source_format)
}

#[tauri::command]
fn kvas_is_data_file(path: String) -> Option<String> {
    kvasir_core::is_data_file(&path)
}

// ============================================================================
// Ratatoskr commands (rata_ prefix)
// ============================================================================

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
// App Entry
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Hlidskjalf
            hlid_start_listener,
            hlid_speak,
            // Svalinn
            sval_scan_directory,
            sval_open_in_editor,
            sval_run_saga,
            sval_list_directory,
            // Kvasir
            kvas_list_directory,
            kvas_read_file,
            kvas_open_in_editor,
            kvas_convert_all_formats,
            kvas_is_data_file,
            // Ratatoskr
            rata_load_graph,
            rata_save_graph,
            rata_get_graph_stats,
            rata_generate_sample_graph,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
