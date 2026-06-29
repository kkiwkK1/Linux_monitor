#![allow(dead_code)]

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

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    let _ = &*locale::L;
    log::info!("{} starting...", locale::L.app_name());

    if instance::try_become_primary().is_err() {
        log::info!("Another instance is already running. Exiting.");
        return;
    }

    let listener = Rc::new(RefCell::new(Some(instance::start_listener())));

    let mut app = ui::app::TrafficMonitorApp::new(listener.clone());
    app.run();

    instance::cleanup();
    log::info!("{} exiting.", locale::L.app_name());
}