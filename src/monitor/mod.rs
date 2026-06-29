pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod memory;
pub mod network;
pub mod thermal;

use serde::{Deserialize, Serialize};

/// Snapshot of all system metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub timestamp: u64,
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub network: Vec<NetworkMetrics>,
    pub disk: Vec<DiskMetrics>,
    pub gpu: Vec<GpuMetrics>,
    pub thermal: Vec<ThermalMetrics>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage_percent: f32,
    pub per_core_usage: Vec<f32>,
    pub frequency_mhz: u64,
    pub core_count: usize,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total_kb: u64,
    pub used_kb: u64,
    pub available_kb: u64,
    pub swap_total_kb: u64,
    pub swap_used_kb: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub interface: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_speed_bytes: u64, // bytes per second since last poll
    pub tx_speed_bytes: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub mount_point: String,
    pub device: String,
    pub total_kb: u64,
    pub used_kb: u64,
    pub available_kb: u64,
    pub usage_percent: f32,
    pub read_bytes: u64,
    pub write_bytes: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpuMetrics {
    pub name: String,
    pub usage_percent: f32,
    pub memory_total_mb: u64,
    pub memory_used_mb: u64,
    pub temperature_c: f32,
    pub vendor: GpuVendor,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub enum GpuVendor {
    #[default]
    Unknown,
    Nvidia,
    Amd,
    Intel,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThermalMetrics {
    pub name: String,
    pub temperature_c: f32,
    pub high_c: Option<f32>,
    pub crit_c: Option<f32>,
    pub sensor_type: String,
}

impl SystemSnapshot {
    pub fn collect() -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            cpu: cpu::collect_cpu(),
            memory: memory::collect_memory(),
            network: network::collect_network(),
            disk: disk::collect_disk(),
            gpu: gpu::collect_gpu(),
            thermal: thermal::collect_thermal(),
        }
    }
}
