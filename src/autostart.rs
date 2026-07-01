//! XDG autostart: toggles `~/.config/autostart/linux-monitor.desktop`, the
//! desktop-standard "start on login" mechanism (GNOME/KDE/XFCE all honor it).
//! State is derived from the file's existence — no separate config flag.

use std::fs;
use std::io;
use std::path::PathBuf;

fn autostart_dir() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|_| {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())).join(".config")
    });
    base.join("autostart")
}

pub fn desktop_path() -> PathBuf {
    autostart_dir().join("linux-monitor.desktop")
}

/// Whether start-on-login is currently enabled.
pub fn is_enabled() -> bool {
    desktop_path().exists()
}

/// Enable or disable start-on-login by writing/removing the autostart entry.
/// The `Exec` points at the current executable so it survives a moved binary.
pub fn set_enabled(on: bool) -> io::Result<()> {
    let path = desktop_path();
    if on {
        let exe = std::env::current_exe()?;
        fs::create_dir_all(autostart_dir())?;
        // Reference the exported icon PNG if it exists (absolute path works
        // regardless of icon-theme installation).
        let icon_path = crate::ui::icon::ensure_png();
        let icon_line = match icon_path {
            Some(p) => format!("Icon={}\n", p.display()),
            None => String::new(),
        };
        let content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=LinuxMonitor\n\
             Comment=Hardware performance monitor\n\
             Exec={}\n\
             {}\
             Terminal=false\n\
             X-GNOME-Autostart-enabled=true\n",
            exe.display(),
            icon_line
        );
        fs::write(&path, content)?;
    } else if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_creates_and_removes_entry() {
        let tmp = std::env::temp_dir().join(format!("lm-autostart-{}", std::process::id()));
        std::env::set_var("XDG_CONFIG_HOME", &tmp);
        std::env::set_var("XDG_DATA_HOME", tmp.join("data"));

        assert!(!is_enabled());
        set_enabled(true).unwrap();
        assert!(is_enabled(), "entry should exist after enabling");

        let content = fs::read_to_string(desktop_path()).unwrap();
        assert!(content.contains("[Desktop Entry]"));
        assert!(content.contains("Exec="));
        assert!(content.contains("X-GNOME-Autostart-enabled=true"));

        set_enabled(false).unwrap();
        assert!(!is_enabled(), "entry should be gone after disabling");

        let _ = fs::remove_dir_all(&tmp);
    }
}
