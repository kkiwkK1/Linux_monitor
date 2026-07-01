use super::draw::{self, Metrics};
use super::horizontal::fmt_speed;
use super::Skin;
use crate::config::AppearanceConfig;
use crate::locale::L;
use crate::monitor::SystemSnapshot;
use crate::ui::theme;
use gtk::prelude::*;
use gtk::{Box as GtkBox, DrawingArea, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

/// Refined Flat — slim rounded bars replace the block-char bars, values
/// right-aligned, temperature as a colored readout.
pub struct RefinedSkin {
    bars: RefCell<Vec<DrawingArea>>,
    pcts: Vec<Rc<RefCell<f64>>>, // cpu, mem, disk
    vals: RefCell<Vec<Label>>,   // cpu, mem, disk
    net: RefCell<Option<Label>>,
    temp: RefCell<Option<Label>>,
}

impl RefinedSkin {
    pub fn new() -> Self {
        Self {
            bars: RefCell::new(Vec::new()),
            pcts: vec![
                Rc::new(RefCell::new(0.0)),
                Rc::new(RefCell::new(0.0)),
                Rc::new(RefCell::new(0.0)),
            ],
            vals: RefCell::new(Vec::new()),
            net: RefCell::new(None),
            temp: RefCell::new(None),
        }
    }
}

impl Skin for RefinedSkin {
    fn name(&self) -> &str {
        "refined"
    }
    fn display_name(&self) -> &str {
        L.skin_refined()
    }

    fn create_widget(&self, config: &AppearanceConfig) -> gtk::Widget {
        let fs = config.font_size.max(9) as i32;
        let outer = GtkBox::new(Orientation::Vertical, 6);
        outer.style_context().add_class("skin-refined");
        outer.set_margin_start(12);
        outer.set_margin_end(12);
        outer.set_margin_top(8);
        outer.set_margin_bottom(8);

        let rows: [(&str, theme::Color); 3] =
            [("CPU", theme::CPU), ("MEM", theme::MEM), ("DSK", theme::DISK)];
        // Size groups keep the key/value columns equal-width so every bar
        // starts and ends at the same x (labels use a proportional font).
        let key_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        let val_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        let mut bars = Vec::new();
        let mut vals = Vec::new();
        for (i, (key, color)) in rows.iter().enumerate() {
            let row = GtkBox::new(Orientation::Horizontal, 8);

            let k = Label::new(None);
            k.set_markup(&format!(
                "<span font_desc='{}' foreground='{}'><b>{}</b></span>",
                fs - 1,
                color.hex(),
                key
            ));
            k.set_xalign(0.0);
            key_group.add_widget(&k);
            row.pack_start(&k, false, false, 0);

            let bar = DrawingArea::new();
            bar.set_size_request(-1, 12);
            bar.set_valign(gtk::Align::Center);
            let pct = self.pcts[i].clone();
            let col = *color;
            bar.connect_draw(move |a, cr| {
                let w = a.allocation().width() as f64;
                let h = a.allocation().height() as f64;
                let bh = 6.0;
                let y = (h - bh) / 2.0;
                // track
                draw::rounded_rect(cr, 0.0, y, w, bh, bh / 2.0);
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.07);
                let _ = cr.fill();
                // fill
                let fw = (w * (*pct.borrow() / 100.0)).clamp(0.0, w);
                if fw > 1.0 {
                    draw::rounded_rect(cr, 0.0, y, fw, bh, bh / 2.0);
                    draw::set_color(cr, col, 1.0);
                    let _ = cr.fill();
                }
                gtk::glib::Propagation::Proceed
            });
            row.pack_start(&bar, true, true, 0);
            bars.push(bar);

            let v = Label::new(None);
            v.set_xalign(1.0);
            val_group.add_widget(&v);
            row.pack_start(&v, false, false, 0);
            vals.push(v);

            outer.add(&row);
        }
        *self.bars.borrow_mut() = bars;
        *self.vals.borrow_mut() = vals;

        // net + temp footer
        let foot = GtkBox::new(Orientation::Horizontal, 8);
        foot.set_margin_top(2);
        let net = Label::new(None);
        net.set_xalign(0.0);
        foot.pack_start(&net, true, true, 0);
        let temp = Label::new(None);
        temp.set_xalign(1.0);
        foot.pack_start(&temp, false, false, 0);
        outer.add(&foot);
        *self.net.borrow_mut() = Some(net);
        *self.temp.borrow_mut() = Some(temp);

        outer.upcast::<gtk::Widget>()
    }

    fn update(&self, snapshot: &SystemSnapshot, config: &AppearanceConfig) {
        let fs = config.font_size.max(9) as i32;
        let m = Metrics::from(snapshot);
        *self.pcts[0].borrow_mut() = m.cpu;
        *self.pcts[1].borrow_mut() = m.mem;
        *self.pcts[2].borrow_mut() = m.disk_pct;

        let vals = self.vals.borrow();
        let pcts = [m.cpu, m.mem, m.disk_pct];
        for (v, p) in vals.iter().zip(pcts.iter()) {
            v.set_markup(&format!(
                "<span font_desc='{}' foreground='{}'>{:>3.0}%</span>",
                fs - 1,
                theme::TEXT_DIM.hex(),
                p
            ));
        }
        for b in self.bars.borrow().iter() {
            b.queue_draw();
        }

        if let Some(ref l) = *self.net.borrow() {
            l.set_markup(&format!(
                "<span font_desc='{}' foreground='{}'>↓{} ↑{}</span>",
                fs - 1,
                theme::NET_RX.hex(),
                fmt_speed(m.rx_b as u64),
                fmt_speed(m.tx_b as u64)
            ));
        }
        if let Some(ref l) = *self.temp.borrow() {
            l.set_markup(&format!(
                "<span font_desc='{}' foreground='{}'><b>{:>3.0}°C</b></span>",
                fs,
                theme::temp_color(m.temp).hex(),
                m.temp
            ));
        }
    }
}
