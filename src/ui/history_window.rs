use crate::history::store::HistoryStore;
use crate::history::TimeRange;
use crate::locale::L;
use gtk::prelude::*;
use std::sync::Arc;

pub struct HistoryWindow {
    _window: gtk::Window,
}

impl HistoryWindow {
    pub fn new(store: Arc<HistoryStore>, parent: &gtk::ApplicationWindow) -> Self {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_title(L.history_title());
        window.set_default_size(720, 420);
        window.set_modal(true);
        window.set_transient_for(Some(parent));
        window.set_destroy_with_parent(true);

        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Header
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header.set_margin_start(12); header.set_margin_end(12); header.set_margin_top(8);

        let title = gtk::Label::new(None);
        title.set_markup(&format!("<b>{}</b>", L.history_title()));
        header.add(&title);
        header.set_hexpand(true);

        let ranges = [L.range_5min(), L.range_30min(), L.range_1hour(),
                       L.range_6hours(), L.range_24hours(), L.range_all()];
        let range_combo = gtk::ComboBoxText::new();
        for r in &ranges { range_combo.append_text(r); }
        range_combo.set_active(Some(2));
        header.add(&range_combo);

        let export_btn = gtk::Button::with_label(L.history_export());
        header.add(&export_btn);
        main_box.add(&header);

        // Stats bar
        let stats_label = gtk::Label::new(Some(&format!("{}: ... | {}: ... | {}",
            L.history_records(), L.history_db_size(), L.history_retention())));
        stats_label.set_margin_start(12); stats_label.set_margin_end(12); stats_label.set_margin_top(4);
        main_box.add(&stats_label);

        // Drawing area
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);
        drawing_area.set_margin_start(12); drawing_area.set_margin_end(12); drawing_area.set_margin_bottom(12);
        drawing_area.set_size_request(-1, 280);
        main_box.add(&drawing_area);

        let store_for_draw = store.clone();
        drawing_area.connect_draw(move |area, cr| {
            draw_charts(area, cr, &store_for_draw, &TimeRange::Last1Hour);
            gtk::glib::Propagation::Proceed
        });

        // Stats
        if let Ok((count, size)) = store.stats() {
            stats_label.set_text(&format!("{}: {} | {}: {} | {}",
                L.history_records(), count, L.history_db_size(), format_bytes(size), L.history_retention()));
        }

        // Range change → redraw
        let da = drawing_area.clone();
        range_combo.connect_changed(move |combo| {
            da.queue_draw();
            let _ = combo.active();
        });

        // Export
        let store_export = store.clone();
        export_btn.connect_clicked(move |_| { export_csv_dialog(&store_export, &TimeRange::Last1Hour); });

        window.add(&main_box);
        window.show_all();

        Self { _window: window }
    }
}

fn draw_charts(area: &gtk::DrawingArea, cr: &cairo::Context,
               store: &Arc<HistoryStore>, range: &TimeRange)
{
    let records = match store.query(range.clone()) {
        Ok(r) => r, Err(_) => return,
    };
    if records.is_empty() {
        cr.set_source_rgb(0.5, 0.5, 0.5);
        cr.move_to(20.0, 30.0);
        let layout = pangocairo::create_layout(cr);
        layout.set_text(L.history_no_data());
        pangocairo::show_layout(cr, &layout);
        return;
    }

    let width = area.allocation().width() as f64;
    let height = area.allocation().height() as f64;
    let ml = 50.0; let mr = 20.0; let mt = 15.0; let mb = 30.0;
    let cw = width - ml - mr;
    let ch = height - mt - mb;

    cr.set_source_rgba(0.1, 0.1, 0.1, 0.5);
    cr.rectangle(ml, mt, cw, ch);
    let _ = cr.fill();
    cr.set_line_width(0.5);
    cr.set_source_rgba(0.3, 0.3, 0.3, 0.5);
    for i in 0..=4 {
        let y = mt + ch * (i as f64 / 4.0);
        cr.move_to(ml, y); cr.line_to(ml + cw, y);
    }
    let _ = cr.stroke();

    draw_line(cr, &records, ml, mt, cw, ch, |r| r.cpu_percent as f64, 0.0, 100.0, 0.2, 0.7, 1.0);
    draw_line(cr, &records, ml, mt, cw, ch, |r| r.mem_percent as f64, 0.0, 100.0, 0.4, 0.9, 0.4);
    draw_line(cr, &records, ml, mt, cw, ch, |r| (r.net_rx_speed as f64 / 1024.0).min(1024.0), 0.0, 1024.0, 0.9, 0.7, 0.2);

    // Legend
    cr.set_font_size(11.0);
    let ly = height - 10.0;
    cr.set_source_rgb(0.2, 0.7, 1.0); cr.move_to(ml, ly); let _ = cr.show_text("── CPU");
    cr.set_source_rgb(0.4, 0.9, 0.4); cr.move_to(ml + 90.0, ly); let _ = cr.show_text("── MEM");
    cr.set_source_rgb(0.9, 0.7, 0.2); cr.move_to(ml + 190.0, ly); let _ = cr.show_text("── Net KB/s");
}

