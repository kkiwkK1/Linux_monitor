//! Declarative user skins.
//!
//! Users drop `*.toml` spec files in `~/.config/linux-monitor/skins/`; each is
//! parsed into a [`SkinSpec`] and rendered generically by [`TemplateSkin`] using
//! the shared `draw` primitives. No code executes — it's pure data, so skins are
//! safe to share as text files and hot-reload on the next scan.
//!
//! Example spec:
//! ```toml
//! name = "My Skin"
//! layout = "vertical"      # vertical | horizontal
//! [[row]]
//! metric = "cpu"           # cpu | mem | net | disk | gpu | temp
//! element = "bar"          # bar | text | sparkline | ring
//! color = "#6cb2ff"        # hex, or "auto"/omit for the metric's theme color
//! label = "CPU"            # optional; defaults to the metric name
//! ```

use super::draw::{self, Metrics};
use super::horizontal::fmt_speed;
use super::Skin;
use crate::config::AppearanceConfig;
use crate::monitor::SystemSnapshot;
use crate::ui::theme::{self, Color};
use gtk::prelude::*;
use gtk::{Box as GtkBox, DrawingArea, Label, Orientation};
use serde::Deserialize;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

const SPARK_POINTS: usize = 90;

// ── spec ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SkinSpec {
    name: String,
    #[serde(default = "default_layout")]
    layout: String,
    #[serde(default)]
    row: Vec<RowSpec>,
}

#[derive(Deserialize, Clone)]
struct RowSpec {
    metric: String,
    #[serde(default = "default_element")]
    element: String,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    label: Option<String>,
}

fn default_layout() -> String {
    "vertical".into()
}
fn default_element() -> String {
    "text".into()
}

#[derive(Clone, Copy, PartialEq)]
enum Metric {
    Cpu,
    Mem,
    Net,
    Disk,
    Gpu,
    Temp,
}

impl Metric {
    fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cpu" => Some(Self::Cpu),
            "mem" | "memory" => Some(Self::Mem),
            "net" | "network" => Some(Self::Net),
            "disk" | "dsk" => Some(Self::Disk),
            "gpu" => Some(Self::Gpu),
            "temp" | "temperature" => Some(Self::Temp),
            _ => None,
        }
    }
    fn default_label(&self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Mem => "MEM",
            Self::Net => "NET",
            Self::Disk => "DSK",
            Self::Gpu => "GPU",
            Self::Temp => "TMP",
        }
    }
}

/// (bar/ring percentage, display text, the metric's natural theme color)
fn metric_value(m: &Metrics, metric: Metric) -> (f64, String, Color) {
    match metric {
        Metric::Cpu => (m.cpu, format!("{:.0}%", m.cpu), theme::CPU),
        Metric::Mem => (m.mem, format!("{:.0}%", m.mem), theme::MEM),
        Metric::Disk => (m.disk_pct, format!("{:.0}%", m.disk_pct), theme::DISK),
        Metric::Gpu => (m.gpu, format!("{:.0}%", m.gpu), theme::GPU),
        Metric::Net => (
            m.net_pct,
            format!("↓{} ↑{}", fmt_speed(m.rx_b as u64), fmt_speed(m.tx_b as u64)),
            theme::NET_RX,
        ),
        Metric::Temp => (
            m.temp.clamp(0.0, 100.0),
            format!("{:.0}°C", m.temp),
            theme::temp_color(m.temp),
        ),
    }
}

fn parse_hex(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 {
        let n = u32::from_str_radix(s, 16).ok()?;
        Some(Color::new((n >> 16) as u8, (n >> 8) as u8, n as u8))
    } else {
        None
    }
}

/// Fixed hex wins; "auto"/omitted → the metric's natural (grading) color.
fn resolve_color(field: &Option<String>, natural: Color) -> Color {
    match field {
        Some(s) if s != "auto" => parse_hex(s).unwrap_or(natural),
        _ => natural,
    }
}

// ── runtime widgets ──────────────────────────────────────────────────────────

enum RowWidget {
    Bar { metric: Metric, fixed: Option<Color>, pct: Rc<RefCell<f64>>, val: Label },
    Text { metric: Metric, fixed: Option<Color>, name: String, label: Label },
    Spark { metric: Metric, color: Color, hist: Rc<RefCell<Vec<f64>>>, area: DrawingArea },
    Ring { metric: Metric, fixed: Option<Color>, pct: Rc<RefCell<f64>>, area: DrawingArea },
}

