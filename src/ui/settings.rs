use crate::config::AppConfig;
use crate::locale::L;
use crate::plugin::PluginManager;
use crate::ui::floating_window::FloatingWindow;
use crate::ui::skins;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Settings/preferences window
pub struct SettingsWindow {
    _window: gtk::Window,
}

impl SettingsWindow {
    pub fn new(
        config: Rc<RefCell<AppConfig>>,
        parent: &gtk::ApplicationWindow,
        floating: Option<&Rc<FloatingWindow>>,
        plugin_manager: Option<Arc<PluginManager>>,
    ) -> Self {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_title(L.settings_title());
        window.set_default_size(480, 520);
        window.set_modal(true);
        window.set_transient_for(Some(parent));
        window.set_destroy_with_parent(true);

        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let header = gtk::HeaderBar::new();
        header.set_show_close_button(true);
        window.set_titlebar(Some(&header));

        let notebook = gtk::Notebook::new();
        notebook.set_tab_pos(gtk::PositionType::Left);

        // Tab 1: General
        let general_page = build_general_page(&config);
        let general_label = gtk::Label::new(Some(L.settings_tab_general()));
        notebook.append_page(&general_page, Some(&general_label));

        // Tab 2: Monitors
        let monitors_page = build_monitors_page(&config);
        let monitors_label = gtk::Label::new(Some(L.settings_tab_monitors()));
        notebook.append_page(&monitors_page, Some(&monitors_label));

        // Tab 3: Appearance
        let appearance_page = build_appearance_page(&config, floating.cloned());
        let appearance_label = gtk::Label::new(Some(L.settings_tab_appearance()));
        notebook.append_page(&appearance_page, Some(&appearance_label));

        // Tab 4: Plugins
        if let Some(ref pm) = plugin_manager {
            let plugins_page = build_plugins_page(pm);
            let plugins_label = gtk::Label::new(Some(L.plugins_tab()));
            notebook.append_page(&plugins_page, Some(&plugins_label));
        }

        main_box.add(&notebook);

        // Bottom buttons
        let button_box = gtk::ButtonBox::new(gtk::Orientation::Horizontal);
        button_box.set_margin_top(12); button_box.set_margin_bottom(12);
        button_box.set_margin_start(12); button_box.set_margin_end(12);
        button_box.set_layout(gtk::ButtonBoxStyle::End);

        let cancel_btn = gtk::Button::with_label(L.settings_cancel());
        let apply_btn = gtk::Button::with_label(L.settings_apply());
        apply_btn.style_context().add_class("suggested-action");

        let win_close = window.clone();
        cancel_btn.connect_clicked(move |_| win_close.close());

        let config_save = config.clone();
        let win_save = window.clone();
        apply_btn.connect_clicked(move |_| {
            if let Err(e) = config_save.borrow().save() {
                log::error!("Failed to save config: {}", e);
            }
            win_save.close();
        });

        button_box.add(&cancel_btn);
        button_box.add(&apply_btn);
        main_box.add(&button_box);

        window.add(&main_box);
        window.show_all();

        Self { _window: window }
    }
}

fn build_general_page(config: &Rc<RefCell<AppConfig>>) -> gtk::Widget {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 16);
    page.set_margin_start(24); page.set_margin_end(24); page.set_margin_top(24);

    let title = gtk::Label::new(None);
    title.set_markup(&format!("<b>{}</b>", L.settings_general_title()));
    title.set_halign(gtk::Align::Start);
    page.add(&title);

    for (label, desc, min, max, step, val) in [
        (L.settings_active_interval(), L.settings_active_desc(), 200.0, 10000.0, 100.0,
         config.borrow().poll.active_interval_ms as f64),
        (L.settings_bg_interval(), L.settings_bg_desc(), 1000.0, 60000.0, 500.0,
         config.borrow().poll.background_interval_ms as f64),
        (L.settings_idle_interval(), L.settings_idle_desc(), 5000.0, 120000.0, 1000.0,
         config.borrow().poll.idle_interval_ms as f64),
    ] {
        let (row, spin) = build_spin_row(label, desc, min, max, step, val);
        let cfg = config.clone();
        let label_copy = label.to_string();
        spin.connect_value_changed(move |s| {
            let mut c = cfg.borrow_mut();
            let v = s.value() as u64;
            if label_copy.contains("Active") || label_copy.contains("活跃") {
                c.poll.active_interval_ms = v;
            } else if label_copy.contains("Background") || label_copy.contains("后台") {
                c.poll.background_interval_ms = v;
            } else {
                c.poll.idle_interval_ms = v;
            }
        });
        page.add(&row);
    }
    page.upcast::<gtk::Widget>()
}

