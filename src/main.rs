#![allow(dead_code)]

mod autostart;
mod config;
mod engine;
mod history;
mod instance;
mod locale;
mod monitor;
mod plugin;
mod ui;

use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Force X11 backend — GTK3's set_keep_above and begin_move_drag
    // require X11/XWayland; native Wayland doesn't support them.
    std::env::set_var("GDK_BACKEND", "x11");

    // Run in the background by default; `--foreground`/`-f` keeps it attached
    // to the terminal (useful for debugging — logs then stay on stderr).
    let foreground = std::env::args().any(|a| a == "--foreground" || a == "-f");

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    let _ = &*locale::L;
    log::info!("{} starting...", locale::L.app_name());

    // Check for an existing instance before forking: a secondary launch just
    // pings the primary to show itself and exits, no daemon needed.
    if instance::try_become_primary().is_err() {
        log::info!("Another instance is already running. Exiting.");
        return;
    }

    // Detach from the controlling terminal so closing it doesn't kill us.
    if !foreground {
        unsafe { daemonize() };
    }

    let listener = Rc::new(RefCell::new(Some(instance::start_listener())));

    let mut app = ui::app::TrafficMonitorApp::new(listener.clone());
    app.run();

    instance::cleanup();
    log::info!("{} exiting.", locale::L.app_name());
}

/// Classic double-fork daemonize: detach from the controlling terminal and
/// redirect stdio to /dev/null. Called before GTK init (no threads yet).
unsafe fn daemonize() {
    // First fork: parent returns the shell prompt.
    match libc::fork() {
        -1 => return, // fork failed — carry on in the foreground
        0 => {}
        _ => std::process::exit(0),
    }
    // New session so we're not attached to the old controlling terminal.
    libc::setsid();
    // Second fork: ensure we can never reacquire a controlling terminal.
    match libc::fork() {
        -1 => {}
        0 => {}
        _ => std::process::exit(0),
    }
    // Point stdio at /dev/null (the terminal may be gone).
    let devnull = libc::open(b"/dev/null\0".as_ptr() as *const libc::c_char, libc::O_RDWR);
    if devnull >= 0 {
        libc::dup2(devnull, 0);
        libc::dup2(devnull, 1);
        libc::dup2(devnull, 2);
        if devnull > 2 {
            libc::close(devnull);
        }
    }
}