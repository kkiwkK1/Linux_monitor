//! The app icon ("Pulse" concept), drawn with Cairo so no image assets or
//! external rasterizer are needed. Used as the window/default icon and exported
//! to a PNG that the autostart `.desktop` (and packaging) can reference.

use crate::ui::skins::draw::rounded_rect;
use gtk::cairo::{Context, Format, ImageSurface, LinearGradient};
use gtk::gdk_pixbuf::Pixbuf;
use std::path::PathBuf;

/// Paint the icon in a 128×128 coordinate space (caller scales the context).
fn paint(cr: &Context) {
    // Card background — dark vertical gradient, rounded square.
    let bg = LinearGradient::new(0.0, 4.0, 0.0, 124.0);
    bg.add_color_stop_rgb(0.0, 0x19 as f64 / 255.0, 0x1b as f64 / 255.0, 0x20 as f64 / 255.0);
    bg.add_color_stop_rgb(1.0, 0x0c as f64 / 255.0, 0x0d as f64 / 255.0, 0x10 as f64 / 255.0);
    rounded_rect(cr, 4.0, 4.0, 120.0, 120.0, 28.0);
    let _ = cr.set_source(&bg);
    let _ = cr.fill();

    // Top sheen (clipped to the card) for a bit of depth.
    cr.save().ok();
    rounded_rect(cr, 4.0, 4.0, 120.0, 120.0, 28.0);
    let _ = cr.clip();
    let sheen = LinearGradient::new(0.0, 4.0, 0.0, 68.0);
    sheen.add_color_stop_rgba(0.0, 1.0, 1.0, 1.0, 0.10);
    sheen.add_color_stop_rgba(1.0, 1.0, 1.0, 1.0, 0.0);
    let _ = cr.set_source(&sheen);
    cr.rectangle(0.0, 0.0, 128.0, 68.0);
    let _ = cr.fill();
    cr.restore().ok();

    // Area fill under the primary sparkline.
    let spark = [(22.0, 86.0), (38.0, 78.0), (50.0, 92.0), (64.0, 52.0), (78.0, 70.0), (92.0, 40.0), (106.0, 58.0)];
    polyline(cr, &spark);
    cr.line_to(106.0, 100.0);
    cr.line_to(22.0, 100.0);
    cr.close_path();
    let af = LinearGradient::new(0.0, 40.0, 0.0, 100.0);
    af.add_color_stop_rgba(0.0, 0x6c as f64 / 255.0, 0xb2 as f64 / 255.0, 1.0, 0.35);
    af.add_color_stop_rgba(1.0, 0x6c as f64 / 255.0, 0xb2 as f64 / 255.0, 1.0, 0.0);
    let _ = cr.set_source(&af);
    let _ = cr.fill();

    cr.set_line_cap(gtk::cairo::LineCap::Round);
    cr.set_line_join(gtk::cairo::LineJoin::Round);

    // Primary sparkline (CPU blue).
    polyline(cr, &spark);
    cr.set_source_rgb(0x6c as f64 / 255.0, 0xb2 as f64 / 255.0, 1.0);
    cr.set_line_width(7.0);
    let _ = cr.stroke();

    // Secondary line (MEM green).
    let line2 = [(22.0, 74.0), (40.0, 68.0), (54.0, 78.0), (70.0, 60.0), (86.0, 72.0), (106.0, 50.0)];
    polyline(cr, &line2);
    cr.set_source_rgba(0x4a as f64 / 255.0, 0xd6 as f64 / 255.0, 0xa4 as f64 / 255.0, 0.9);
    cr.set_line_width(5.0);
    let _ = cr.stroke();

    // Emphasized endpoint.
    cr.arc(106.0, 58.0, 6.0, 0.0, 2.0 * std::f64::consts::PI);
    cr.set_source_rgb(0xee as f64 / 255.0, 0xf0 as f64 / 255.0, 0xf3 as f64 / 255.0);
    let _ = cr.fill();

    // Hairline border.
    rounded_rect(cr, 4.5, 4.5, 119.0, 119.0, 27.5);
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.12);
    cr.set_line_width(1.0);
    let _ = cr.stroke();
}

fn polyline(cr: &Context, pts: &[(f64, f64)]) {
    for (i, (x, y)) in pts.iter().enumerate() {
        if i == 0 {
            cr.move_to(*x, *y);
        } else {
            cr.line_to(*x, *y);
        }
    }
}

fn render(size: i32) -> Option<ImageSurface> {
    let surface = ImageSurface::create(Format::ARgb32, size, size).ok()?;
    {
        let cr = Context::new(&surface).ok()?;
        cr.scale(size as f64 / 128.0, size as f64 / 128.0);
        paint(&cr);
    }
    Some(surface)
}

/// The icon as a Pixbuf at the given size (for `Window::set_icon`).
pub fn pixbuf(size: i32) -> Option<Pixbuf> {
    let surface = render(size)?;
    gtk::gdk::pixbuf_get_from_surface(&surface, 0, 0, size, size)
}

/// `~/.local/share/linux-monitor/icon.png`
pub fn png_path() -> PathBuf {
    let base = std::env::var("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|_| {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())).join(".local/share")
    });
    base.join("linux-monitor").join("icon.png")
}

/// Render a 256px PNG to [`png_path`] (idempotently overwrites). Returns the
/// path on success so the autostart entry / packaging can point `Icon=` at it.
pub fn ensure_png() -> Option<PathBuf> {
    let path = png_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok()?;
    }
    let surface = render(256)?;
    let mut file = std::fs::File::create(&path).ok()?;
    surface.write_to_png(&mut file).ok()?;
    Some(path)
}
