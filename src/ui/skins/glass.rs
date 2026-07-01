use super::draw::Metrics;
use super::horizontal::fmt_speed;
use super::Skin;
use crate::config::AppearanceConfig;
use crate::locale::L;
use crate::monitor::SystemSnapshot;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation};
use std::cell::RefCell;

/// Frosted Glass — translucent card with vivid pastel accents. GTK3/X11 can't
/// sample the backdrop for a real blur, so this leans on translucency + a soft
/// gradient (see `.skin-glass` in style.css) rather than a true frosted effect.
pub struct GlassSkin {
    cpu: RefCell<Option<Label>>,
    mem: RefCell<Option<Label>>,
    net: RefCell<Option<Label>>,
    temp: RefCell<Option<Label>>,
    rx: RefCell<Option<Label>>,
    tx: RefCell<Option<Label>>,
}

// Bright pastels that stay legible over the colorful translucent card.
const C_CPU: &str = "#bcd6ff";
const C_MEM: &str = "#a9f0d5";
const C_NET: &str = "#ffe0a8";
const C_TMP: &str = "#ffd0b0";
const C_VAL: &str = "#ffffff";

impl GlassSkin {
    pub fn new() -> Self {
        Self {
            cpu: RefCell::new(None),
            mem: RefCell::new(None),
            net: RefCell::new(None),
            temp: RefCell::new(None),
            rx: RefCell::new(None),
            tx: RefCell::new(None),
        }
    }
}

impl Skin for GlassSkin {
    fn name(&self) -> &str {
        "glass"
    }
    fn display_name(&self) -> &str {
        L.skin_glass()
    }
    fn self_backed(&self) -> bool {
        true
    }

    fn create_widget(&self, config: &AppearanceConfig) -> gtk::Widget {
        let fs = config.font_size.max(9) as i32;
        // No margins: the panel is the whole card (padding comes from
        // `.skin-glass` CSS so its background fills to the rounded edge).
        let outer = GtkBox::new(Orientation::Vertical, 6);
        outer.style_context().add_class("skin-glass");

        let labels = GtkBox::new(Orientation::Horizontal, 14);
        labels.set_halign(gtk::Align::Center);
        macro_rules! two_line {
            ($field:ident, $head:expr, $col:expr) => {{
                let l = Label::new(None);
                l.set_justify(gtk::Justification::Center);
                l.set_markup(&format!(
                    "<span font_desc='{}' foreground='{}'>{}</span>\n<span font_desc='{}' foreground='{}'><b>--</b></span>",
                    fs - 2, $col, $head, fs, C_VAL
                ));
                labels.pack_start(&l, false, false, 0);
                *self.$field.borrow_mut() = Some(l);
            }};
        }
        two_line!(cpu, "CPU", C_CPU);
        two_line!(mem, "MEM", C_MEM);
        two_line!(net, "NET", C_NET);
        two_line!(temp, "TMP", C_TMP);
        outer.add(&labels);

        let pills = GtkBox::new(Orientation::Horizontal, 6);
        pills.set_halign(gtk::Align::Center);
        for field in ["rx", "tx"] {
            let p = Label::new(None);
            p.style_context().add_class("glass-pill");
            match field {
                "rx" => *self.rx.borrow_mut() = Some(p.clone()),
                _ => *self.tx.borrow_mut() = Some(p.clone()),
            }
            pills.pack_start(&p, false, false, 0);
        }
        outer.add(&pills);

        outer.upcast::<gtk::Widget>()
    }

    fn update(&self, snapshot: &SystemSnapshot, config: &AppearanceConfig) {
        let fs = config.font_size.max(9) as i32;
        let m = Metrics::from(snapshot);
        let set = |cell: &RefCell<Option<Label>>, head: &str, col: &str, val: String| {
            if let Some(ref l) = *cell.borrow() {
                l.set_markup(&format!(
                    "<span font_desc='{}' foreground='{}'>{}</span>\n<span font_desc='{}' foreground='{}'><b>{}</b></span>",
                    fs - 2, col, head, fs, C_VAL, val
                ));
            }
        };
        set(&self.cpu, "CPU", C_CPU, format!("{:.0}%", m.cpu));
        set(&self.mem, "MEM", C_MEM, format!("{:.0}%", m.mem));
        set(&self.net, "NET", C_NET, fmt_speed(m.rx_b.max(m.tx_b) as u64));
        set(&self.temp, "TMP", C_TMP, format!("{:.0}°", m.temp));

        if let Some(ref l) = *self.rx.borrow() {
            l.set_markup(&format!("<span font_desc='{}' foreground='{}'>↓ {}</span>", fs - 2, C_VAL, fmt_speed(m.rx_b as u64)));
        }
        if let Some(ref l) = *self.tx.borrow() {
            l.set_markup(&format!("<span font_desc='{}' foreground='{}'>↑ {}</span>", fs - 2, C_VAL, fmt_speed(m.tx_b as u64)));
        }
    }
}
