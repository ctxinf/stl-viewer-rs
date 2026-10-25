use std::path::PathBuf;
pub const DEFAULT: [u8; 3] = [14, 17, 22];
fn path() -> Result<PathBuf, String> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
        .ok_or("Cannot locate configuration directory")?;
    Ok(base.join("stl-viewer-linux/background"))
}
pub fn load() -> [u8; 3] {
    path()
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| {
            s.split_whitespace()
                .map(str::parse::<u8>)
                .collect::<Result<Vec<_>, _>>()
                .ok()
        })
        .and_then(|v| v.try_into().ok())
        .unwrap_or(DEFAULT)
}
pub fn save(color: [u8; 3]) -> Result<(), String> {
    let path = path()?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let temp = path.with_extension("tmp");
    std::fs::write(&temp, format!("{} {} {}\n", color[0], color[1], color[2]))
        .map_err(|e| e.to_string())?;
    std::fs::rename(temp, path).map_err(|e| e.to_string())
}
