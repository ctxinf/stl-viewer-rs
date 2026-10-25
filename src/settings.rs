use std::path::PathBuf;
pub const DEFAULT: [u8; 3] = [14, 17, 22];
pub const DEFAULT_MODEL: [u8; 3] = [82, 173, 224];
fn path(name: &str) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .ok_or("Cannot locate APPDATA directory")?;
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(|p| PathBuf::from(p).join("Library/Application Support"))
        .ok_or("Cannot locate home directory")?;
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
        .ok_or("Cannot locate configuration directory")?;
    Ok(base.join("stl-viewer-linux").join(name))
}
pub fn load(name: &str, default: [u8; 3]) -> [u8; 3] {
    path(name)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| {
            s.split_whitespace()
                .map(str::parse::<u8>)
                .collect::<Result<Vec<_>, _>>()
                .ok()
        })
        .and_then(|v| v.try_into().ok())
        .unwrap_or(default)
}
pub fn save(name: &str, color: [u8; 3]) -> Result<(), String> {
    let path = path(name)?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let temp = path.with_extension("tmp");
    std::fs::write(&temp, format!("{} {} {}\n", color[0], color[1], color[2]))
        .map_err(|e| e.to_string())?;
    // Windows rename does not replace an existing destination.
    #[cfg(target_os = "windows")]
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    std::fs::rename(temp, path).map_err(|e| e.to_string())
}
