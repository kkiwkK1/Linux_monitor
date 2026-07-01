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
use std::f64::consts::PI;
use std::rc::Rc;

/// Ring Gauges — CPU / MEM / GPU as arc gauges with the value in the center.
pub struct RingsSkin {
    /// (cpu, mem, gpu) usage percentages.
    vals: Rc<RefCell<(f64, f64, f64)>>,
    area: RefCell<Option<DrawingArea>>,
    foot: RefCell<Option<Label>>,
}

impl RingsSkin {
    pub fn new() -> Self {
        Self {
            vals: Rc::new(RefCell::new((0.0, 0.0, 0.0))),
            area: RefCell::new(None),
            foot: RefCell::new(None),
        }
    }
}

impl Skin for RingsSkin {
    fn name(&self) -> &str {
        "rings"
    }
    fn display_name(&self) -> &str {
        L.skin_rings()
    }

    fn create_widget(&self, _config: &AppearanceConfig) -> gtk::Widget {
        let outer = GtkBox::new(Orientation::Vertical, 4);
        outer.style_context().add_class("skin-rings");
        outer.set_margin_start(8);
        outer.set_margin_end(8);
        outer.set_margin_top(8);
        outer.set_margin_bottom(6);

        let area = DrawingArea::new();
        area.set_size_request(240, 104);
        let vals = self.vals.clone();
        area.connect_draw(move |a, cr| {
            let w = a.allocation().width() as f64;
            let h = a.allocation().height() as f64;
            if w < 30.0 || h < 30.0 {
                return gtk::glib::Propagation::Proceed;
            }
            let (cpu, mem, gpu) = *vals.borrow();
            let rings = [
                ("CPU", cpu, theme::CPU),
                ("MEM", mem, theme::MEM),
                ("GPU", gpu, theme::GPU),
            ];
            // Reserve a strip at the bottom for labels; center each ring in the
            // remaining space and size the radius so it always fits.
            let label_h = 15.0;
            let cy = (h - label_h) / 2.0;
            let rad = ((w / 3.0) * 0.34).min(cy - 3.0).max(8.0);
            let lw = (rad * 0.22).max(4.0);
            let start = -PI / 2.0;
            cr.set_line_cap(gtk::cairo::LineCap::Butt);
            for (i, (label, pct, color)) in rings.iter().enumerate() {
                let cx = w * (i as f64 + 0.5) / 3.0;
                let p = (pct / 100.0).clamp(0.0, 1.0);
                cr.set_line_width(lw);
                // track (full faint ring). new_sub_path() breaks the current
                // point so arc() doesn't draw a connecting line from prior text.
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.10);
                cr.new_sub_path();
                cr.arc(cx, cy, rad, 0.0, 2.0 * PI);
                let _ = cr.stroke();
                // value arc — clockwise from 12 o'clock, proportional to pct
                if p > 0.0 {
                    draw::set_color(cr, *color, 1.0);
                    cr.new_sub_path();
                    cr.arc(cx, cy, rad, start, start + p * 2.0 * PI);
                    let _ = cr.stroke();
                }
                // center number + label below the ring
                draw::text(cr, &format!("{:.0}", pct), cx, cy, rad * 0.64, theme::TEXT, 1.0, true, true);
                draw::text(cr, label, cx, h - label_h / 2.0, 11.0, theme::TEXT_DIM, 1.0, true, true);
            }
            gtk::glib::Propagation::Proceed
        });
        outer.add(&area);
        *self.area.borrow_mut() = Some(area);

        let foot = Label::new(None);
        foot.set_halign(gtk::Align::Center);
        outer.add(&foot);
        *self.foot.borrow_mut() = Some(foot);

        outer.upcast::<gtk::Widget>()
    }

    fn update(&self, snapshot: &SystemSnapshot, config: &AppearanceConfig) {
        let fs = config.font_size.max(9) as i32;
        let m = Metrics::from(snapshot);
        *self.vals.borrow_mut() = (m.cpu, m.mem, m.gpu);
        if let Some(ref a) = *self.area.borrow() {
            a.queue_draw();
        }
        if let Some(ref l) = *self.foot.borrow() {
            l.set_markup(&format!(
                "<span font_desc='{}' foreground='{}'>↓{} ↑{}</span>  <span font_desc='{}' foreground='{}'><b>{:.0}°C</b></span>",
                fs - 1,
                theme::NET_RX.hex(),
                fmt_speed(m.rx_b as u64),
                fmt_speed(m.tx_b as u64),
                fs - 1,
                theme::temp_color(m.temp).hex(),
                m.temp
            ));
        }
    }
}
