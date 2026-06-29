use super::NetworkMetrics;
use std::collections::HashMap;
use std::fs;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::Instant;

struct NetState {
    prev_rx: u64,
    prev_tx: u64,
    last_read: Instant,
}

static NET_STATES: LazyLock<Mutex<HashMap<String, NetState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn collect_network() -> Vec<NetworkMetrics> {
    let mut metrics = Vec::new();

    let content = match fs::read_to_string("/proc/net/dev") {
        Ok(c) => c,
        Err(_) => return metrics,
    };

    let now = Instant::now();
    let mut states = NET_STATES.lock().unwrap();

    for line in content.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }

        let iface = parts[0].trim_end_matches(':');
        if iface == "lo" {
            continue;
        }

        let rx_bytes: u64 = parts[1].parse().unwrap_or(0);
        let tx_bytes: u64 = parts[9].parse().unwrap_or(0);

        let (rx_speed, tx_speed) = if let Some(state) = states.get(iface) {
            let elapsed = now.duration_since(state.last_read).as_secs_f64();
            if elapsed > 0.0 {
                let rx_delta = rx_bytes.saturating_sub(state.prev_rx);
                let tx_delta = tx_bytes.saturating_sub(state.prev_tx);
                (
                    (rx_delta as f64 / elapsed) as u64,
                    (tx_delta as f64 / elapsed) as u64,
                )
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        };

        states.insert(
            iface.to_string(),
            NetState {
                prev_rx: rx_bytes,
                prev_tx: tx_bytes,
                last_read: now,
            },
        );

        metrics.push(NetworkMetrics {
            interface: iface.to_string(),
            rx_bytes,
            tx_bytes,
            rx_speed_bytes: rx_speed,
            tx_speed_bytes: tx_speed,
        });

        // One-time diagnostic
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            log::info!("Network interfaces detected: {} (iface={}, rx={}, tx={})",
                metrics.len(), iface, rx_bytes, tx_bytes);
        });
    }

    // Sort by total bytes (descending) so most active interface comes first
    metrics.sort_by(|a, b| (b.rx_bytes + b.tx_bytes).cmp(&(a.rx_bytes + a.tx_bytes)));

    metrics
}
