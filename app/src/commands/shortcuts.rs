//! Desktop shortcut creation commands

use super::state::AppState;
use crate::core::instance::Instance;
use tauri::State;

/// Create a desktop shortcut for an instance
#[tauri::command]
pub async fn create_instance_shortcut(
    state: State<'_, AppState>,
    instance_id: String,
    location: String, // "desktop" or "start_menu"
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    
    #[cfg(target_os = "windows")]
    {
        create_windows_shortcut(&instance, &exe_path, &location)
    }
    
    #[cfg(target_os = "linux")]
    {
        create_linux_shortcut(&instance, &exe_path, &location)
    }
    
    #[cfg(target_os = "macos")]
    {
        let _ = (&instance, &exe_path, &location);
        Err("Shortcut creation on macOS is not yet implemented".to_string())
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = (&instance, &exe_path, &location);
        Err("Shortcut creation is not supported on this platform".to_string())
    }
}

#[cfg(target_os = "windows")]
fn create_windows_shortcut(
    instance: &Instance,
    exe_path: &std::path::Path,
    location: &str,
) -> Result<(), String> {
    use std::process::Command;
    
    let shortcut_dir = match location {
        "desktop" => dirs::desktop_dir().ok_or("Could not find desktop directory")?,
        "start_menu" => {
            let app_data = dirs::data_dir().ok_or("Could not find app data directory")?;
            app_data.join("Microsoft").join("Windows").join("Start Menu").join("Programs")
        }
        _ => return Err(format!("Unknown location: {}", location)),
    };
    
    let shortcut_path = shortcut_dir.join(format!("{}.lnk", instance.name));
    let args = format!("--launch {}", instance.id);
    
    let script = format!(
        r#"
        $WshShell = New-Object -comObject WScript.Shell
        $Shortcut = $WshShell.CreateShortcut("{}")
        $Shortcut.TargetPath = "{}"
        $Shortcut.Arguments = "{}"
        $Shortcut.WorkingDirectory = "{}"
        $Shortcut.Description = "Launch {} in OxideLauncher"
        $Shortcut.Save()
        "#,
        shortcut_path.to_string_lossy().replace("\\", "\\\\"),
        exe_path.to_string_lossy().replace("\\", "\\\\"),
        args,
        exe_path.parent().unwrap_or(exe_path).to_string_lossy().replace("\\", "\\\\"),
        instance.name
    );
    
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create shortcut: {}", stderr));
    }
    
    Ok(())
}

#[cfg(target_os = "linux")]
fn create_linux_shortcut(
    instance: &Instance,
    exe_path: &std::path::Path,
    location: &str,
) -> Result<(), String> {
    let shortcut_dir = match location {
        "desktop" => dirs::desktop_dir().ok_or("Could not find desktop directory")?,
        "start_menu" => {
            let data_dir = dirs::data_dir().ok_or("Could not find data directory")?;
            data_dir.join("applications")
        }
        _ => return Err(format!("Unknown location: {}", location)),
    };
    
    std::fs::create_dir_all(&shortcut_dir).map_err(|e| e.to_string())?;
    
    let desktop_file = shortcut_dir.join(format!("oxide-launcher-{}.desktop", instance.id));
    
    let content = format!(
        r#"[Desktop Entry]
Type=Application
Name={}
Comment=Launch {} in OxideLauncher
Exec="{}" --launch {}
Icon=minecraft
Terminal=false
Categories=Game;
"#,
        instance.name,
        instance.name,
        exe_path.to_string_lossy(),
        instance.id
    );
    
    std::fs::write(&desktop_file, content).map_err(|e| e.to_string())?;
    
    // Make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&desktop_file)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&desktop_file, perms).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}
