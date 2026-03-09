use ratatoskr_core::{GraphData, GraphStats};

#[tauri::command]
fn load_graph(path: String) -> Result<GraphData, String> {
    ratatoskr_core::load_graph(&path)
}

#[tauri::command]
fn save_graph(path: String, graph: GraphData) -> Result<(), String> {
    ratatoskr_core::save_graph(&path, &graph)
}

#[tauri::command]
fn get_graph_stats(graph: GraphData) -> GraphStats {
    ratatoskr_core::get_graph_stats(&graph)
}

#[tauri::command]
fn generate_sample_graph() -> GraphData {
    ratatoskr_core::generate_sample_graph()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![load_graph, save_graph, get_graph_stats, generate_sample_graph])
        .run(tauri::generate_context!())
        .expect("Ratatoskr failed to start");
}