// ── skin ─────────────────────────────────────────────────────────────────────

pub struct TemplateSkin {
    name: String,
    display: String,
    vertical: bool,
    rows: Vec<RowSpec>,
    widgets: RefCell<Vec<RowWidget>>,
}

impl TemplateSkin {
    fn from_spec(slug: &str, spec: SkinSpec) -> Self {
        Self {
            name: format!("user-{}", slug),
            display: spec.name,
            vertical: spec.layout.to_lowercase() != "horizontal",
            rows: spec.row,
            widgets: RefCell::new(Vec::new()),
        }
    }
}

fn fixed_color(field: &Option<String>) -> Option<Color> {
    match field {
        Some(s) if s != "auto" => parse_hex(s),
        _ => None,
    }
}

/// Draw a slim rounded progress bar filling `pct` percent, left-aligned.
fn connect_bar(area: &DrawingArea, pct: Rc<RefCell<f64>>, color: Color) {
    area.connect_draw(move |a, cr| {
        let w = a.allocation().width() as f64;
        let h = a.allocation().height() as f64;
        let bh = 6.0;
        let y = (h - bh) / 2.0;
        draw::rounded_rect(cr, 0.0, y, w, bh, bh / 2.0);
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.08);
        let _ = cr.fill();
        let fw = (w * (*pct.borrow() / 100.0)).clamp(0.0, w);
        if fw > 1.0 {
            draw::rounded_rect(cr, 0.0, y, fw, bh, bh / 2.0);
            draw::set_color(cr, color, 1.0);
            let _ = cr.fill();
        }
        gtk::glib::Propagation::Proceed
    });
}

fn connect_spark(area: &DrawingArea, hist: Rc<RefCell<Vec<f64>>>, color: Color) {
    area.connect_draw(move |a, cr| {
        let w = a.allocation().width() as f64;
        let h = a.allocation().height() as f64;
        let data = hist.borrow();
        if data.len() < 2 {
            return gtk::glib::Propagation::Proceed;
        }
        let sw = w / SPARK_POINTS as f64;
        let x0 = w * (1.0 - data.len() as f64 / SPARK_POINTS as f64);
        draw::set_color(cr, color, 0.9);
        cr.set_line_width(1.3);
        cr.set_line_join(gtk::cairo::LineJoin::Round);
        for (i, v) in data.iter().enumerate() {
            let x = x0 + i as f64 * sw;
            let yy = h - (v / 100.0 * h).clamp(0.0, h);
            if i == 0 {
                cr.move_to(x, yy);
            } else {
                cr.line_to(x, yy);
            }
        }
        let _ = cr.stroke();
        gtk::glib::Propagation::Proceed
    });
}

fn connect_ring(area: &DrawingArea, pct: Rc<RefCell<f64>>, color: Color) {
    area.connect_draw(move |a, cr| {
        let w = a.allocation().width() as f64;
        let h = a.allocation().height() as f64;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let rad = (cx.min(cy) - 5.0).max(6.0);
        let lw = (rad * 0.24).max(3.5);
        let start = -std::f64::consts::PI / 2.0;
        cr.set_line_width(lw);
        cr.set_line_cap(gtk::cairo::LineCap::Butt);
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.10);
        cr.new_sub_path();
        cr.arc(cx, cy, rad, 0.0, 2.0 * std::f64::consts::PI);
        let _ = cr.stroke();
        let p = (*pct.borrow() / 100.0).clamp(0.0, 1.0);
        if p > 0.0 {
            draw::set_color(cr, color, 1.0);
            cr.new_sub_path();
            cr.arc(cx, cy, rad, start, start + p * 2.0 * std::f64::consts::PI);
            let _ = cr.stroke();
        }
        draw::text(cr, &format!("{:.0}", *pct.borrow()), cx, cy, rad * 0.7, theme::TEXT, 1.0, true, true);
        gtk::glib::Propagation::Proceed
    });
}

impl Skin for TemplateSkin {
    fn name(&self) -> &str {
        &self.name
    }
    fn display_name(&self) -> &str {
        &self.display
    }

