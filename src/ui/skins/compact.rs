use super::Skin;
use super::horizontal::fmt_speed;
use crate::config::AppearanceConfig;
use crate::locale::L;
use crate::monitor::SystemSnapshot;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation};
use std::cell::RefCell;

pub struct CompactSkin {
    cpu_label: RefCell<Option<Label>>,
    net_label: RefCell<Option<Label>>,
    mem_label: RefCell<Option<Label>>,
    temp_label: RefCell<Option<Label>>,
}

impl CompactSkin {
    pub fn new() -> Self {
        Self { cpu_label: RefCell::new(None), net_label: RefCell::new(None),
               mem_label: RefCell::new(None), temp_label: RefCell::new(None) }
    }
}

impl Skin for CompactSkin {
    fn name(&self) -> &str { "compact" }
    fn display_name(&self) -> &str { L.skin_compact() }

    fn create_widget(&self, config: &AppearanceConfig) -> gtk::Widget {
        let outer = GtkBox::new(Orientation::Horizontal, 6);
        outer.style_context().add_class("skin-compact");
        outer.set_margin_start(6); outer.set_margin_end(6);
        outer.set_margin_top(6); outer.set_margin_bottom(6);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);

        let fs = config.font_size.max(8);

        let cpu = Label::new(None);
        cpu.set_markup(&format!("<span font_desc='{}' foreground='#66ccff'><b>--%</b></span>", fs+2));
        *self.cpu_label.borrow_mut() = Some(cpu.clone());

        let net = Label::new(None);
        net.set_markup(&format!("<span font_desc='{}' foreground='#ffcc66'>↓-- ↑--</span>", fs));
        *self.net_label.borrow_mut() = Some(net.clone());

        let mem = Label::new(None);
        mem.set_markup(&format!("<span font_desc='{}' foreground='#99ee99'>MEM --%</span>", fs));
        *self.mem_label.borrow_mut() = Some(mem.clone());

        let temp = Label::new(None);
        temp.set_markup(&format!("<span font_desc='{}' foreground='#ff9966'>--°C</span>", fs));
        *self.temp_label.borrow_mut() = Some(temp.clone());

        let col1 = GtkBox::new(Orientation::Vertical, 2);
        col1.add(&cpu); col1.add(&mem);
        let col2 = GtkBox::new(Orientation::Vertical, 2);
        col2.add(&net); col2.add(&temp);
        outer.add(&col1); outer.add(&col2);

        outer.upcast::<gtk::Widget>()
    }

    fn update(&self, snapshot: &SystemSnapshot, config: &AppearanceConfig) {
        let fs = config.font_size.max(8);

        if let Some(ref label) = *self.cpu_label.borrow() {
            label.set_markup(&format!("<span font_desc='{}' foreground='#66ccff'><b>{:3.0}%</b></span>", fs+2, snapshot.cpu.usage_percent));
        }
        if let Some(ref label) = *self.mem_label.borrow() {
            label.set_markup(&format!("<span font_desc='{}' foreground='#99ee99'>MEM {:3.0}%</span>", fs, snapshot.memory.usage_percent));
        }
        if let Some(ref label) = *self.net_label.borrow() {
            if let Some(net) = snapshot.network.first() {
                label.set_markup(&format!("<span font_desc='{}' foreground='#ffcc66'>↓{} ↑{}</span>", fs, fmt_speed(net.rx_speed_bytes), fmt_speed(net.tx_speed_bytes)));
            }
        }
        if let Some(ref label) = *self.temp_label.borrow() {
            let t = snapshot.thermal.iter().find(|t| t.sensor_type.contains("x86") || t.sensor_type.contains("cpu") || t.sensor_type.contains("pch"))
                .or_else(|| snapshot.thermal.first()).map(|t| t.temperature_c);
            if let Some(tv) = t {
                label.set_markup(&format!("<span font_desc='{}' foreground='#ff9966'>{:3.0}°C</span>", fs, tv));
            }
        }
    }
}
