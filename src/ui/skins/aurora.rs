use super::draw::{self, Metrics, NET_FULL_SCALE};
use super::Skin;
use crate::config::AppearanceConfig;
use crate::locale::L;
use crate::monitor::SystemSnapshot;
use crate::ui::theme;
use gtk::cairo::LinearGradient;
use gtk::prelude::*;
use gtk::{Box as GtkBox, DrawingArea, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

const CHART_POINTS: usize = 120;

/// Aurora — metrics as stacked gradient fields, no chrome, no grid. The bold,
/// distinctive pick; still pure Cairo.
pub struct AuroraSkin {
    h_cpu: Rc<RefCell<Vec<f64>>>,
    h_mem: Rc<RefCell<Vec<f64>>>,
    h_net: Rc<RefCell<Vec<f64>>>,
    area: RefCell<Option<DrawingArea>>,
    legend: RefCell<Option<Label>>,
}

impl AuroraSkin {
    pub fn new() -> Self {
        Self {
            h_cpu: Rc::new(RefCell::new(Vec::with_capacity(CHART_POINTS))),
            h_mem: Rc::new(RefCell::new(Vec::with_capacity(CHART_POINTS))),
            h_net: Rc::new(RefCell::new(Vec::with_capacity(CHART_POINTS))),
            area: RefCell::new(None),
            legend: RefCell::new(None),
        }
    }
}

fn push(buf: &Rc<RefCell<Vec<f64>>>, v: f64) {
    let mut b = buf.borrow_mut();
    if b.is_empty() {
        // Pre-seed the whole window so the ribbon starts full, not empty-left.
        b.resize(CHART_POINTS, v);
        return;
    }
    b.push(v);
    if b.len() > CHART_POINTS {
        b.remove(0);
    }
}

fn area_fill(cr: &gtk::cairo::Context, data: &[f64], color: theme::Color, x0: f64, sw: f64, h: f64) {
    if data.len() < 2 {
        return;
    }
    let (r, g, b) = color.rgb_f();
    let grad = LinearGradient::new(0.0, 0.0, 0.0, h);
    grad.add_color_stop_rgba(0.0, r, g, b, 0.5);
    grad.add_color_stop_rgba(1.0, r, g, b, 0.02);
    cr.move_to(x0, h);
    for (i, v) in data.iter().enumerate() {
        cr.line_to(x0 + i as f64 * sw, h - (v / 100.0 * h).clamp(0.0, h));
    }
    cr.line_to(x0 + (data.len() - 1) as f64 * sw, h);
    cr.close_path();
    let _ = cr.set_source(&grad);
    let _ = cr.fill();
    // top edge
    cr.set_source_rgba(r, g, b, 0.9);
    cr.set_line_width(1.4);
    cr.set_line_join(gtk::cairo::LineJoin::Round);
    for (i, v) in data.iter().enumerate() {
        let x = x0 + i as f64 * sw;
        let y = h - (v / 100.0 * h).clamp(0.0, h);
        if i == 0 {
            cr.move_to(x, y);
        } else {
            cr.line_to(x, y);
        }
    }
    let _ = cr.stroke();
}

impl Skin for AuroraSkin {
    fn name(&self) -> &str {
        "aurora"
    }
    fn display_name(&self) -> &str {
        L.skin_aurora()
    }

    fn create_widget(&self, _config: &AppearanceConfig) -> gtk::Widget {
        let outer = GtkBox::new(Orientation::Vertical, 5);
        outer.style_context().add_class("skin-aurora");
        outer.set_margin_start(8);
        outer.set_margin_end(8);
        outer.set_margin_top(8);
        outer.set_margin_bottom(6);

        let area = DrawingArea::new();
        area.set_size_request(-1, 44);
        let hc = self.h_cpu.clone();
        let hm = self.h_mem.clone();
        let hn = self.h_net.clone();
        area.connect_draw(move |a, cr| {
            let w = a.allocation().width() as f64;
            let h = a.allocation().height() as f64;
            if w < 10.0 {
                return gtk::glib::Propagation::Proceed;
            }
            let cpu = hc.borrow();
            let mem = hm.borrow();
            let net = hn.borrow();
            let sw = w / CHART_POINTS as f64;
            let x0 = w * (1.0 - cpu.len() as f64 / CHART_POINTS as f64);
            area_fill(cr, &cpu, theme::CPU, x0, sw, h);
            area_fill(cr, &mem, theme::MEM, x0, sw, h);
            // net as a bare line
            if net.len() >= 2 {
                draw::set_color(cr, theme::NET_RX, 0.85);
                cr.set_line_width(1.2);
                for (i, v) in net.iter().enumerate() {
                    let x = x0 + i as f64 * sw;
                    let y = h - (v / 100.0 * h).clamp(0.0, h);
                    if i == 0 {
                        cr.move_to(x, y);
                    } else {
                        cr.line_to(x, y);
                    }
                }
                let _ = cr.stroke();
            }
            gtk::glib::Propagation::Proceed
        });
        outer.add(&area);
        *self.area.borrow_mut() = Some(area);

        let legend = Label::new(None);
        legend.set_halign(gtk::Align::Start);
        outer.add(&legend);
        *self.legend.borrow_mut() = Some(legend);

        outer.upcast::<gtk::Widget>()
    }

    fn update(&self, snapshot: &SystemSnapshot, config: &AppearanceConfig) {
        let fs = config.font_size.max(9) as i32;
        let m = Metrics::from(snapshot);
        push(&self.h_cpu, m.cpu);
        push(&self.h_mem, m.mem);
        push(&self.h_net, (m.rx_b.max(m.tx_b) / NET_FULL_SCALE * 100.0).min(100.0));
        if let Some(ref a) = *self.area.borrow() {
            a.queue_draw();
        }
        if let Some(ref l) = *self.legend.borrow() {
            l.set_markup(&format!(
                "<span font_desc='{}' foreground='{}'>● cpu <span foreground='{}'>{:.0}%</span>   <span foreground='{}'>● </span>mem <span foreground='{}'>{:.0}%</span>   <span foreground='{}'>● net</span></span>",
                fs - 2,
                theme::CPU.hex(),
                theme::TEXT.hex(),
                m.cpu,
                theme::MEM.hex(),
                theme::TEXT.hex(),
                m.mem,
                theme::NET_RX.hex()
            ));
        }
    }
}
