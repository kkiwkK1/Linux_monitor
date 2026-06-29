use super::Skin;
use super::horizontal::{fmt_bytes, fmt_speed};
use crate::config::AppearanceConfig;
use crate::locale::L;
use crate::monitor::SystemSnapshot;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation};
use std::cell::RefCell;

pub struct VerticalSkin {
    labels: RefCell<Vec<Label>>,
    container: RefCell<Option<GtkBox>>,
}

impl VerticalSkin {
    pub fn new() -> Self { Self { labels: RefCell::new(Vec::new()), container: RefCell::new(None) } }
}

impl Skin for VerticalSkin {
    fn name(&self) -> &str { "vertical" }
    fn display_name(&self) -> &str { L.skin_vertical() }

    fn create_widget(&self, config: &AppearanceConfig) -> gtk::Widget {
        let container = GtkBox::new(Orientation::Vertical, 3);
        container.style_context().add_class("skin-vertical");
        container.set_margin_start(10); container.set_margin_end(10);
        container.set_margin_top(6); container.set_margin_bottom(6);
        let mut labels = Vec::new();
        for _ in 0..5 {
            let l = Label::new(None);
            l.set_halign(gtk::Align::Start);
            l.set_xalign(0.0);
            container.add(&l);
            labels.push(l);
        }
        *self.labels.borrow_mut() = labels;
        *self.container.borrow_mut() = Some(container.clone());
        container.upcast::<gtk::Widget>()
    }

    fn update(&self, snapshot: &SystemSnapshot, config: &AppearanceConfig) {
        let fs = config.font_size.max(9);
        let labels = self.labels.borrow();
        if labels.len() < 5 { return; }

        // CPU
        let cpu_pct = snapshot.cpu.usage_percent as f64;
        labels[0].set_markup(&format!(
            "<span font_desc='{}' foreground='#66ccff'><b>🖥 CPU</b>  {}</span>  <span font_desc='{}' foreground='#5599cc'>{}MHz  {}c</span>",
            fs, bar(cpu_pct, 12), fs - 1, snapshot.cpu.frequency_mhz, snapshot.cpu.core_count));

        // MEM
        let mem_pct = snapshot.memory.usage_percent as f64;
        labels[1].set_markup(&format!(
            "<span font_desc='{}' foreground='#99ee99'><b>🧠 MEM</b>  {}</span>  <span font_desc='{}' foreground='#66aa66'>{}/{}</span>",
            fs, bar(mem_pct, 12), fs - 1, fmt_bytes(snapshot.memory.used_kb*1024), fmt_bytes(snapshot.memory.total_kb*1024)));

        // NET
        let net = snapshot.network.first();
        let rx = net.map(|n| fmt_speed(n.rx_speed_bytes)).unwrap_or("--".into());
        let tx = net.map(|n| fmt_speed(n.tx_speed_bytes)).unwrap_or("--".into());
        let iface = net.map(|n| n.interface.clone()).unwrap_or_default();
        let net_max = net.map(|n| n.rx_speed_bytes.max(n.tx_speed_bytes) as f64).unwrap_or(0.0);
        let net_pct = (net_max / 10_485_760.0 * 100.0).min(100.0);
        labels[2].set_markup(&format!(
            "<span font_desc='{}' foreground='#ffcc66'><b>🌐 NET</b>  {}</span>  <span font_desc='{}' foreground='#bb9933'>↓{} ↑{}  {}</span>",
            fs, bar(net_pct, 12), fs - 1, rx, tx, iface));

        // DISK
        if let Some(disk) = snapshot.disk.first() {
            labels[3].set_markup(&format!(
                "<span font_desc='{}' foreground='#cc99ff'><b>💾 DSK</b>  {}</span>  <span font_desc='{}' foreground='#9977bb'>{}/{}  {}</span>",
                fs, bar(disk.usage_percent as f64, 12), fs - 1, fmt_bytes(disk.used_kb*1024), fmt_bytes(disk.total_kb*1024), disk.mount_point));
        }

        // GPU
        if let Some(gpu) = snapshot.gpu.first() {
            let gpu_pct = gpu.usage_percent as f64;
            labels[4].set_markup(&format!(
                "<span font_desc='{}' foreground='#ff9966'><b>🎮 GPU</b>  {}</span>  <span font_desc='{}' foreground='#cc6633'>{:3.0}°C  {}/{}</span>",
                fs, bar(gpu_pct, 12), fs - 1, gpu.temperature_c,
                fmt_bytes(gpu.memory_used_mb*1024*1024), fmt_bytes(gpu.memory_total_mb*1024*1024)));
        }
    }
}

fn bar(pct: f64, n: usize) -> String {
    let filled = ((pct / 100.0 * n as f64).round() as usize).min(n);
    let empty = n - filled;
    format!("{}{} {:3.0}%", "█".repeat(filled), "░".repeat(empty), pct)
}