fn draw_line<F>(cr: &cairo::Context, records: &[crate::history::HistoryRecord],
    ml: f64, mt: f64, cw: f64, ch: f64, get_val: F,
    min_val: f64, max_val: f64, r: f64, g: f64, b: f64)
where F: Fn(&crate::history::HistoryRecord) -> f64
{
    if records.len() < 2 { return; }
    let vr = max_val - min_val;
    cr.set_source_rgba(r, g, b, 0.7);
    cr.set_font_size(9.0);
    for i in 0..=4 {
        let val = max_val - vr * (i as f64 / 4.0);
        let y = mt + ch * (i as f64 / 4.0);
        let label = if max_val <= 100.0 { format!("{:.0}%", val) } else { format!("{:.0}", val) };
        cr.move_to(ml - 42.0, y + 3.0);
        let _ = cr.show_text(&label);
    }
    cr.set_source_rgba(r, g, b, 0.8);
    cr.set_line_width(1.8);
    cr.set_line_cap(cairo::LineCap::Round);
    let fts = records[0].timestamp as f64;
    let lts = records[records.len()-1].timestamp as f64;
    let tr = if lts > fts { lts - fts } else { 1.0 };
    let mut first = true;
    for rec in records {
        let x = ml + cw * (rec.timestamp as f64 - fts) / tr;
        let v = get_val(rec).clamp(min_val, max_val);
        let y = mt + ch * (1.0 - (v - min_val) / vr);
        if first { cr.move_to(x, y); first = false; }
        else { cr.line_to(x, y); }
    }
    let _ = cr.stroke();
}

fn export_csv_dialog(store: &Arc<HistoryStore>, range: &TimeRange) {
    let dialog = gtk::FileChooserDialog::new(
        Some(L.history_export_title()),
        None::<&gtk::Window>,
        gtk::FileChooserAction::Save,
    );
    dialog.add_button(L.settings_cancel(), gtk::ResponseType::Cancel);
    dialog.add_button(L.history_export_btn(), gtk::ResponseType::Accept);
    dialog.set_current_name("linux-monitor-history.csv");
    let store = store.clone();
    let range = range.clone();
    dialog.connect_response(move |dlg, resp| {
        if resp == gtk::ResponseType::Accept {
            if let Some(file) = dlg.file() {
                if let Some(path) = file.path() {
                    if let Some(p) = path.to_str() {
                        match store.export_csv(p, range.clone()) {
                            Ok(n) => log::info!("Exported {} records", n),
                            Err(e) => log::error!("Export failed: {}", e),
                        }
                    }
                }
            }
        }
        dlg.close();
    });
    dialog.show_all();
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 { format!("{:.1} MB", bytes as f64 / (1024.0*1024.0)) }
    else if bytes >= 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else { format!("{} B", bytes) }
}
