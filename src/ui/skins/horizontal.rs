use super::Skin;
use crate::config::AppearanceConfig;
use crate::locale::L;
use crate::monitor::SystemSnapshot;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation, DrawingArea};
use std::cell::RefCell;
use std::rc::Rc;

const CHART_POINTS: usize = 120;

pub struct HorizontalSkin {
    cpu_label: RefCell<Option<Label>>,
    mem_label: RefCell<Option<Label>>,
    net_label: RefCell<Option<Label>>,
    temp_label: RefCell<Option<Label>>,
    container: RefCell<Option<GtkBox>>,
    chart: RefCell<Option<DrawingArea>>,
    h_cpu: Rc<RefCell<Vec<f64>>>,
    h_mem: Rc<RefCell<Vec<f64>>>,
    h_rx:  Rc<RefCell<Vec<f64>>>,
    h_tx:  Rc<RefCell<Vec<f64>>>,
    h_gpu: Rc<RefCell<Vec<f64>>>,
    tick:  RefCell<u32>,
}

impl HorizontalSkin {
    pub fn new() -> Self {
        Self {
            cpu_label: RefCell::new(None), mem_label: RefCell::new(None),
            net_label: RefCell::new(None), temp_label: RefCell::new(None),
            container: RefCell::new(None), chart: RefCell::new(None),
            h_cpu: Rc::new(RefCell::new(Vec::with_capacity(CHART_POINTS))),
            h_mem: Rc::new(RefCell::new(Vec::with_capacity(CHART_POINTS))),
            h_rx:  Rc::new(RefCell::new(Vec::with_capacity(CHART_POINTS))),
            h_tx:  Rc::new(RefCell::new(Vec::with_capacity(CHART_POINTS))),
            h_gpu: Rc::new(RefCell::new(Vec::with_capacity(CHART_POINTS))),
            tick: RefCell::new(0),
        }
    }
}

impl Skin for HorizontalSkin {
    fn name(&self) -> &str { "horizontal" }
    fn display_name(&self) -> &str { L.skin_horizontal() }

