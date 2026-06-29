pub mod store;

use serde::{Deserialize, Serialize};

/// A single snapshot record for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub timestamp: u64,
    pub cpu_percent: f32,
    pub mem_percent: f32,
    pub mem_used_kb: u64,
    pub mem_total_kb: u64,
    pub net_rx_speed: u64,
    pub net_tx_speed: u64,
    pub net_iface: String,
    pub disk_used_percent: f32,
    pub disk_mount: String,
    pub gpu_temp: f32,
    pub cpu_temp: f32,
}

impl Default for HistoryRecord {
    fn default() -> Self {
        Self {
            timestamp: 0,
            cpu_percent: 0.0,
            mem_percent: 0.0,
            mem_used_kb: 0,
            mem_total_kb: 0,
            net_rx_speed: 0,
            net_tx_speed: 0,
            net_iface: String::new(),
            disk_used_percent: 0.0,
            disk_mount: String::new(),
            gpu_temp: 0.0,
            cpu_temp: 0.0,
        }
    }
}

/// Time range for history queries
#[derive(Debug, Clone)]
pub enum TimeRange {
    Last5Min,
    Last30Min,
    Last1Hour,
    Last6Hours,
    Last24Hours,
    All,
}
