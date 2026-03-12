use serde::{Deserialize, Serialize};
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
    pub ron: FormatConversion,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub path: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
    pub column_count: usize,
    pub source_format: String,
    pub size_bytes: u64,
}

const MAX_TABLE_ROWS: usize = 100_000;

// ============================================================================
// Public Functions
// ============================================================================

pub fn list_directory(directory: &str, show_hidden: bool) -> Result<Vec<KvasFileTreeEntry>, String> {
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
        "ron" => ron::from_str(content).map_err(|e| e.to_string())?,
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

    let ron_content = ron::ser::to_string_pretty(&value, ron::ser::PrettyConfig::default())
        .unwrap_or_else(|e| format!("// RON: {}", e));

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
        ron: FormatConversion {
            token_count: estimate_token_count(&ron_content),
            content: ron_content,
        },
        source_format: source_format.to_string(),
    })
}

pub fn detect_data_format(path: &str) -> Option<String> {
    let file_path = Path::new(path);
    let ext = effective_extension(file_path);
    let format = match ext.as_str() {
        "json" | "jsonld" | "qa" | "meta" | "index" => "json",
        "jsonl" => "jsonl",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "toon" => "toon",
        "ron" => "ron",
        "csv" => "csv",
        "tsv" => "tsv",
        "parquet" => "parquet",
        _ => return None,
    };
    Some(format.into())
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
            "ron" => all.ron.content,
            _ => return Err(format!("Unknown format: {}", format)),
        }
    };

    let filename = format!("kvasir-{}-{}.{}", source_name, index, format);
    let path = std::env::temp_dir().join(filename);

    std::fs::write(&path, &converted)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}

pub fn read_table(path: &str) -> Result<TableData, String> {
    let file_path = Path::new(path);
    if !file_path.is_file() {
        return Err(format!("Not a file: {}", path));
    }

    let ext = effective_extension(file_path);
    match ext.as_str() {
        "csv" => read_csv_table(file_path, b','),
        "tsv" => read_csv_table(file_path, b'\t'),
        "parquet" => read_parquet_table(file_path),
        _ => Err(format!("Not a tabular format: {}", ext)),
    }
}

pub fn export_table_csv(
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    source_path: &str,
) -> Result<String, String> {
    let source = Path::new(source_path);
    let stem = source.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "table".into());

    let dir = source.parent().unwrap_or_else(|| Path::new("."));
    let output_path = dir.join(format!("{}_resorted.csv", stem));

    let file = std::fs::File::create(&output_path)
        .map_err(|e| format!("Failed to create {}: {}", output_path.display(), e))?;

    let mut writer = csv::Writer::from_writer(file);
    writer.write_record(&headers)
        .map_err(|e| format!("Failed to write headers: {}", e))?;
    for row in &rows {
        writer.write_record(row)
            .map_err(|e| format!("Failed to write row: {}", e))?;
    }
    writer.flush()
        .map_err(|e| format!("Failed to flush: {}", e))?;

    Ok(output_path.to_string_lossy().to_string())
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

fn read_csv_table(file_path: &Path, delimiter: u8) -> Result<TableData, String> {
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| format!("Failed to get metadata: {}", e))?;

    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open: {}", e))?;

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .from_reader(file);

    let headers: Vec<String> = reader.headers()
        .map_err(|e| format!("Failed to read headers: {}", e))?
        .iter()
        .map(|h| h.to_string())
        .collect();

    let column_count = headers.len();
    let mut rows = Vec::new();

    for result in reader.records() {
        if rows.len() >= MAX_TABLE_ROWS {
            break;
        }
        let record = result.map_err(|e| format!("CSV parse error at row {}: {}", rows.len() + 1, e))?;
        let row: Vec<String> = record.iter().map(|f| f.to_string()).collect();
        rows.push(row);
    }

    let row_count = rows.len();
    let source_format = if delimiter == b'\t' { "tsv" } else { "csv" };

    Ok(TableData {
        path: file_path.to_string_lossy().to_string(),
        headers,
        rows,
        row_count,
        column_count,
        source_format: source_format.to_string(),
        size_bytes: metadata.len(),
    })
}

fn read_parquet_table(file_path: &Path) -> Result<TableData, String> {
    use arrow::array::Array;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

    let metadata = std::fs::metadata(file_path)
        .map_err(|e| format!("Failed to get metadata: {}", e))?;

    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open: {}", e))?;

    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| format!("Failed to read parquet: {}", e))?;

    let schema = builder.schema().clone();
    let headers: Vec<String> = schema.fields().iter().map(|f| f.name().clone()).collect();
    let column_count = headers.len();

    let reader = builder
        .with_batch_size(8192)
        .build()
        .map_err(|e| format!("Failed to build parquet reader: {}", e))?;

    let mut rows = Vec::new();

    for batch_result in reader {
        let batch = batch_result.map_err(|e| format!("Parquet read error: {}", e))?;

        for row_idx in 0..batch.num_rows() {
            if rows.len() >= MAX_TABLE_ROWS {
                break;
            }
            let mut row = Vec::with_capacity(column_count);
            for col_idx in 0..batch.num_columns() {
                let col = batch.column(col_idx);
                let value = if col.is_null(row_idx) {
                    String::new()
                } else {
                    arrow::util::display::array_value_to_string(col, row_idx)
                        .unwrap_or_default()
                };
                row.push(value);
            }
            rows.push(row);
        }

        if rows.len() >= MAX_TABLE_ROWS {
            break;
        }
    }

    let row_count = rows.len();

    Ok(TableData {
        path: file_path.to_string_lossy().to_string(),
        headers,
        rows,
        row_count,
        column_count,
        source_format: "parquet".into(),
        size_bytes: metadata.len(),
    })
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
        "ron" => "ron",
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
