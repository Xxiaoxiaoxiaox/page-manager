use std::process::Command;

pub fn launch_exe(exe_path: &str) -> Result<(), String> {
    if exe_path.is_empty() {
        return Err("exe_path is empty".into());
    }
    Command::new(exe_path)
        .spawn()
        .map_err(|e| format!("Failed to launch {}: {}", exe_path, e))?;
    Ok(())
}
