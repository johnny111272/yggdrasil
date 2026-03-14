use tauri::Emitter;

#[tauri::command]
async fn start_monitor(handle: tauri::AppHandle) -> Result<(), String> {
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    hlidskjalf_core::start_all(sender).await?;
    tokio::spawn(async move {
        while let Some(datagram) = receiver.recv().await {
            let _ = handle.emit("datagram", &datagram);
        }
    });
    Ok(())
}

#[tauri::command]
fn speak(text: String) {
    hlidskjalf_core::speak(&text);
}

#[tauri::command]
fn open_in_editor(path: String, line: usize) -> Result<(), String> {
    common_core::open_in_editor(&path, line)
}

#[tauri::command]
fn open_default(path: String) -> Result<(), String> {
    std::process::Command::new("open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_monitor, speak, open_in_editor, open_default])
        .run(tauri::generate_context!())
        .unwrap_or_else(|_| std::process::exit(1));
}
