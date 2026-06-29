use crate::config::AppConfig;
use crate::engine::{MonitorEngine, PollConfig, PollMode};
use crate::history::store::HistoryStore;
use crate::history::HistoryRecord;
use crate::ui::controller::AppController;
use crate::ui::floating_window::FloatingWindow;
use gtk::prelude::*;
use std::cell::RefCell;
use std::os::unix::io::AsRawFd;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TrafficMonitorApp {
    app: gtk::Application,
    config: AppConfig,
    listener: Rc<RefCell<Option<std::os::unix::net::UnixListener>>>,
}

impl TrafficMonitorApp {
    pub fn new(listener: Rc<RefCell<Option<std::os::unix::net::UnixListener>>>) -> Self {
        let config = AppConfig::load();
        let app = gtk::Application::builder()
            .application_id("com.linuxmonitor.app")
            .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
            .build();

        Self { app, config, listener }
    }

    pub fn application(&self) -> &gtk::Application { &self.app }

    pub fn run(&mut self) {
        let config = Rc::new(RefCell::new(self.config.clone()));

        let history_store = match HistoryStore::open() {
            Ok(store) => { log::info!("History store opened successfully"); Some(Arc::new(store)) }
            Err(e) => { log::warn!("History store unavailable: {}", e); None }
        };

        // Plugin system
        let plugin_manager = match crate::plugin::PluginManager::new() {
            Ok(pm) => {
                let count = pm.scan();
                log::info!("Plugin manager ready, {} plugins loaded", count);
                Some(Arc::new(pm))
            }
            Err(e) => { log::warn!("Plugin system unavailable: {}", e); None }
        };

        let floating_ref: Rc<RefCell<Option<Rc<FloatingWindow>>>> = Rc::new(RefCell::new(None));
        let history_for_menu = history_store.clone();
        let plugin_for_menu = plugin_manager.clone();

        let cfg = config.clone();
        let fr = floating_ref.clone();
        self.app.connect_activate(move |app| {
            let existing = fr.borrow();
            if existing.is_some() {
                if let Some(ref f) = *existing {
                    if f.is_visible() { f.hide(); } else { f.show(); }
                }
                return;
            }
            drop(existing);

            log::info!("{} starting...", crate::locale::L.app_name());

            let engine = Rc::new(RefCell::new(MonitorEngine::new(PollConfig {
                active_interval_ms: cfg.borrow().poll.active_interval_ms,
                background_interval_ms: cfg.borrow().poll.background_interval_ms,
                idle_interval_ms: cfg.borrow().poll.idle_interval_ms,
                mode: PollMode::Active,
            })));

            let floating = FloatingWindow::new(app, engine.clone(), cfg.clone());
            floating.start_polling();

            if let Some(ref store) = history_for_menu {
                let engine_for_history = engine.clone();
                let store_for_history = store.clone();
                let plugin_for_history = plugin_for_menu.clone();
                let alert_handler = crate::plugin::make_alert_handler();
                gtk::glib::timeout_add_local(std::time::Duration::from_secs(60), move || {
                    let snap = engine_for_history.borrow().poll_snapshot();
                    // Record history
                    let record = HistoryRecord {
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                        cpu_percent: snap.cpu.usage_percent,
                        mem_percent: snap.memory.usage_percent,
                        mem_used_kb: snap.memory.used_kb,
                        mem_total_kb: snap.memory.total_kb,
                        net_rx_speed: snap.network.first().map(|n| n.rx_speed_bytes).unwrap_or(0),
                        net_tx_speed: snap.network.first().map(|n| n.tx_speed_bytes).unwrap_or(0),
                        net_iface: snap.network.first().map(|n| n.interface.clone()).unwrap_or_default(),
                        disk_used_percent: snap.disk.first().map(|d| d.usage_percent).unwrap_or(0.0),
                        disk_mount: snap.disk.first().map(|d| d.mount_point.clone()).unwrap_or_default(),
                        gpu_temp: snap.gpu.first().map(|g| g.temperature_c).unwrap_or(0.0),
                        cpu_temp: snap.thermal.first().map(|t| t.temperature_c).unwrap_or(0.0),
                    };
                    if let Err(e) = store_for_history.insert(&record) {
                        log::error!("Failed to insert history record: {}", e);
                    }
                    let _ = store_for_history.enforce_row_limit(100_000);
                    // Run plugins
                    if let Some(ref pm) = plugin_for_history {
                        pm.run(&snap, &alert_handler);
                    }
                    gtk::glib::ControlFlow::Continue
                });
            }

            let floating_rc = Rc::new(floating);
            *fr.borrow_mut() = Some(floating_rc.clone());

            let history_for_ctrl = history_for_menu.clone();
            let _controller = AppController::new(app, &floating_rc, engine, cfg.clone(), history_for_ctrl);

            let app_quit = app.clone();
            let quit_action = gtk::gio::SimpleAction::new("quit", None);
            quit_action.connect_activate(move |_, _| app_quit.quit());
            app.add_action(&quit_action);
            app.set_accels_for_action("app.quit", &["<Control>q"]);
        });

        // Unix socket listener
        let listener_rc = self.listener.clone();
        let fr_show = floating_ref.clone();
        let fd = { let guard = listener_rc.borrow(); guard.as_ref().map(|l| l.as_raw_fd()) };
        if let Some(fd) = fd {
            gtk::glib::unix_fd_add_local(fd, gtk::glib::IOCondition::IN, move |_fd, _condition| {
                if let Some(ref listener) = *listener_rc.borrow() {
                    if let Ok((mut stream, _)) = listener.accept() {
                        use std::io::Read;
                        let mut buf = [0u8; 16];
                        if let Ok(n) = stream.read(&mut buf) {
                            if String::from_utf8_lossy(&buf[..n]).contains("show") {
                                if let Some(ref f) = *fr_show.borrow() { f.show(); }
                            }
                        }
                    }
                }
                gtk::glib::ControlFlow::Continue
            });
        }

        self.app.run();
    }
}