fn build_monitors_page(config: &Rc<RefCell<AppConfig>>) -> gtk::Widget {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 12);
    page.set_margin_start(24); page.set_margin_end(24); page.set_margin_top(24);

    let title = gtk::Label::new(None);
    title.set_markup(&format!("<b>{}</b>", L.settings_monitors_title()));
    title.set_halign(gtk::Align::Start);
    page.add(&title);

    let monitors: [(&str, &str, bool); 6] = [
        (L.settings_monitor_cpu(), L.settings_monitor_cpu_desc(), config.borrow().monitors.cpu),
        (L.settings_monitor_memory(), L.settings_monitor_memory_desc(), config.borrow().monitors.memory),
        (L.settings_monitor_network(), L.settings_monitor_network_desc(), config.borrow().monitors.network),
        (L.settings_monitor_disk(), L.settings_monitor_disk_desc(), config.borrow().monitors.disk),
        (L.settings_monitor_gpu(), L.settings_monitor_gpu_desc(), config.borrow().monitors.gpu),
        (L.settings_monitor_thermal(), L.settings_monitor_thermal_desc(), config.borrow().monitors.thermal),
    ];

    for (label, desc, enabled) in &monitors {
        let switch = gtk::Switch::new();
        switch.set_active(*enabled);
        switch.set_valign(gtk::Align::Center);

        let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let name_label = gtk::Label::new(None);
        name_label.set_halign(gtk::Align::Start);
        name_label.set_markup(&format!("<b>{}</b>", label));
        let desc_label = gtk::Label::new(Some(desc));
        desc_label.set_halign(gtk::Align::Start);
        desc_label.style_context().add_class("dim-label");
        text_box.add(&name_label);
        text_box.add(&desc_label);
        row.add(&text_box);
        row.set_hexpand(true);
        row.add(&switch);

        let cfg = config.clone();
        let ls = label.to_string();
        switch.connect_state_set(move |_, active| {
            let mut c = cfg.borrow_mut();
            match () {
                _ if ls.contains("CPU") => c.monitors.cpu = active,
                _ if ls.contains("内存") || ls.contains("Memory") => c.monitors.memory = active,
                _ if ls.contains("网络") || ls.contains("Network") => c.monitors.network = active,
                _ if ls.contains("磁盘") || ls.contains("Disk") => c.monitors.disk = active,
                _ if ls.contains("GPU") => c.monitors.gpu = active,
                _ => c.monitors.thermal = active,
            }
            gtk::glib::Propagation::Proceed
        });
        page.add(&row);
    }
    page.upcast::<gtk::Widget>()
}

