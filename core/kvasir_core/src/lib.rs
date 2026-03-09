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

#[derive(Debug, Clone, Serialize)]
pub struct JsonlInfo {
    pub path: String,
    pub entry_count: usize,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonlEntry {
    pub index: usize,
    pub content: String,
    pub entry_count: usize,
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

    let extension = effective_extension(file_path);

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

pub use common_core::open_in_editor;

pub fn convert_to_all_formats(content: &str, source_format: &str) -> Result<AllFormats, String> {
    use format_core::{parse, serialize, convert::strip_nulls};

    let value: serde_json::Value = match source_format {
        "json" => parse::json(content).map_err(|e| e.to_string())?,
        "yaml" => parse::yaml(content).map_err(|e| e.to_string())?,
        "toml" => parse::toml(content).map_err(|e| e.to_string())?,
        "toon" => parse::toon(content).map_err(|e| e.to_string())?,
        _ => return Err(format!("Unsupported format: {}", source_format)),
    };

    let json_content = serialize::to_json(&value).map_err(|e| e.to_string())?;
    let yaml_content = serialize::to_yaml(&value).map_err(|e| e.to_string())?;

    // Strip nulls before TOML serialization — TOML cannot represent null
    let toml_safe = strip_nulls(value.clone());
    let toml_content = serialize::to_toml(&toml_safe)
        .unwrap_or_else(|e| format!("# TOML: {}", e));

    let toon_content = serialize::to_toon(&value)
        .unwrap_or_else(|e| format!("# TOON: {}", e));

    Ok(AllFormats {
        json: FormatConversion {
            token_count: estimate_token_count(&json_content),
            content: json_content,
        },
        yaml: FormatConversion {
            token_count: estimate_token_count(&yaml_content),
            content: yaml_content,
        },
        toml: FormatConversion {
            token_count: estimate_token_count(&toml_content),
            content: toml_content,
        },
        toon: FormatConversion {
            token_count: estimate_token_count(&toon_content),
            content: toon_content,
        },
        source_format: source_format.to_string(),
    })
}

pub fn detect_data_format(path: &str) -> Option<String> {
    let file_path = Path::new(path);
    let ext = effective_extension(file_path);
    match ext.as_str() {
        "json" | "jsonld" | "qa" | "meta" | "index" => Some("json".to_string()),
        "jsonl" => Some("jsonl".to_string()),
        "yaml" | "yml" => Some("yaml".to_string()),
        "toml" => Some("toml".to_string()),
        "toon" => Some("toon".to_string()),
        _ => None,
    }
}

pub fn read_jsonl_info(path: &str) -> Result<JsonlInfo, String> {
    let file_path = Path::new(path);
    if !file_path.is_file() {
        return Err(format!("Not a file: {}", path));
    }
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| format!("Failed to get metadata: {}", e))?;

    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open: {}", e))?;
    let reader = std::io::BufReader::new(file);

    use std::io::BufRead;
    let entry_count = reader.lines().count();

    Ok(JsonlInfo {
        path: path.to_string(),
        entry_count,
        size_bytes: metadata.len(),
    })
}

pub fn read_jsonl_entry(path: &str, index: usize) -> Result<JsonlEntry, String> {
    let file_path = Path::new(path);
    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open: {}", e))?;
    let reader = std::io::BufReader::new(file);

    use std::io::BufRead;
    let mut entry_count = 0;
    let mut target_line = None;

    for (i, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("Read error at line {}: {}", i, e))?;
        entry_count = i + 1;
        if i == index {
            target_line = Some(line);
        }
    }

    let raw = target_line
        .ok_or_else(|| format!("Index {} out of range (file has {} entries)", index, entry_count))?;

    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("Invalid JSON at line {}: {}", index, e))?;
    let content = serde_json::to_string_pretty(&value)
        .map_err(|e| format!("Serialization error: {}", e))?;

    Ok(JsonlEntry {
        index,
        content,
        entry_count,
    })
}

pub fn export_entry_as(
    content: &str,
    format: &str,
    source_name: &str,
    index: usize,
) -> Result<String, String> {
    let converted = if format == "json" {
        content.to_string()
    } else {
        let all = convert_to_all_formats(content, "json")?;
        match format {
            "yaml" => all.yaml.content,
            "toml" => all.toml.content,
            "toon" => all.toon.content,
            _ => return Err(format!("Unknown format: {}", format)),
        }
    };

    let filename = format!("kvasir-{}-{}.{}", source_name, index, format);
    let path = std::env::temp_dir().join(filename);

    std::fs::write(&path, &converted)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}

// ============================================================================
// Internal Helpers
// ============================================================================

fn effective_extension(path: &Path) -> String {
    let ext = path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if ext == "bak" {
        Path::new(path.file_stem().unwrap_or_default())
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default()
    } else {
        ext
    }
}

fn estimate_token_count(content: &str) -> usize {
    content.len() / 4
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
        "json" | "jsonld" | "qa" | "meta" | "index" | "jsonl" => "json",
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
