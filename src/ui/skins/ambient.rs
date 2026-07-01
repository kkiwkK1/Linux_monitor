use super::draw::Metrics;
use super::horizontal::fmt_speed;
use super::Skin;
use crate::config::AppearanceConfig;
use crate::locale::L;
use crate::monitor::SystemSnapshot;
use crate::ui::theme;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation};
use std::cell::RefCell;

/// Ambient Big-Number — CPU load as a large hero figure, other metrics as a
/// quiet satellite row. Calmest and most glanceable.
pub struct AmbientSkin {
    hero: RefCell<Option<Label>>,
    sat: RefCell<Option<Label>>,
}

impl AmbientSkin {
    pub fn new() -> Self {
        Self {
            hero: RefCell::new(None),
            sat: RefCell::new(None),
        }
    }
}

impl Skin for AmbientSkin {
    fn name(&self) -> &str {
        "ambient"
    }
    fn display_name(&self) -> &str {
        L.skin_ambient()
    }

    fn create_widget(&self, config: &AppearanceConfig) -> gtk::Widget {
        let fs = config.font_size.max(9) as i32;
        let outer = GtkBox::new(Orientation::Vertical, 2);
        outer.style_context().add_class("skin-ambient");
        outer.set_margin_start(16);
        outer.set_margin_end(16);
        outer.set_margin_top(12);
        outer.set_margin_bottom(12);

        let cap = Label::new(None);
        cap.set_xalign(0.0);
        cap.set_markup(&format!(
            "<span font_desc='{}' foreground='{}'>CPU LOAD</span>",
            fs - 3,
            theme::TEXT_DIM.hex()
        ));
        outer.add(&cap);

        let hero = Label::new(None);
        hero.set_xalign(0.0);
        outer.add(&hero);
        *self.hero.borrow_mut() = Some(hero);

        let sat = Label::new(None);
        sat.set_xalign(0.0);
        sat.set_margin_top(6);
        outer.add(&sat);
        *self.sat.borrow_mut() = Some(sat);

        outer.upcast::<gtk::Widget>()
    }

    fn update(&self, snapshot: &SystemSnapshot, config: &AppearanceConfig) {
        let fs = config.font_size.max(9) as i32;
        let m = Metrics::from(snapshot);
        if let Some(ref l) = *self.hero.borrow() {
            l.set_markup(&format!(
                "<span font_desc='{}' foreground='{}'><b>{:.0}</b></span><span font_desc='{}' foreground='{}'>%</span>",
                fs * 3,
                theme::CPU.hex(),
                m.cpu,
                fs + 2,
                theme::TEXT_DIM.hex()
            ));
        }
        if let Some(ref l) = *self.sat.borrow() {
            l.set_markup(&format!(
                "<span font_desc='{}' foreground='{}'>MEM <span foreground='{}'><b>{:.0}%</b></span>   ↓{} <span foreground='{}'><b>{:.0}°C</b></span></span>",
                fs - 1,
                theme::TEXT_DIM.hex(),
                theme::TEXT.hex(),
                m.mem,
                fmt_speed(m.rx_b as u64),
                theme::temp_color(m.temp).hex(),
                m.temp
            ));
        }
    }
}
