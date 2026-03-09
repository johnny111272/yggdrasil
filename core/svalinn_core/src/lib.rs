use glob::glob;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub tool: String,
    pub code: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub column: Option<usize>,
    pub message: String,
    #[serde(default)]
    pub signal: Option<String>,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub canary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub directory: String,
    pub files_scanned: usize,
    pub issues: Vec<Issue>,
    pub by_tool: HashMap<String, usize>,
    pub by_severity: HashMap<String, usize>,
    pub by_file: HashMap<String, usize>,
    pub by_code: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SagaResult {
    pub success: bool,
    pub files_analyzed: usize,
    pub total_issues: usize,
    pub output: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SvalFileTreeEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub has_sidecar: bool,
    pub issue_count: usize,
}

// ============================================================================
// Sidecar Reading
// ============================================================================

#[derive(Debug, Deserialize)]
struct QaSidecarReport {
    file: String,
    #[allow(dead_code)]
    relative_path: Option<String>,
    #[allow(dead_code)]
    content_hash: Option<String>,
    #[allow(dead_code)]
    generated_at: Option<String>,
    issues: Vec<SidecarIssue>,
}

#[derive(Debug, Deserialize)]
struct SidecarIssue {
    tool: String,
    code: String,
    severity: String,
    line: usize,
    column: Option<usize>,
    message: String,
    #[serde(default)]
    signal: Option<String>,
    #[serde(default)]
    direction: Option<String>,
    #[serde(default)]
    canary: Option<String>,
}

fn find_sidecars(directory: &str, include_tests: bool) -> Result<Vec<PathBuf>, String> {
    let pattern = format!("{}/**/.*.qa", directory);
    Ok(glob(&pattern)
        .map_err(|e| format!("Invalid glob pattern: {}", e))?
        .filter_map(Result::ok)
        .filter(|path| {
            if include_tests {
                true
            } else {
                !path.components().any(|c| c.as_os_str() == "tests")
            }
        })
        .collect())
}

fn read_sidecar(path: &Path) -> Result<Vec<Issue>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let report: QaSidecarReport = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;

    Ok(report
        .issues
        .into_iter()
        .map(|i| Issue {
            tool: i.tool,
            code: i.code,
            severity: i.severity,
            file: report.file.clone(),
            line: i.line,
            column: i.column,
            message: i.message,
            signal: i.signal,
            direction: i.direction,
            canary: i.canary,
        })
        .collect())
}

// ============================================================================
// Public Functions
// ============================================================================

pub fn scan_directory(directory: &str, include_tests: bool) -> Result<ScanResult, String> {
    let sidecars = find_sidecars(directory, include_tests)?;

    let mut all_issues: Vec<Issue> = vec![];
    let mut files_scanned = 0;

    for sidecar in sidecars {
        match read_sidecar(&sidecar) {
            Ok(issues) => {
                files_scanned += 1;
                all_issues.extend(issues);
            }
            Err(e) => eprintln!("Warning: {}", e),
        }
    }

    let mut by_tool: HashMap<String, usize> = HashMap::new();
    let mut by_severity: HashMap<String, usize> = HashMap::new();
    let mut by_file: HashMap<String, usize> = HashMap::new();
    let mut by_code: HashMap<String, usize> = HashMap::new();

    for issue in &all_issues {
        *by_tool.entry(issue.tool.clone()).or_insert(0) += 1;
        *by_severity.entry(issue.severity.clone()).or_insert(0) += 1;
        *by_file.entry(issue.file.clone()).or_insert(0) += 1;
        *by_code.entry(issue.code.clone()).or_insert(0) += 1;
    }

    Ok(ScanResult {
        directory: directory.to_string(),
        files_scanned,
        issues: all_issues,
        by_tool,
        by_severity,
        by_file,
        by_code,
    })
}

pub use common_core::open_in_editor;

pub fn run_saga(directory: &str) -> Result<SagaResult, String> {
    let saga_path = dirs::home_dir()
        .map(|h| h.join(".ai/phoenix/quality/saga/.venv/bin/saga"))
        .ok_or("Could not determine home directory")?;

    if !saga_path.exists() {
        return Err(format!("Saga not found at {}", saga_path.display()));
    }

    let home = dirs::home_dir().map(|h| h.display().to_string()).unwrap_or_default();
    let path = std::env::var("PATH").unwrap_or_default();
    let extended_path = format!(
        "{}/.local/bin:/opt/homebrew/bin:/usr/local/bin:{}",
        home, path
    );

    let output = std::process::Command::new(&saga_path)
        .arg("--force")
        .arg(directory)
        .env("PATH", &extended_path)
        .output()
        .map_err(|e| format!("Failed to run saga: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = if stderr.is_empty() {
        stdout.clone()
    } else {
        format!("{}\n{}", stdout, stderr)
    };

    let (files_analyzed, total_issues) = parse_saga_output(&stdout);

    Ok(SagaResult {
        success: output.status.success(),
        files_analyzed,
        total_issues,
        output: combined,
    })
}

fn parse_saga_output(output: &str) -> (usize, usize) {
    for line in output.lines() {
        if line.starts_with("Analyzed") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let files = parts[1].parse().unwrap_or(0);
                let issues = parts[4].parse().unwrap_or(0);
                return (files, issues);
            }
        }
    }
    (0, 0)
}

pub fn list_qa_tree(directory: &str) -> Result<Vec<SvalFileTreeEntry>, String> {
    let dir_path = Path::new(directory);
    if !dir_path.is_dir() {
        return Err(format!("Not a directory: {}", directory));
    }

    let mut entries: Vec<SvalFileTreeEntry> = vec![];

    let read_dir = std::fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') {
            continue;
        }

        let is_dir = path.is_dir();

        let (has_sidecar, issue_count) = if !is_dir && name.ends_with(".py") {
            let sidecar_path = path.parent().unwrap().join(format!(".{}.qa", name));
            if sidecar_path.exists() {
                let count = read_sidecar(&sidecar_path)
                    .map(|issues| issues.len())
                    .unwrap_or(0);
                (true, count)
            } else {
                (false, 0)
            }
        } else if is_dir {
            let pattern = format!("{}/**/.*.qa", path.display());
            let count: usize = glob(&pattern)
                .map(|paths| {
                    paths
                        .filter_map(Result::ok)
                        .filter_map(|p| read_sidecar(&p).ok())
                        .map(|issues| issues.len())
                        .sum()
                })
                .unwrap_or(0);
            (false, count)
        } else {
            (false, 0)
        };

        entries.push(SvalFileTreeEntry {
            name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            has_sidecar,
            issue_count,
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
