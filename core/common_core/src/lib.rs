/// Open a file at a specific line in VS Code.
pub fn open_in_editor(path: &str, line: usize) -> Result<(), String> {
    std::process::Command::new("code")
        .args(["--goto", &format!("{}:{}", path, line)])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
