/// Open a file in the system editor (Zed via macOS `open -a`).
pub fn open_in_editor(path: &str, _line: usize) -> Result<(), String> {
    std::process::Command::new("open")
        .args(["-a", "Zed", path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
