use super::{GpuMetrics, GpuVendor};
use std::fs;
use std::path::Path;

pub fn collect_gpu() -> Vec<GpuMetrics> {
    let mut metrics = Vec::new();

    // Check for NVIDIA GPUs via /sys/module/nvidia
    if Path::new("/sys/module/nvidia").exists() || Path::new("/proc/driver/nvidia").exists() {
        if let Some(m) = collect_nvidia_gpu() {
            metrics.push(m);
        }
    }

    // Check for AMD GPUs via /sys/class/drm
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("card") && !name.contains('-') {
                let path = entry.path();
                if let Some(m) = collect_drm_gpu(&path, &name, GpuVendor::Amd) {
                    // Avoid duplicates if NVIDIA also shows up here
                    let is_duplicate = metrics.iter().any(|existing| {
                        existing.name.contains(&name) || m.name == existing.name
                    });
                    if !is_duplicate {
                        metrics.push(m);
                    }
                }
            }
        }
    }

    // Check for Intel integrated GPU
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("card") {
                let vendor_path = entry.path().join("device/vendor");
                if let Ok(vendor) = fs::read_to_string(&vendor_path) {
                    if vendor.trim() == "0x8086" {
                        // Already handled above or not;
                        // Only add if not already present
                        if !metrics.iter().any(|m| m.vendor == GpuVendor::Intel) {
                            if let Some(m) = collect_drm_gpu(&entry.path(), &name, GpuVendor::Intel) {
                                metrics.push(m);
                            }
                        }
                    }
                }
            }
        }
    }

    if metrics.is_empty() {
        // Unknown GPU - try basic DRM detection
        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("card") && !name.contains('-') {
                    if let Some(m) = collect_drm_gpu(&entry.path(), &name, GpuVendor::Unknown) {
                        metrics.push(m);
                        break;
                    }
                }
            }
        }
    }

    metrics
}

fn collect_nvidia_gpu() -> Option<GpuMetrics> {
    // Try to read NVIDIA GPU info from /proc/driver/nvidia/gpus
    let gpu_path = "/proc/driver/nvidia/gpus";
    if let Ok(entries) = fs::read_dir(gpu_path) {
        for entry in entries.flatten() {
            let info_path = entry.path().join("information");
            if let Ok(info) = fs::read_to_string(&info_path) {
                let mut metrics = GpuMetrics {
                    vendor: GpuVendor::Nvidia,
                    name: "NVIDIA GPU".to_string(),
                    ..Default::default()
                };

                for line in info.lines() {
                    if line.starts_with("Model:") {
                        metrics.name = line
                            .split(':')
                            .nth(1)
                            .unwrap_or("NVIDIA GPU")
                            .trim()
                            .to_string();
                    }
                }

                // Try NVML-style temperature from hwmon
                if let Some(temp) = read_hwmon_temp(&entry.path()) {
                    metrics.temperature_c = temp;
                }

                // Try to get memory info from the GPU sysfs
                let mem_path = entry.path().join("information");
                if let Ok(mem_info) = fs::read_to_string(&mem_path) {
                    for line in mem_info.lines() {
                        if line.contains("BAR1") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 4 {
                                metrics.memory_total_mb =
                                    parts[2].parse().unwrap_or(0) / 1024 / 1024;
                                metrics.memory_used_mb =
                                    parts[4].parse().unwrap_or(0) / 1024 / 1024;
                            }
                        }
                    }
                }

                return Some(metrics);
            }
        }
    }
    None
}

fn collect_drm_gpu(card_path: &Path, card_name: &str, vendor: GpuVendor) -> Option<GpuMetrics> {
    let mut metrics = GpuMetrics {
        vendor,
        name: format!("GPU ({})", card_name),
        ..Default::default()
    };

    // Try to get GPU name
    let device_path = card_path.join("device");
    if let Ok(vendor_str) = fs::read_to_string(device_path.join("vendor")) {
        let vendor_id = vendor_str.trim();
        if vendor == GpuVendor::Unknown {
            metrics.vendor = match vendor_id {
                "0x10de" => GpuVendor::Nvidia,
                "0x1002" => GpuVendor::Amd,
                "0x8086" => GpuVendor::Intel,
                _ => GpuVendor::Unknown,
            };
        }
    }

    // Try to read GPU usage from /sys/class/drm/cardX/device/gpu_busy_percent
    if let Ok(usage) = fs::read_to_string(device_path.join("gpu_busy_percent")) {
        metrics.usage_percent = usage.trim().parse().unwrap_or(0.0);
    }

    // Try to read memory info
    if let Ok(mem_total) = fs::read_to_string(device_path.join("mem_info_vram_total")) {
        let total_bytes: u64 = mem_total.trim().parse().unwrap_or(0);
        metrics.memory_total_mb = total_bytes / 1024 / 1024;
    }
    if let Ok(mem_used) = fs::read_to_string(device_path.join("mem_info_vram_used")) {
        let used_bytes: u64 = mem_used.trim().parse().unwrap_or(0);
        metrics.memory_used_mb = used_bytes / 1024 / 1024;
    }

    // Temperature from hwmon
    if let Some(temp) = read_hwmon_temp(card_path) {
        metrics.temperature_c = temp;
    }

    // Also try reading from device/hwmon
    if metrics.temperature_c == 0.0 {
        if let Some(temp) = read_hwmon_temp(&device_path) {
            metrics.temperature_c = temp;
        }
    }

    Some(metrics)
}

fn read_hwmon_temp(base_path: &Path) -> Option<f32> {
    let hwmon_path = base_path.join("hwmon");
    if let Ok(hwmon_dir) = fs::read_dir(&hwmon_path) {
        for hwmon in hwmon_dir.flatten() {
            let hwmon = hwmon.path();
            for i in 1..=5 {
                let temp_input = hwmon.join(format!("temp{}_input", i));
                if let Ok(temp_str) = fs::read_to_string(&temp_input) {
                    if let Ok(temp) = temp_str.trim().parse::<f32>() {
                        return Some(temp / 1000.0); // millidegrees to degrees
                    }
                }
            }
        }
    }
    None
}
