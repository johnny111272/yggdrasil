use tauri::Emitter;

#[tauri::command]
async fn start_listener(app: tauri::AppHandle) -> Result<(), String> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    hlidskjalf_core::start_all(tx).await?;
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = app.emit("hook-event", &event);
        }
    });
    Ok(())
}

#[tauri::command]
fn speak(text: String) {
    hlidskjalf_core::speak(&text);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_listener, speak])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
