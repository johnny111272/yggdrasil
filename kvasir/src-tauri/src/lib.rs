use kvasir_core::*;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![list_directory, read_file, open_in_editor, convert_to_all_formats, detect_data_format])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
