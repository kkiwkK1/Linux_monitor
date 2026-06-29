use super::CpuMetrics;
use std::fs;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

struct CpuState {
    prev_idle: u64,
    prev_total: u64,
    last_read: Option<Instant>,
}

static CPU_STATE: LazyLock<Mutex<CpuState>> = LazyLock::new(|| {
    Mutex::new(CpuState {
        prev_idle: 0,
        prev_total: 0,
        last_read: None,
    })
});

pub fn collect_cpu() -> CpuMetrics {
    let mut metrics = CpuMetrics::default();

    let content = match fs::read_to_string("/proc/stat") {
        Ok(c) => c,
        Err(_) => return metrics,
    };

    let mut lines = content.lines();

    // Line 1: aggregate CPU stats
    if let Some(line) = lines.next() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 && parts[0] == "cpu" {
            let user: u64 = parts[1].parse().unwrap_or(0);
            let nice: u64 = parts[2].parse().unwrap_or(0);
            let system: u64 = parts[3].parse().unwrap_or(0);
            let idle: u64 = parts[4].parse().unwrap_or(0);
            let iowait: u64 = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
            let irq: u64 = parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);
            let softirq: u64 = parts.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);
            let steal: u64 = parts.get(8).and_then(|s| s.parse().ok()).unwrap_or(0);

            let idle_total = idle + iowait;
            let total = user + nice + system + idle_total + irq + softirq + steal;

            let mut state = CPU_STATE.lock().unwrap();
            if let Some(ref last) = state.last_read {
                let elapsed = last.elapsed().as_secs_f32();
                if elapsed > 0.0 && state.prev_total > 0 {
                    let total_delta = total.saturating_sub(state.prev_total);
                    let idle_delta = idle_total.saturating_sub(state.prev_idle);
                    if total_delta > 0 {
                        metrics.usage_percent =
                            ((total_delta - idle_delta) as f32 / total_delta as f32) * 100.0;
                    }
                }
            }
            state.prev_idle = idle_total;
            state.prev_total = total;
            state.last_read = Some(Instant::now());
        }
    }

    // Per-core CPU usage
    for line in content.lines().skip(1) {
        if !line.starts_with("cpu") {
            break;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let user: u64 = parts[1].parse().unwrap_or(0);
            let nice: u64 = parts[2].parse().unwrap_or(0);
            let system: u64 = parts[3].parse().unwrap_or(0);
            let idle: u64 = parts[4].parse().unwrap_or(0);
            let iowait: u64 = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
            let total = user + nice + system + idle + iowait;
            let idle_total = idle + iowait;
            if total > 0 {
                let core_usage = ((total - idle_total) as f32 / total as f32) * 100.0;
                metrics.per_core_usage.push(core_usage);
            }
        }
    }
    metrics.core_count = metrics.per_core_usage.len();

    // Read CPU name
    if let Ok(info) = fs::read_to_string("/proc/cpuinfo") {
        for line in info.lines() {
            if line.starts_with("model name") {
                if let Some(name) = line.split(':').nth(1) {
                    metrics.name = name.trim().to_string();
                    break;
                }
            }
        }
    }

    // Read CPU frequency
    if let Ok(freq) =
        fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
    {
        metrics.frequency_mhz = freq.trim().parse::<u64>().unwrap_or(0) / 1000;
    }

    metrics
}