    fn create_widget(&self, config: &AppearanceConfig) -> gtk::Widget {
        let outer = GtkBox::new(Orientation::Vertical, 0);
        outer.style_context().add_class("skin-horizontal");

        // Top row: centered labels with fixed-width gaps
        let top = GtkBox::new(Orientation::Horizontal, 0);
        top.set_halign(gtk::Align::Center);
        top.set_margin_start(14); top.set_margin_end(14);
        top.set_margin_top(6); top.set_margin_bottom(4);
        let fs = config.font_size.max(8);

        macro_rules! mk_label {
            ($field:ident, $color:expr, $text:expr, $pad:expr) => {{
                let l = Label::new(None);
                l.set_markup(&format!("<span font_desc='{}' foreground='{}'>{}</span>", fs, $color, $text));
                l.set_width_chars($pad);
                l.set_xalign(0.5);
                *self.$field.borrow_mut() = Some(l.clone());
                top.pack_start(&l, false, false, 2);
            }};
        }
        // Fixed-width labels: CPU(6ch) spacer MEM(7ch) spacer NET(15ch) spacer GPU(9ch)
        mk_label!(cpu_label, "#66ccff", "  CPU\n  --%", 6);
        mk_label!(mem_label, "#99ee99", "  MEM\n  --%", 7);
        mk_label!(net_label, "#ffcc66", "  ↓-------\n  ↑-------", 15);
        mk_label!(temp_label, "#ff9966", " 🌡 --°C", 9);
        outer.add(&top);

        // Chart — taller, two sections
        let chart = DrawingArea::new();
        chart.set_size_request(-1, 72);
        chart.set_margin_start(6); chart.set_margin_end(6); chart.set_margin_bottom(4);
        *self.chart.borrow_mut() = Some(chart.clone());
        outer.add(&chart);

        let hc = self.h_cpu.clone(); let hm = self.h_mem.clone();
        let hr = self.h_rx.clone(); let ht = self.h_tx.clone(); let hg = self.h_gpu.clone();
        chart.connect_draw(move |area, cr| {
            let w = area.allocation().width() as f64;
            let h = area.allocation().height() as f64;
            if w < 10.0 || h < 10.0 { return gtk::glib::Propagation::Proceed; }

            // Subtle center line
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.04);
            cr.set_line_width(0.5);
            let mid = h / 2.0;
            cr.move_to(0.0, mid); cr.line_to(w, mid);
            cr.move_to(0.0, h * 0.25); cr.line_to(w, h * 0.25);
            cr.move_to(0.0, h * 0.75); cr.line_to(w, h * 0.75);
            let _ = cr.stroke();

            let cpu = hc.borrow(); let mem = hm.borrow();
            let rx = hr.borrow(); let tx = ht.borrow(); let gpu = hg.borrow();
            if cpu.len() < 2 { return gtk::glib::Propagation::Proceed; }

            let slots = CHART_POINTS as f64;
            let sw = w / slots;
            let x0 = w * (1.0 - cpu.len() as f64 / slots);
            let top_h = mid; // upper half: CPU + MEM (0-100%)
            let bot_h = h - mid; // lower half: Network + GPU

            // Upper: CPU (blue), MEM (green) — 0-100%
            draw_line(cr, &cpu, 100.0, 0.3, 0.8, 1.0, x0, sw, 0.0, top_h, 1.8);
            draw_line(cr, &mem, 100.0, 0.3, 0.95, 0.5, x0, sw, 0.0, top_h, 1.8);

            // Lower left: Net RX (yellow), Net TX (orange) — 0-10MB/s = 100%
            draw_line(cr, &rx,  100.0, 0.95, 0.75, 0.2, x0, sw, mid, bot_h, 1.3);
            draw_line(cr, &tx,  100.0, 1.0,  0.5,  0.25, x0, sw, mid, bot_h, 1.3);
            // Lower right: GPU temp (red, dashed-ish, mapped 0-100°C)
            draw_line(cr, &gpu, 100.0, 1.0,  0.35, 0.35, x0, sw, mid, bot_h, 1.0);

            // Labels
            cr.set_font_size(7.0);
            cr.set_source_rgba(0.3,0.8,1.0,0.6);
            cr.move_to(2.0, 9.0); let _ = cr.show_text("CPU");
            cr.set_source_rgba(0.3,0.95,0.5,0.6);
            cr.move_to(30.0, 9.0); let _ = cr.show_text("MEM");
            cr.set_source_rgba(0.95,0.75,0.2,0.5);
            cr.move_to(2.0, mid + 9.0); let _ = cr.show_text("▼RX");
            cr.set_source_rgba(1.0,0.5,0.25,0.5);
            cr.move_to(35.0, mid + 9.0); let _ = cr.show_text("▲TX");
            cr.set_source_rgba(1.0,0.35,0.35,0.5);
            cr.move_to(68.0, mid + 9.0); let _ = cr.show_text("GPU°C");

            gtk::glib::Propagation::Proceed
        });

