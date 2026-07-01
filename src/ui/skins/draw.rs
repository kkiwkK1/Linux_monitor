//! Shared helpers for the Cairo-drawn skins: a distilled [`Metrics`] view of a
//! snapshot plus small drawing primitives (color, rounded rect, centered text).

use crate::monitor::SystemSnapshot;
use crate::ui::theme::Color;
use gtk::cairo::{Context, FontSlant, FontWeight};

/// Network speed that maps to a full-scale (100%) bar/line, matching the
/// original skins' 10 MB/s reference.
pub const NET_FULL_SCALE: f64 = 10_485_760.0;

/// The metrics every skin actually renders, in draw-ready units.
pub struct Metrics {
    pub cpu: f64,
    pub mem: f64,
    pub rx_b: f64,
    pub tx_b: f64,
    pub net_pct: f64,
    pub disk_pct: f64,
    pub gpu: f64,
    pub temp: f64,
    pub freq_mhz: u64,
    pub cores: usize,
    pub mem_used_kb: u64,
    pub mem_total_kb: u64,
    pub iface: String,
    pub disk_used_kb: u64,
    pub disk_total_kb: u64,
    pub mount: String,
    pub gpu_mem_used_mb: u64,
    pub gpu_mem_total_mb: u64,
}

impl Metrics {
    pub fn from(s: &SystemSnapshot) -> Self {
        let net = s.network.first();
        let rx_b = net.map(|n| n.rx_speed_bytes as f64).unwrap_or(0.0);
        let tx_b = net.map(|n| n.tx_speed_bytes as f64).unwrap_or(0.0);
        let net_pct = (rx_b.max(tx_b) / NET_FULL_SCALE * 100.0).min(100.0);

        let cpu_temp = s
            .thermal
            .iter()
            .find(|t| t.sensor_type.contains("x86") || t.sensor_type.contains("cpu"))
            .or_else(|| s.thermal.first())
            .map(|t| t.temperature_c as f64);
        let gpu0 = s.gpu.first();
        let gpu_temp = gpu0.map(|g| g.temperature_c as f64);
        let temp = gpu_temp.or(cpu_temp).unwrap_or(0.0);

        let disk = s.disk.first();
        Self {
            cpu: s.cpu.usage_percent as f64,
            mem: s.memory.usage_percent as f64,
            rx_b,
            tx_b,
            net_pct,
            disk_pct: disk.map(|d| d.usage_percent as f64).unwrap_or(0.0),
            gpu: gpu0.map(|g| g.usage_percent as f64).unwrap_or(0.0),
            temp,
            freq_mhz: s.cpu.frequency_mhz,
            cores: s.cpu.core_count,
            mem_used_kb: s.memory.used_kb,
            mem_total_kb: s.memory.total_kb,
            iface: net.map(|n| n.interface.clone()).unwrap_or_default(),
            disk_used_kb: disk.map(|d| d.used_kb).unwrap_or(0),
            disk_total_kb: disk.map(|d| d.total_kb).unwrap_or(0),
            mount: disk.map(|d| d.mount_point.clone()).unwrap_or_default(),
            gpu_mem_used_mb: gpu0.map(|g| g.memory_used_mb).unwrap_or(0),
            gpu_mem_total_mb: gpu0.map(|g| g.memory_total_mb).unwrap_or(0),
        }
    }
}

/// Set the Cairo source to a theme color at the given alpha.
pub fn set_color(cr: &Context, c: Color, a: f64) {
    let (r, g, b) = c.rgb_f();
    cr.set_source_rgba(r, g, b, a);
}

/// Trace a rounded-rectangle path (does not fill/stroke).
pub fn rounded_rect(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let d = std::f64::consts::PI / 180.0;
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -90.0 * d, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, 90.0 * d);
    cr.arc(x + r, y + h - r, r, 90.0 * d, 180.0 * d);
    cr.arc(x + r, y + r, r, 180.0 * d, 270.0 * d);
    cr.close_path();
}

/// Draw monospace text. When `center` is true, `(x, y)` is the text's center.
pub fn text(cr: &Context, s: &str, x: f64, y: f64, size: f64, c: Color, a: f64, center: bool, bold: bool) {
    cr.select_font_face(
        "monospace",
        FontSlant::Normal,
        if bold { FontWeight::Bold } else { FontWeight::Normal },
    );
    cr.set_font_size(size);
    set_color(cr, c, a);
    let (tx, ty) = if center {
        match cr.text_extents(s) {
            Ok(e) => (x - e.width() / 2.0 - e.x_bearing(), y - e.height() / 2.0 - e.y_bearing()),
            Err(_) => (x, y),
        }
    } else {
        (x, y)
    };
    cr.move_to(tx, ty);
    let _ = cr.show_text(s);
}
