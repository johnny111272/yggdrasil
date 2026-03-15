use serde::Serialize;
use std::path::Path;

/// Open a file in the system editor (Zed via macOS `open -a`).
pub fn open_in_editor(path: &str, _line: usize) -> Result<(), String> {
    std::process::Command::new("open")
        .args(["-a", "Zed", path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================================
// File Tree
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct FileTreeEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub extension: Option<String>,
    pub size_bytes: u64,
}

pub fn list_directory(directory: &str, show_hidden: bool) -> Result<Vec<FileTreeEntry>, String> {
    let dir_path = Path::new(directory);
    if !dir_path.is_dir() {
        return Err(format!("Not a directory: {}", directory));
    }

    let mut entries: Vec<FileTreeEntry> = vec![];

    let read_dir = std::fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let is_dir = path.is_dir();
        let extension = if !is_dir {
            path.extension().map(|e| e.to_string_lossy().to_string())
        } else {
            None
        };

        let size_bytes = if !is_dir {
            std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        entries.push(FileTreeEntry {
            name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            extension,
            size_bytes,
        });
    }

    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}