        *self.container.borrow_mut() = Some(outer.clone());
        outer.upcast::<gtk::Widget>()
    }

    fn update(&self, snapshot: &SystemSnapshot, config: &AppearanceConfig) {
        let fs = config.font_size.max(8);
        let cpu_v = snapshot.cpu.usage_percent as f64;
        let mem_v = snapshot.memory.usage_percent as f64;
        let net = snapshot.network.first();
        let rx_v = net.map(|n| n.rx_speed_bytes as f64).unwrap_or(0.0);
        let tx_v = net.map(|n| n.tx_speed_bytes as f64).unwrap_or(0.0);
        let gpu_v = snapshot.thermal.iter()
            .find(|t| t.sensor_type.contains("x86") || t.sensor_type.contains("cpu"))
            .map(|t| t.temperature_c as f64)
            .or_else(|| snapshot.gpu.first().map(|g| g.temperature_c as f64))
            .unwrap_or(0.0);

        // Fixed-width values — two-line layout
        if let Some(ref l) = *self.cpu_label.borrow() {
            l.set_markup(&format!("<span font_desc='{}' foreground='#66ccff'>  CPU\n {:>4.0}%</span>", fs, cpu_v));
        }
        if let Some(ref l) = *self.mem_label.borrow() {
            l.set_markup(&format!("<span font_desc='{}' foreground='#99ee99'>  MEM\n {:>4.0}%</span>", fs, mem_v));
        }
        if let Some(ref l) = *self.net_label.borrow() {
            let rx_s = fmt_speed(rx_v as u64);
            let tx_s = fmt_speed(tx_v as u64);
            l.set_markup(&format!("<span font_desc='{}' foreground='#ffcc66'>  ↓{:>7}\n  ↑{:>7}</span>", fs, rx_s, tx_s));
        }
        if let Some(ref l) = *self.temp_label.borrow() {
            let gpu_t = snapshot.gpu.first().map(|g| g.temperature_c).unwrap_or(gpu_v as f32) as f64;
            let c = if gpu_t > 80.0 { "#ff4444" } else if gpu_t > 60.0 { "#ffaa44" } else { "#ff9966" };
            l.set_markup(&format!("<span font_desc='{}' foreground='{}'> 🌡 {:>3.0}°C</span>", fs, c, gpu_t));
        }

        push(&self.h_cpu, cpu_v); push(&self.h_mem, mem_v);
        push(&self.h_rx,  (rx_v / 10_485_760.0 * 100.0).min(100.0));
        push(&self.h_tx,  (tx_v / 10_485_760.0 * 100.0).min(100.0));
        push(&self.h_gpu, (gpu_v / 100.0 * 100.0).min(100.0));

        let mut t = self.tick.borrow_mut(); *t += 1;
        if *t % 2 == 0 { if let Some(ref c) = *self.chart.borrow() { c.queue_draw(); } }
    }
}

fn push(buf: &Rc<RefCell<Vec<f64>>>, val: f64) {
    let mut b = buf.borrow_mut();
    b.push(val); if b.len() > CHART_POINTS { b.remove(0); }
}

fn draw_line(cr: &gtk::cairo::Context, data: &[f64], max: f64,
             r: f64, g: f64, b: f64, x0: f64, sw: f64, y0: f64, yh: f64, lw: f64) {
    if max <= 0.0 || data.len() < 2 { return; }
    cr.set_source_rgba(r, g, b, 0.85);
    cr.set_line_width(lw);
    cr.set_line_cap(gtk::cairo::LineCap::Round);
    let mut first = true;
    for (i, v) in data.iter().enumerate() {
        let x = x0 + i as f64 * sw;
        let y = y0 + yh - (v / max * yh).clamp(0.0, yh);
        if first { cr.move_to(x, y); first = false; } else { cr.line_to(x, y); }
    }
    let _ = cr.stroke();
}

pub fn fmt_speed(bps: u64) -> String {
    if bps >= 1_000_000_000 { format!("{:5.1}GB/s", bps as f64 / 1_000_000_000.0) }
    else if bps >= 1_000_000 { format!("{:5.1}MB/s", bps as f64 / 1_000_000.0) }
    else if bps >= 1_000 { format!("{:5.1}KB/s", bps as f64 / 1_000.0) }
    else { format!("{:5}B/s", bps) }
}

pub fn fmt_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 { format!("{:.1}GB", bytes as f64 / (1024.0*1024.0*1024.0)) }
    else if bytes >= 1024 * 1024 { format!("{:.1}MB", bytes as f64 / (1024.0*1024.0)) }
    else if bytes >= 1024 { format!("{:.1}KB", bytes as f64 / 1024.0) }
    else { format!("{}B", bytes) }
}
