//! Plugin system: Rhai scripting engine with sandboxed execution.
//!
//! Plugins are `.rhai` scripts in `~/.config/linux-monitor/plugins/`.
//! Each script runs in a sandbox: no filesystem, no network, no external commands.

use crate::monitor::SystemSnapshot;
use rhai::{Engine, Scope, AST};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Manages plugin lifecycle
pub struct PluginManager {
    engine: Mutex<Engine>,
    plugins: Mutex<HashMap<String, PluginInfo>>,
    plugin_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub enabled: bool,
    pub source: String,
    pub ast: Option<AST>,
}

/// Alert callback: (threshold, current_value, message)
pub type AlertCallback = Arc<dyn Fn(&str, f64, f64, &str) + Send + Sync>;

impl PluginManager {
    /// Create a new plugin manager with sandboxed engine
    pub fn new() -> anyhow::Result<Self> {
        let engine = Self::build_engine();

        let plugin_dir = Self::plugin_dir();
        fs::create_dir_all(&plugin_dir).ok();

        Ok(Self {
            engine: Mutex::new(engine),
            plugins: Mutex::new(HashMap::new()),
            plugin_dir,
        })
    }

    /// Build a sandboxed Rhai engine
    fn build_engine() -> Engine {
        let mut engine = Engine::new_raw();
        engine.set_max_operations(500_000);
        engine.set_max_modules(5);
        engine.set_max_call_levels(10);

        // Register API functions
        engine.register_fn("log", |msg: &str| {
            log::info!("[plugin] {}", msg);
        });
        engine.register_fn("format_bytes", |bytes: i64| -> String {
            if bytes >= 1_073_741_824 { format!("{:.1} GB", bytes as f64 / 1_073_741_824.0) }
            else if bytes >= 1_048_576 { format!("{:.1} MB", bytes as f64 / 1_048_576.0) }
            else if bytes >= 1_024 { format!("{:.1} KB", bytes as f64 / 1_024.0) }
            else { format!("{} B", bytes) }
        });

        engine
    }

    /// Scan plugin directory and load all `.rhai` scripts
    pub fn scan(&self) -> usize {
        let engine = self.engine.lock().unwrap();
        let mut plugins = self.plugins.lock().unwrap();
        plugins.clear();

        if let Ok(entries) = fs::read_dir(&self.plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "rhai") {
                    if let Ok(source) = fs::read_to_string(&path) {
                        let name = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let ast = engine.compile(&source).ok();
                        plugins.insert(name.clone(), PluginInfo { name, enabled: true, source, ast });
                    }
                }
            }
        }
        plugins.len()
    }

    /// Execute all enabled plugins with current metrics
    pub fn run(&self, snapshot: &SystemSnapshot, alerts: &AlertCallback) {
        let engine = self.engine.lock().unwrap();
        let plugins = self.plugins.lock().unwrap();
        let mut scope = Scope::new();

        scope.push("cpu_percent", snapshot.cpu.usage_percent as f64);
        scope.push("mem_percent", snapshot.memory.usage_percent as f64);
        scope.push("net_rx", snapshot.network.first().map(|n| n.rx_speed_bytes as f64).unwrap_or(0.0));
        scope.push("net_tx", snapshot.network.first().map(|n| n.tx_speed_bytes as f64).unwrap_or(0.0));
        scope.push("cpu_temp", snapshot.thermal.iter()
            .find(|t| t.sensor_type.contains("x86") || t.sensor_type.contains("cpu"))
            .map(|t| t.temperature_c as f64).unwrap_or(0.0));
        scope.push("gpu_temp", snapshot.gpu.first().map(|g| g.temperature_c as f64).unwrap_or(0.0));
        scope.push("core_count", snapshot.cpu.core_count as f64);

        let alerts_clone = alerts.clone();
        scope.push_constant("ALERT", alerts_clone);

        for (_, info) in plugins.iter() {
            if !info.enabled || info.ast.is_none() { continue; }
            if let Err(e) = engine.run_ast_with_scope(&mut scope, info.ast.as_ref().unwrap()) {
                log::warn!("Plugin '{}' error: {}", info.name, e);
            }
        }
    }

    /// Get list of loaded plugins
    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins.lock().unwrap().values().cloned().collect()
    }

    /// Toggle a plugin on/off
    pub fn toggle(&self, name: &str) -> bool {
        let mut plugins = self.plugins.lock().unwrap();
        if let Some(info) = plugins.get_mut(name) {
            info.enabled = !info.enabled;
            return info.enabled;
        }
        false
    }

    fn plugin_dir() -> PathBuf {
        let base = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
                    .join(".config")
            });
        base.join("linux-monitor").join("plugins")
    }
}

/// Create a default alert handler that logs and shows notifications
pub fn make_alert_handler() -> AlertCallback {
    Arc::new(|_type: &str, threshold: f64, value: f64, msg: &str| {
        log::warn!("[alert] {}: {} >= {} ({})", _type, value, threshold, msg);
        // TODO: show desktop notification via D-Bus notify
    })
}
