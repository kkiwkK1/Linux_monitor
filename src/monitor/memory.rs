use super::MemoryMetrics;
use std::fs;

pub fn collect_memory() -> MemoryMetrics {
    let mut metrics = MemoryMetrics::default();

    let content = match fs::read_to_string("/proc/meminfo") {
        Ok(c) => c,
        Err(_) => return metrics,
    };

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let value: u64 = parts[1].parse().unwrap_or(0);

        match parts[0] {
            "MemTotal:" => metrics.total_kb = value,
            "MemAvailable:" => metrics.available_kb = value,
            "SwapTotal:" => metrics.swap_total_kb = value,
            "SwapFree:" => {
                metrics.swap_used_kb = metrics.swap_total_kb.saturating_sub(value);
            }
            _ => {}
        }
    }

    // Calculate used and usage percent
    if metrics.total_kb > 0 && metrics.available_kb > 0 {
        metrics.used_kb = metrics.total_kb - metrics.available_kb;
        metrics.usage_percent =
            (metrics.used_kb as f32 / metrics.total_kb as f32) * 100.0;
    }

    metrics
}