fn build_appearance_page(config: &Rc<RefCell<AppConfig>>, floating: Option<Rc<FloatingWindow>>) -> gtk::Widget {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 16);
    page.set_margin_start(24); page.set_margin_end(24); page.set_margin_top(24);

    let title = gtk::Label::new(None);
    title.set_markup(&format!("<b>{}</b>", L.settings_appearance_title()));
    title.set_halign(gtk::Align::Start);
    page.add(&title);

    let skin_label = gtk::Label::new(Some(L.settings_skin()));
    skin_label.set_halign(gtk::Align::Start);
    page.add(&skin_label);

    let skin_combo = gtk::ComboBoxText::new();
    for s in skins::available_skins() {
        skin_combo.append(Some(s.name()), s.display_name());
    }
    let current_skin = &config.borrow().appearance.skin;
    skin_combo.set_active_id(Some(current_skin));
    let cfg = config.clone();
    skin_combo.connect_changed(move |combo| {
        if let Some(id) = combo.active_id() {
            cfg.borrow_mut().appearance.skin = id.to_string();
        }
    });
    page.add(&skin_combo);

    let (font_row, font_spin) = build_spin_row(L.settings_font_size(), L.settings_font_desc(),
        8.0, 24.0, 1.0, config.borrow().appearance.font_size as f64);
    let cfg = config.clone();
    font_spin.connect_value_changed(move |s| { cfg.borrow_mut().appearance.font_size = s.value() as u32; });
    page.add(&font_row);

    let (opacity_row, opacity_spin) = build_spin_row(L.settings_opacity(), L.settings_opacity_desc(),
        0.3, 1.0, 0.05, config.borrow().appearance.opacity);
    let cfg = config.clone();
    let fl_op = floating.clone();
    opacity_spin.connect_value_changed(move |s| {
        let v = s.value();
        cfg.borrow_mut().appearance.opacity = v;
        if let Some(ref f) = fl_op { f.set_opacity(v); }
    });
    page.add(&opacity_row);

    let top_switch = gtk::Switch::new();
    top_switch.set_active(config.borrow().appearance.always_on_top);
    top_switch.set_valign(gtk::Align::Center);
    let top_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let top_label = gtk::Label::new(Some(L.settings_always_on_top()));
    top_label.set_halign(gtk::Align::Start);
    top_row.add(&top_label);
    top_row.set_hexpand(true);
    top_row.add(&top_switch);
    let cfg = config.clone();
    let fl = floating.clone();
    top_switch.connect_state_set(move |_, active| {
        cfg.borrow_mut().appearance.always_on_top = active;
        if let Some(ref f) = fl {
            f.set_keep_above(active);
        }
        gtk::glib::Propagation::Proceed
    });
    page.add(&top_row);

    // ── Background ──
    let bg_label = gtk::Label::new(Some(L.settings_bg_type()));
    bg_label.set_halign(gtk::Align::Start);
    bg_label.set_margin_top(8);
    page.add(&bg_label);

    let bg_combo = gtk::ComboBoxText::new();
    bg_combo.append_text(L.settings_bg_none());
    bg_combo.append_text(L.settings_bg_color());
    bg_combo.append_text(L.settings_bg_image());
    bg_combo.set_active(match config.borrow().appearance.background_type {
        crate::config::BackgroundType::None => Some(0),
        crate::config::BackgroundType::Color => Some(1),
        crate::config::BackgroundType::Image => Some(2),
    });
    page.add(&bg_combo);

    // Color picker
    let color_btn = gtk::ColorButton::new();
    color_btn.set_title(L.settings_pick_color());
    page.add(&color_btn);

    // Image picker
    let img_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let img_label = gtk::Label::new(None);
    if !config.borrow().appearance.background_image.is_empty() {
        img_label.set_text(&config.borrow().appearance.background_image);
    } else {
        img_label.set_text(L.settings_pick_image());
    }
    img_label.set_halign(gtk::Align::Start);
    img_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    img_row.add(&img_label);
    let img_btn = gtk::Button::with_label("...");
    img_row.add(&img_btn);
    page.add(&img_row);

    // Connect background changes
    let cfg_bg = config.clone();
    let fl_bg = floating.clone();
    let color_btn2 = color_btn.clone();
    let img_label2 = img_label.clone();
    bg_combo.connect_changed(move |combo| {
        let bg_type = match combo.active() {
            Some(0) => crate::config::BackgroundType::None,
            Some(1) => crate::config::BackgroundType::Color,
            _ => crate::config::BackgroundType::Image,
        };
        cfg_bg.borrow_mut().appearance.background_type = bg_type.clone();
        if let Some(ref f) = fl_bg { f.set_background(&cfg_bg.borrow().appearance); }
    });

    let cfg_color = config.clone();
    let fl_color = floating.clone();
    let bg_combo_cc = bg_combo.clone(); color_btn.connect_color_set(move |btn| {
        let rgba = btn.rgba();
        let hex = format!("#{:02x}{:02x}{:02x}", (rgba.red()*255.0) as u8, (rgba.green()*255.0) as u8, (rgba.blue()*255.0) as u8);
        cfg_color.borrow_mut().appearance.background_color = hex;
        cfg_color.borrow_mut().appearance.background_type = crate::config::BackgroundType::Color;
        bg_combo_cc.set_active(Some(1));
        if let Some(ref f) = fl_color { f.set_background(&cfg_color.borrow().appearance); }
    });

    let cfg_img = config.clone();
    let fl_img = floating.clone();
    let img_lbl = img_label2.clone();
    img_btn.connect_clicked(move |_| {
        let dialog = gtk::FileChooserDialog::new(
            Some(L.settings_pick_image()), None::<&gtk::Window>,
            gtk::FileChooserAction::Open,
        );
        dialog.add_button(L.settings_cancel(), gtk::ResponseType::Cancel);
        dialog.add_button("Open", gtk::ResponseType::Accept);
        let filter = gtk::FileFilter::new();
        filter.add_pixbuf_formats();
        dialog.set_filter(&filter);
        let cfg = cfg_img.clone();
        let fl = fl_img.clone();
        let lbl = img_lbl.clone();
        let combo = bg_combo.clone();
        dialog.connect_response(move |dlg, resp| {
            if resp == gtk::ResponseType::Accept {
                if let Some(file) = dlg.file() {
                    if let Some(path) = file.path() {
                        if let Some(p) = path.to_str() {
                            lbl.set_text(p);
                            cfg.borrow_mut().appearance.background_image = p.to_string();
                            cfg.borrow_mut().appearance.background_type = crate::config::BackgroundType::Image;
                            combo.set_active(Some(2));
                            if let Some(ref f) = fl { f.set_background(&cfg.borrow().appearance); }
                        }
                    }
                }
            }
            dlg.close();
        });
        dialog.show_all();
    });

    page.upcast::<gtk::Widget>()
}