    fn create_widget(&self, config: &AppearanceConfig) -> gtk::Widget {
        let fs = config.font_size.max(9) as i32;
        let orient = if self.vertical { Orientation::Vertical } else { Orientation::Horizontal };
        let outer = GtkBox::new(orient, if self.vertical { 6 } else { 12 });
        outer.style_context().add_class("skin-template");
        outer.set_margin_start(12);
        outer.set_margin_end(12);
        outer.set_margin_top(8);
        outer.set_margin_bottom(8);

        let key_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        let val_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        let mut widgets = Vec::new();

        for row in &self.rows {
            let metric = match Metric::parse(&row.metric) {
                Some(m) => m,
                None => continue,
            };
            let name = row.label.clone().unwrap_or_else(|| metric.default_label().to_string());
            let natural = metric_value(&Metrics::from(&empty_snapshot()), metric).2;
            let color = resolve_color(&row.color, natural);
            let fixed = fixed_color(&row.color);

            match row.element.to_lowercase().as_str() {
                "bar" => {
                    let r = GtkBox::new(Orientation::Horizontal, 8);
                    let k = Label::new(None);
                    k.set_markup(&format!("<span font_desc='{}' foreground='{}'><b>{}</b></span>", fs - 1, color.hex(), name));
                    k.set_xalign(0.0);
                    key_group.add_widget(&k);
                    r.pack_start(&k, false, false, 0);

                    let bar = DrawingArea::new();
                    bar.set_size_request(80, 12);
                    bar.set_valign(gtk::Align::Center);
                    let pct = Rc::new(RefCell::new(0.0));
                    connect_bar(&bar, pct.clone(), color);
                    r.pack_start(&bar, true, true, 0);

                    let v = Label::new(None);
                    v.set_xalign(1.0);
                    val_group.add_widget(&v);
                    r.pack_start(&v, false, false, 0);
                    outer.add(&r);
                    widgets.push(RowWidget::Bar { metric, fixed, pct, val: v });
                }
                "sparkline" | "spark" => {
                    let r = GtkBox::new(Orientation::Horizontal, 8);
                    let k = Label::new(None);
                    k.set_markup(&format!("<span font_desc='{}' foreground='{}'><b>{}</b></span>", fs - 1, color.hex(), name));
                    k.set_xalign(0.0);
                    key_group.add_widget(&k);
                    r.pack_start(&k, false, false, 0);
                    let area = DrawingArea::new();
                    area.set_size_request(90, 22);
                    let hist = Rc::new(RefCell::new(Vec::with_capacity(SPARK_POINTS)));
                    connect_spark(&area, hist.clone(), color);
                    r.pack_start(&area, true, true, 0);
                    outer.add(&r);
                    widgets.push(RowWidget::Spark { metric, color, hist, area });
                }
                "ring" => {
                    let r = GtkBox::new(Orientation::Horizontal, 8);
                    let area = DrawingArea::new();
                    area.set_size_request(52, 52);
                    let pct = Rc::new(RefCell::new(0.0));
                    connect_ring(&area, pct.clone(), color);
                    r.pack_start(&area, false, false, 0);
                    let k = Label::new(None);
                    k.set_markup(&format!("<span font_desc='{}' foreground='{}'><b>{}</b></span>", fs, color.hex(), name));
                    k.set_xalign(0.0);
                    r.pack_start(&k, false, false, 0);
                    outer.add(&r);
                    widgets.push(RowWidget::Ring { metric, fixed, pct, area });
                }
                _ => {
                    // text
                    let l = Label::new(None);
                    l.set_xalign(0.0);
                    if self.vertical {
                        key_group.add_widget(&l);
                    }
                    outer.add(&l);
                    widgets.push(RowWidget::Text { metric, fixed, name, label: l });
                }
            }
        }

        if widgets.is_empty() {
            let l = Label::new(None);
            l.set_markup(&format!("<span foreground='{}'>empty skin</span>", theme::TEXT_DIM.hex()));
            outer.add(&l);
        }

        *self.widgets.borrow_mut() = widgets;
        outer.upcast::<gtk::Widget>()
    }

