use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Poll intervals (milliseconds)
    #[serde(default)]
    pub poll: PollConfig,

    /// UI appearance settings
    #[serde(default)]
    pub appearance: AppearanceConfig,

    /// Which monitors are enabled
    #[serde(default)]
    pub monitors: MonitorConfig,

    /// Window position and state
    #[serde(default)]
    pub window: WindowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollConfig {
    #[serde(default = "default_active_interval")]
    pub active_interval_ms: u64,
    #[serde(default = "default_bg_interval")]
    pub background_interval_ms: u64,
    #[serde(default = "default_idle_interval")]
    pub idle_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    #[serde(default = "default_skin")]
    pub skin: String,
    #[serde(default = "default_true")]
    pub show_cpu: bool,
    #[serde(default = "default_true")]
    pub show_memory: bool,
    #[serde(default = "default_true")]
    pub show_network: bool,
    #[serde(default = "default_true")]
    pub show_disk: bool,
    #[serde(default = "default_true")]
    pub show_gpu: bool,
    #[serde(default = "default_true")]
    pub show_temperature: bool,
    #[serde(default)]
    pub font_size: u32,
    #[serde(default)]
    pub opacity: f64,
    /// Always on top
    #[serde(default = "default_true")]
    pub always_on_top: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    #[serde(default = "default_true")]
    pub cpu: bool,
    #[serde(default = "default_true")]
    pub memory: bool,
    #[serde(default = "default_true")]
    pub network: bool,
    #[serde(default = "default_true")]
    pub disk: bool,
    #[serde(default = "default_true")]
    pub gpu: bool,
    #[serde(default = "default_true")]
    pub thermal: bool,
    /// Network interfaces to show (empty = auto-detect)
    #[serde(default)]
    pub network_interfaces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default = "default_width")]
    pub width: i32,
    #[serde(default = "default_height")]
    pub height: i32,
    #[serde(default = "default_true")]
    pub visible: bool,
}

// Default value functions
fn default_active_interval() -> u64 { 1000 }
fn default_bg_interval() -> u64 { 5000 }
fn default_idle_interval() -> u64 { 15000 }
fn default_skin() -> String { "horizontal".to_string() }
fn default_true() -> bool { true }
fn default_width() -> i32 { 280 }
fn default_height() -> i32 { 120 }
fn default_font_size() -> u32 { 11 }

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            active_interval_ms: default_active_interval(),
            background_interval_ms: default_bg_interval(),
            idle_interval_ms: default_idle_interval(),
        }
    }
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            skin: default_skin(),
            show_cpu: true,
            show_memory: true,
            show_network: true,
            show_disk: true,
            show_gpu: true,
            show_temperature: true,
            font_size: default_font_size(),
            opacity: 0.92,
            always_on_top: true,
        }
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            cpu: true,
            memory: true,
            network: true,
            disk: true,
            gpu: true,
            thermal: true,
            network_interfaces: Vec::new(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: default_width(),
            height: default_height(),
            visible: true,
        }
    }
}

impl AppConfig {
    /// Load config from ~/.config/linux-monitor/config.toml
    pub fn load() -> Self {
        let config_path = Self::config_path();
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    match toml::from_str(&content) {
                        Ok(config) => {
                            log::info!("Loaded config from {:?}", config_path);
                            return config;
                        }
                        Err(e) => {
                            log::warn!("Failed to parse config: {}, using defaults", e);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to read config: {}, using defaults", e);
                }
            }
        }
        log::info!("Using default configuration");
        Self::default()
    }

    /// Save config to disk
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        // Set file permissions to 0600 (owner read/write only)
        fs::write(&config_path, &content)?;
        Self::set_secure_permissions(&config_path)?;
        log::info!("Saved config to {:?}", config_path);
        Ok(())
    }

    fn config_path() -> PathBuf {
        let base = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg)
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".config")
        };
        base.join("linux-monitor").join("config.toml")
    }

    /// Set file permissions to 0600 for security
    fn set_secure_permissions(path: &PathBuf) -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms)?;
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            poll: PollConfig::default(),
            appearance: AppearanceConfig::default(),
            monitors: MonitorConfig::default(),
            window: WindowConfig::default(),
        }
    }
}
