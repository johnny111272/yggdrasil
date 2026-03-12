/// Open a file in the system editor (currently Zed via macOS `open -a`).
pub fn open_in_editor(path: &str, line: usize) -> Result<(), String> {
    let location = format!("{}:{}", path, line);
    std::process::Command::new("open")
        .args(["-a", "Zed", &location])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