    fn update(&self, snapshot: &SystemSnapshot, config: &AppearanceConfig) {
        let fs = config.font_size.max(9) as i32;
        let m = Metrics::from(snapshot);
        for rw in self.widgets.borrow().iter() {
            match rw {
                RowWidget::Bar { metric, fixed, pct, val } => {
                    let (p, _txt, _nat) = metric_value(&m, *metric);
                    *pct.borrow_mut() = p;
                    val.set_markup(&format!(
                        "<span font_desc='{}' foreground='{}'>{:>3.0}%</span>",
                        fs - 1,
                        fixed.unwrap_or(theme::TEXT_DIM).hex(),
                        p
                    ));
                }
                RowWidget::Text { metric, fixed, name, label } => {
                    let (_p, txt, nat) = metric_value(&m, *metric);
                    let c = fixed.unwrap_or(nat);
                    label.set_markup(&format!(
                        "<span font_desc='{}' foreground='{}'><b>{}</b></span>  <span font_desc='{}' foreground='{}'>{}</span>",
                        fs, c.hex(), name, fs, theme::TEXT.hex(), txt
                    ));
                }
                RowWidget::Spark { metric, hist, area, .. } => {
                    let (p, _txt, _nat) = metric_value(&m, *metric);
                    {
                        let mut b = hist.borrow_mut();
                        b.push(p);
                        if b.len() > SPARK_POINTS {
                            b.remove(0);
                        }
                    }
                    area.queue_draw();
                }
                RowWidget::Ring { metric, pct, area, .. } => {
                    let (p, _txt, _nat) = metric_value(&m, *metric);
                    *pct.borrow_mut() = p;
                    area.queue_draw();
                }
            }
        }
    }
}

// ── loading ──────────────────────────────────────────────────────────────────

/// `~/.config/linux-monitor/skins`
pub fn skins_dir() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())).join(".config")
        });
    base.join("linux-monitor").join("skins")
}

/// GAction-safe slug from a file stem.
fn slug(stem: &str) -> String {
    stem.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect()
}

/// Ensure the skins dir exists; on first creation, drop a commented example.
fn ensure_dir() {
    let dir = skins_dir();
    if !dir.exists() {
        if std::fs::create_dir_all(&dir).is_ok() {
            let _ = std::fs::write(dir.join("example.toml"), EXAMPLE_SKIN);
        }
    }
}

fn parse_file(path: &std::path::Path) -> Option<TemplateSkin> {
    let stem = path.file_stem()?.to_str()?;
    let content = std::fs::read_to_string(path).ok()?;
    let spec: SkinSpec = toml::from_str(&content).ok()?;
    Some(TemplateSkin::from_spec(&slug(stem), spec))
}

/// All valid user skins found in the skins dir.
pub fn load_all() -> Vec<TemplateSkin> {
    ensure_dir();
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(skins_dir()) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().map_or(false, |x| x == "toml") {
                if let Some(s) = parse_file(&p) {
                    out.push(s);
                }
            }
        }
    }
    out
}

/// Load a single user skin by its slug (the part after `user-`).
pub fn load_one(slug_arg: &str) -> Option<TemplateSkin> {
    ensure_dir();
    if let Ok(entries) = std::fs::read_dir(skins_dir()) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().map_or(false, |x| x == "toml") {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    if slug(stem) == slug_arg {
                        return parse_file(&p);
                    }
                }
            }
        }
    }
    None
}

/// A synthetic all-zero snapshot, used only to look up a metric's natural color
/// at build time (before any real update arrives).
fn empty_snapshot() -> SystemSnapshot {
    SystemSnapshot {
        timestamp: 0,
        cpu: Default::default(),
        memory: Default::default(),
        network: Vec::new(),
        disk: Vec::new(),
        gpu: Vec::new(),
        thermal: Vec::new(),
    }
}

const EXAMPLE_SKIN: &str = r##"# LinuxMonitor custom skin — edit or copy this file.
# Drop *.toml here (~/.config/linux-monitor/skins/) and pick it from the
# right-click "Skins" menu. Re-open the menu after adding/editing a file.

name = "My Custom Skin"     # shown in the menu
layout = "vertical"          # vertical | horizontal

# metric:  cpu | mem | net | disk | gpu | temp
# element: bar | text | sparkline | ring
# color:   "#rrggbb", or "auto"/omit for the metric's built-in color
#          (temp "auto" grades green→amber→red by heat)

[[row]]
metric = "cpu"
element = "bar"

[[row]]
metric = "mem"
element = "bar"

[[row]]
metric = "cpu"
element = "sparkline"
color = "#6cb2ff"

[[row]]
metric = "net"
element = "text"

[[row]]
metric = "temp"
element = "text"
color = "auto"
"##;
