use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::io::Write;

/// Try to become the primary instance.
/// Returns Ok(()) if this is the primary, Err(()) if another instance exists.
/// If another instance exists, sends a "show" message to it.
pub fn try_become_primary() -> Result<(), ()> {
    let socket_path = socket_path();

    // Try to connect to existing instance
    if socket_path.exists() {
        if let Ok(mut stream) = UnixStream::connect(&socket_path) {
            // Send "show" command to existing instance
            let _ = stream.write_all(b"show");
            log::info!("Sent show command to existing instance");
            return Err(());
        }
        // Socket exists but can't connect - stale, clean up
        let _ = std::fs::remove_file(&socket_path);
    }

    Ok(())
}

/// Start listening for commands from other instances
pub fn start_listener() -> std::os::unix::net::UnixListener {
    let socket_path = socket_path();

    // Remove stale socket if exists
    let _ = std::fs::remove_file(&socket_path);

    let listener = std::os::unix::net::UnixListener::bind(&socket_path)
        .expect("Failed to bind instance socket");

    // Set permissions
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600));

    log::info!("Primary instance listening on {:?}", socket_path);
    listener
}

/// Clean up the socket file
pub fn cleanup() {
    let path = socket_path();
    let _ = std::fs::remove_file(&path);
}

fn socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("linux-monitor.sock")
}
