use serde::Serialize;
use std::path::Path;

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct KvasFileTreeEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub extension: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub language: String,
    pub line_count: usize,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FormatConversion {
    pub content: String,
    pub token_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AllFormats {
    pub json: FormatConversion,
    pub yaml: FormatConversion,
    pub toml: FormatConversion,
    pub toon: FormatConversion,
    pub source_format: String,
}

// ============================================================================
// Public Functions
// ============================================================================

pub fn list_directory(directory: &str) -> Result<Vec<KvasFileTreeEntry>, String> {
    let dir_path = Path::new(directory);
    if !dir_path.is_dir() {
        return Err(format!("Not a directory: {}", directory));
    }

    let mut entries: Vec<KvasFileTreeEntry> = vec![];

    let read_dir = std::fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') {
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

        entries.push(KvasFileTreeEntry {
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

pub fn read_file(path: &str) -> Result<FileContent, String> {
    let file_path = Path::new(path);

    if !file_path.is_file() {
        return Err(format!("Not a file: {}", path));
    }

    let extension = file_path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let binary_extensions = ["png", "jpg", "jpeg", "gif", "ico", "pdf", "zip", "tar", "gz",
                            "exe", "dll", "so", "dylib", "pyc", "pyo", "wasm", "bin"];

    if binary_extensions.contains(&extension.as_str()) {
        return Err(format!("Binary file: {}", path));
    }

    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read {}: {}", path, e))?;

    let metadata = std::fs::metadata(file_path)
        .map_err(|e| format!("Failed to get metadata: {}", e))?;

    let language = detect_language(&extension);
    let line_count = content.lines().count();

    Ok(FileContent {
        path: path.to_string(),
        content,
        language,
        line_count,
        size_bytes: metadata.len(),
    })
}

pub use yggdrasil_shared::open_in_editor;

pub fn convert_all_formats(content: &str, source_format: &str) -> Result<AllFormats, String> {
    let value: serde_json::Value = match source_format {
        "json" => serde_json::from_str(content)
            .map_err(|e| format!("Invalid JSON: {}", e))?,
        "yaml" => serde_yaml::from_str(content)
            .map_err(|e| format!("Invalid YAML: {}", e))?,
        "toml" => {
            let toml_value: toml::Value = toml::from_str(content)
                .map_err(|e| format!("Invalid TOML: {}", e))?;
            serde_json::to_value(toml_value)
                .map_err(|e| format!("TOML conversion error: {}", e))?
        },
        "toon" => serde_toon2::from_str(content)
            .map_err(|e| format!("Invalid TOON: {}", e))?,
        _ => return Err(format!("Unsupported format: {}", source_format)),
    };

    let json_content = serde_json::to_string_pretty(&value)
        .map_err(|e| format!("JSON serialization error: {}", e))?;

    let yaml_content = serde_yaml::to_string(&value)
        .map_err(|e| format!("YAML serialization error: {}", e))?;

    let toml_content = match toml::to_string_pretty(&value) {
        Ok(s) => s,
        Err(_) => "# TOML conversion not supported for this structure".to_string(),
    };

    let toon_content = match serde_toon2::to_string(&value) {
        Ok(s) => s,
        Err(_) => "# TOON conversion not supported for this structure".to_string(),
    };

    Ok(AllFormats {
        json: FormatConversion {
            token_count: count_tokens(&json_content),
            content: json_content,
        },
        yaml: FormatConversion {
            token_count: count_tokens(&yaml_content),
            content: yaml_content,
        },
        toml: FormatConversion {
            token_count: count_tokens(&toml_content),
            content: toml_content,
        },
        toon: FormatConversion {
            token_count: count_tokens(&toon_content),
            content: toon_content,
        },
        source_format: source_format.to_string(),
    })
}

pub fn is_data_file(path: &str) -> Option<String> {
    detect_data_format(path)
}

// ============================================================================
// Internal Helpers
// ============================================================================

fn count_tokens(content: &str) -> usize {
    content.len() / 4
}

fn detect_data_format(path: &str) -> Option<String> {
    let path = Path::new(path);
    let ext = path.extension()?.to_string_lossy().to_lowercase();
    match ext.as_str() {
        "json" | "jsonld" | "qa" => Some("json".to_string()),
        "yaml" | "yml" => Some("yaml".to_string()),
        "toml" => Some("toml".to_string()),
        "toon" => Some("toon".to_string()),
        _ => None,
    }
}

fn detect_language(extension: &str) -> String {
    match extension {
        "py" => "python",
        "rs" => "rust",
        "js" => "javascript",
        "ts" => "typescript",
        "jsx" => "jsx",
        "tsx" => "tsx",
        "svelte" => "svelte",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" => "scss",
        "less" => "less",
        "json" | "jsonld" | "qa" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "md" | "markdown" => "markdown",
        "sql" => "sql",
        "sh" | "bash" | "zsh" => "bash",
        "c" => "c",
        "cpp" | "cc" | "cxx" => "cpp",
        "h" | "hpp" => "cpp",
        "go" => "go",
        "java" => "java",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "scala" => "scala",
        "r" => "r",
        "lua" => "lua",
        "vim" => "vim",
        "xml" => "xml",
        "ini" | "cfg" => "ini",
        "dockerfile" => "dockerfile",
        "makefile" => "makefile",
        "txt" => "plaintext",
        _ => "plaintext",
    }.to_string()
}
