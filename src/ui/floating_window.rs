use crate::config::AppConfig;
use crate::engine::{MonitorEngine, PollMode};
use crate::locale::L;
use crate::ui::skins::{self, Skin};
use gtk::prelude::*;
use gtk::CssProvider;
use std::cell::RefCell;
use std::rc::Rc;

pub struct FloatingWindow {
    window: gtk::ApplicationWindow,
    content_box: gtk::Box,
    engine: Rc<RefCell<MonitorEngine>>,
    current_skin: Rc<RefCell<Option<Box<dyn Skin>>>>,
    config: Rc<RefCell<AppConfig>>,
}

impl FloatingWindow {
    pub fn new(app: &gtk::Application, engine: Rc<RefCell<MonitorEngine>>, config: Rc<RefCell<AppConfig>>) -> Self {
        let skin_name = config.borrow().appearance.skin.clone();
        let window = gtk::ApplicationWindow::new(app);
        window.set_title(L.app_title());
        window.set_default_size(config.borrow().window.width, config.borrow().window.height);
        window.set_opacity(config.borrow().appearance.opacity.clamp(0.3, 1.0));
        window.set_decorated(false);
        window.set_resizable(true);

        // Transparent top-level so only the rounded card shows (no square white
        // corners). Needs an RGBA visual + a compositor; without one, corners
        // fall back to opaque black rather than white.
        if let Some(screen) = gtk::gdk::Screen::default() {
            if let Some(visual) = screen.rgba_visual() {
                window.set_visual(Some(&visual));
            }
        }
        window.set_app_paintable(true);
        window.connect_draw(|_, cr| {
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            cr.set_operator(gtk::cairo::Operator::Source);
            let _ = cr.paint();
            cr.set_operator(gtk::cairo::Operator::Over);
            gtk::glib::Propagation::Proceed
        });

        let css = CssProvider::new();
        css.load_from_data(include_str!("../../assets/style.css").as_bytes()).unwrap();
        gtk::StyleContext::add_provider_for_screen(
            &gtk::gdk::Screen::default().unwrap(), &css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let current_skin: Rc<RefCell<Option<Box<dyn Skin>>>> = Rc::new(RefCell::new(None));
        // Fall back to the default skin if the configured one no longer exists.
        if let Some(skin) = skins::find_skin(&skin_name).or_else(|| skins::find_skin("horizontal")) {
            set_host_card(&content_box, skin.self_backed());
            content_box.add(&skin.create_widget(&config.borrow().appearance));
            *current_skin.borrow_mut() = Some(skin);
        } else {
            set_host_card(&content_box, false);
        }

        // Wrap in EventBox for reliable mouse capture
        let event_box = gtk::EventBox::new();
        event_box.add(&content_box);
        event_box.set_events(
            gtk::gdk::EventMask::BUTTON_PRESS_MASK
                | gtk::gdk::EventMask::BUTTON_RELEASE_MASK
                | gtk::gdk::EventMask::POINTER_MOTION_MASK,
        );
        window.add(&event_box);

        // ── Manual drag with edge snapping ──
        let drag_state: Rc<RefCell<Option<(f64, f64, i32, i32)>>> = Rc::new(RefCell::new(None));
        {
            let wd = window.clone();
            let ds = drag_state.clone();
            event_box.connect_button_press_event(move |_, ev| {
                if ev.button() == 1 {
                    let (rx, ry) = ev.root();
                    let (wx, wy) = wd.position();
                    *ds.borrow_mut() = Some((rx, ry, wx, wy));
                }
                gtk::glib::Propagation::Proceed
            });
        }
        {
            let wd = window.clone();
            let ds = drag_state.clone();
            event_box.connect_motion_notify_event(move |_, ev| {
                let state = ds.borrow().clone(); // copy, drop borrow
                if let Some((sx, sy, wx, wy)) = state {
                    let (rx, ry) = ev.root();
                    wd.move_(wx + rx as i32 - sx as i32, wy + ry as i32 - sy as i32);
                }
                gtk::glib::Propagation::Proceed
            });
        }
        {
            let wd = window.clone();
            let ds = drag_state.clone();
            event_box.connect_button_release_event(move |_, _ev| {
                let has_drag = ds.borrow().is_some();
                if has_drag {
                    *ds.borrow_mut() = None;
                    snap_to_edge(&wd);
                }
                gtk::glib::Propagation::Proceed
            });
        }

        // ── Right-click ──
        let erc = engine.clone(); let crc = config.clone(); let src = current_skin.clone();
        let bc = content_box.clone(); let win = window.clone();
        event_box.connect_button_press_event(move |_, ev| {
            if ev.button() == 3 { show_menu(&erc, &crc, &src, &bc, &win, ev); }
            gtk::glib::Propagation::Proceed
        });

        // ── Close → hide ──
        let ec = engine.clone();
        window.connect_delete_event(move |w, _| {
            w.hide();
            ec.borrow_mut().set_mode(PollMode::Background);
            gtk::glib::Propagation::Stop
        });

        window.show_all();
        window.set_keep_above(config.borrow().appearance.always_on_top);
        // Hug the skin's natural height instead of a fixed default, so short
        // skins don't leave empty dark space below the content.
        window.resize(config.borrow().window.width.max(1), 1);

        Self { window, content_box, engine, current_skin, config }
    }

    pub fn start_polling(&self) {
        let engine = self.engine.clone(); let skin = self.current_skin.clone(); let config = self.config.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            if engine.borrow().should_poll() {
                let snap = engine.borrow_mut().poll();
                if let Some(ref s) = *skin.borrow() { s.update(&snap, &config.borrow().appearance); }
            }
            gtk::glib::ControlFlow::Continue
        });
    }

    pub fn is_visible(&self) -> bool { self.window.is_visible() }
    pub fn show(&self) { self.window.show_all(); self.window.present(); self.engine.borrow_mut().set_mode(PollMode::Active); }
    pub fn hide(&self) { self.window.hide(); self.engine.borrow_mut().set_mode(PollMode::Background); }
    pub fn set_skin(&self, skin: Box<dyn Skin>) { *self.current_skin.borrow_mut() = Some(skin); }
    /// Switch the host card between the shared `.monitor-window` and a bare
    /// transparent host (for skins that paint their own background).
    pub fn set_host_card(&self, self_backed: bool) { set_host_card(&self.content_box, self_backed); }
    pub fn set_keep_above(&self, ka: bool) { self.window.set_keep_above(ka); }
    pub fn set_opacity(&self, op: f64) { self.window.set_opacity(op); }
    pub fn set_background(&self, appearance: &crate::config::AppearanceConfig) {
        let css = CssProvider::new();
        let bg_css = match appearance.background_type {
            crate::config::BackgroundType::None => {
                let (r, g, b) = (crate::ui::theme::BG.r, crate::ui::theme::BG.g, crate::ui::theme::BG.b);
                format!("background-color: rgba({r}, {g}, {b}, 0.94);")
            }
            crate::config::BackgroundType::Color =>
                format!("background-color: {};", appearance.background_color),
            crate::config::BackgroundType::Image =>
                format!("background-image: url('{}'); background-size: cover;", appearance.background_image),
        };
        let data = format!(".monitor-window {{ {} }}", bg_css);
        if css.load_from_data(data.as_bytes()).is_ok() {
            gtk::StyleContext::add_provider_for_screen(
                &gtk::prelude::GtkWindowExt::screen(&self.window).unwrap(), &css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }
    pub fn get_window(&self) -> &gtk::ApplicationWindow { &self.window }
    pub fn get_content(&self) -> &gtk::Box { &self.content_box }
}

fn show_menu(
    _e: &Rc<RefCell<MonitorEngine>>, _c: &Rc<RefCell<AppConfig>>,
    _s: &Rc<RefCell<Option<Box<dyn Skin>>>>, _w: &gtk::Box,
    window: &gtk::ApplicationWindow, event: &gtk::gdk::EventButton,
) {
    let app = window.application().unwrap();
    let menu = gtk::Menu::new();
    menu.append(&mi(&app, L.menu_hide(), "toggle-window"));
    let sm = gtk::Menu::new();
    for s in skins::available_skins() { sm.append(&mi(&app, s.display_name(), &format!("skin-{}", s.name()))); }
    let si = gtk::MenuItem::with_label(L.menu_skins()); si.set_submenu(Some(&sm)); menu.append(&si);
    menu.append(&mi(&app, L.menu_history(), "history"));
    menu.append(&mi(&app, L.menu_settings(), "settings"));
    menu.append(&gtk::SeparatorMenuItem::new());
    menu.append(&mi(&app, L.menu_quit(), "quit"));
    menu.show_all();
    menu.popup_at_pointer(Some(event));
}

fn mi(app: &gtk::Application, label: &str, action: &str) -> gtk::MenuItem {
    let item = gtk::MenuItem::with_label(label);
    let a = action.to_string();
    let app = app.clone();
    item.connect_activate(move |_| { app.activate_action(&a, None); });
    item
}

/// Toggle the content box between the shared card and a bare transparent host.
fn set_host_card(content_box: &gtk::Box, self_backed: bool) {
    let ctx = content_box.style_context();
    if self_backed {
        ctx.remove_class("monitor-window");
        ctx.add_class("card-bare");
    } else {
        ctx.remove_class("card-bare");
        ctx.add_class("monitor-window");
    }
}

const SNAP_THRESHOLD: i32 = 20;

fn snap_to_edge(window: &gtk::ApplicationWindow) {
    if let Some(screen) = gtk::prelude::GtkWindowExt::screen(window) {
        let (wx, wy) = window.position();
        let (ww, wh) = window.size();
        // Use display's primary monitor geometry
        let display = screen.display();
        let monitor = display.primary_monitor().unwrap();
        let geom = monitor.geometry();
        let mx = geom.x(); let my = geom.y();
        let mw = geom.width(); let mh = geom.height();

        let mut nx = wx;
        let mut ny = wy;

        // Snap to left/right edges
        if (wx - mx).abs() < SNAP_THRESHOLD {
            nx = mx;
        } else if (wx + ww - mx - mw).abs() < SNAP_THRESHOLD {
            nx = mx + mw - ww;
        }
        // Snap to top/bottom edges
        if (wy - my).abs() < SNAP_THRESHOLD {
            ny = my;
        } else if (wy + wh - my - mh).abs() < SNAP_THRESHOLD {
            ny = my + mh - wh;
        }

        if nx != wx || ny != wy {
            window.move_(nx, ny);
        }
    }
}

