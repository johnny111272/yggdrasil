use svalinn_core::{ScanResult, SvalFileTreeEntry, SagaResult};

#[tauri::command]
fn scan_directory(directory: String, include_tests: bool) -> Result<ScanResult, String> {
    svalinn_core::scan_directory(&directory, include_tests)
}

#[tauri::command]
fn open_in_editor(path: String, line: usize) -> Result<(), String> {
    svalinn_core::open_in_editor(&path, line)
}

#[tauri::command]
fn run_saga(directory: String) -> Result<SagaResult, String> {
    svalinn_core::run_saga(&directory)
}

#[tauri::command]
fn list_qa_tree(directory: String) -> Result<Vec<SvalFileTreeEntry>, String> {
    svalinn_core::list_qa_tree(&directory)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![scan_directory, open_in_editor, run_saga, list_qa_tree])
        .run(tauri::generate_context!())
        .expect("Svalinn failed to start");
}
