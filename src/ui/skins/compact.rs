use super::Skin;
use super::horizontal::fmt_speed;
use crate::config::AppearanceConfig;
use crate::locale::L;
use crate::monitor::SystemSnapshot;
use crate::ui::theme;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation};
use std::cell::RefCell;

/// Minimal single-line bar — sits unobtrusively, shows all essentials
pub struct CompactSkin {
    label: RefCell<Option<Label>>,
}

impl CompactSkin {
    pub fn new() -> Self { Self { label: RefCell::new(None) } }
}

impl Skin for CompactSkin {
    fn name(&self) -> &str { "compact" }
    fn display_name(&self) -> &str { L.skin_compact() }

    fn create_widget(&self, config: &AppearanceConfig) -> gtk::Widget {
        let outer = GtkBox::new(Orientation::Horizontal, 0);
        outer.style_context().add_class("skin-compact");
        outer.set_margin_start(10); outer.set_margin_end(10);
        outer.set_margin_top(5); outer.set_margin_bottom(5);
        outer.set_halign(gtk::Align::Center);
        let fs = config.font_size.max(8);

        let label = Label::new(None);
        label.set_markup(&format!(
            "<span font_desc='{}' foreground='{}'>CPU --%</span>  \
             <span font_desc='{}' foreground='{}'>MEM --%</span>  \
             <span font_desc='{}' foreground='{}'>↓-- ↑--</span>  \
             <span font_desc='{}' foreground='{}'>--°C</span>",
            fs, theme::CPU.hex(), fs, theme::MEM.hex(), fs, theme::NET_RX.hex(), fs, theme::TEMP.hex()));
        *self.label.borrow_mut() = Some(label.clone());
        outer.add(&label);
        outer.upcast::<gtk::Widget>()
    }

    fn update(&self, snapshot: &SystemSnapshot, config: &AppearanceConfig) {
        let fs = config.font_size.max(8);
        if let Some(ref l) = *self.label.borrow() {
            let rx = snapshot.network.first()
                .map(|n| fmt_speed(n.rx_speed_bytes)).unwrap_or("--".into());
            let tx = snapshot.network.first()
                .map(|n| fmt_speed(n.tx_speed_bytes)).unwrap_or("--".into());
            let t = snapshot.thermal.iter()
                .find(|t| t.sensor_type.contains("x86") || t.sensor_type.contains("cpu"))
                .or_else(|| snapshot.thermal.first())
                .map(|t| t.temperature_c).unwrap_or(0.0);
            let gpu_t = snapshot.gpu.first().map(|g| g.temperature_c).unwrap_or(t);

            l.set_markup(&format!(
                "<span font_desc='{}' foreground='{}'>CPU {:3.0}%</span>  \
                 <span font_desc='{}' foreground='{}'>MEM {:3.0}%</span>  \
                 <span font_desc='{}' foreground='{}'>↓{} ↑{}</span>  \
                 <span font_desc='{}' foreground='{}'>{:3.0}°C</span>",
                fs, theme::CPU.hex(), snapshot.cpu.usage_percent,
                fs, theme::MEM.hex(), snapshot.memory.usage_percent,
                fs, theme::NET_RX.hex(), rx, tx,
                fs, theme::temp_color(gpu_t as f64).hex(), gpu_t));
        }
    }
}