fn build_spin_row(title: &str, subtitle: &str, min: f64, max: f64, step: f64, value: f64)
    -> (gtk::Widget, gtk::SpinButton)
{
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let title_label = gtk::Label::new(None);
    title_label.set_halign(gtk::Align::Start);
    title_label.set_markup(&format!("<b>{}</b>", title));
    let sub_label = gtk::Label::new(Some(subtitle));
    sub_label.set_halign(gtk::Align::Start);
    sub_label.style_context().add_class("dim-label");
    text_box.add(&title_label);
    text_box.add(&sub_label);
    let adj = gtk::Adjustment::new(value, min, max, step, step * 10.0, 0.0);
    let spin = gtk::SpinButton::new(Some(&adj), step, 0);
    spin.set_valign(gtk::Align::Center);
    row.add(&text_box);
    row.set_hexpand(true);
    row.add(&spin);
    (row.upcast::<gtk::Widget>(), spin)
}

fn build_plugins_page(pm: &Arc<PluginManager>) -> gtk::Widget {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 12);
    page.set_margin_start(24); page.set_margin_end(24); page.set_margin_top(24);

    // Directory hint
    let dir_label = gtk::Label::new(Some(L.plugins_dir()));
    dir_label.set_halign(gtk::Align::Start);
    dir_label.style_context().add_class("dim-label");
    dir_label.set_line_wrap(true);
    page.add(&dir_label);

    // Scan button
    let scan_btn = gtk::Button::with_label(L.plugins_scan());
    let pm_scan = pm.clone();
    scan_btn.connect_clicked(move |_| { pm_scan.scan(); });
    page.add(&scan_btn);

    // Plugin list
    let list = pm.list();
    if list.is_empty() {
        let empty = gtk::Label::new(Some(L.plugins_none()));
        empty.set_margin_top(20);
        page.add(&empty);
    } else {
        let count = gtk::Label::new(None);
        count.set_markup(&format!("<b>{}</b>", L.plugins_count().replace("{}", &list.len().to_string())));
        count.set_halign(gtk::Align::Start);
        page.add(&count);

        let scroll = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        let list_box = gtk::Box::new(gtk::Orientation::Vertical, 6);

        for info in &list {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let name = gtk::Label::new(Some(&info.name));
            name.set_halign(gtk::Align::Start);
            row.add(&name);
            row.set_hexpand(true);

            let sw = gtk::Switch::new();
            sw.set_active(info.enabled);
            sw.set_valign(gtk::Align::Center);
            let pm_toggle = pm.clone();
            let name_toggle = info.name.clone();
            sw.connect_state_set(move |_, active| {
                // Toggle is handled by PluginManager::toggle
                gtk::glib::Propagation::Proceed
            });
            row.add(&sw);
            list_box.add(&row);
        }
        scroll.add(&list_box);
        page.add(&scroll);
    }

    page.upcast::<gtk::Widget>()
}
