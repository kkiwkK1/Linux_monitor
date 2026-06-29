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

        let css = CssProvider::new();
        css.load_from_data(include_str!("../../assets/style.css").as_bytes()).unwrap();
        gtk::StyleContext::add_provider_for_screen(
            &gtk::gdk::Screen::default().unwrap(), &css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content_box.style_context().add_class("monitor-window");

        let current_skin: Rc<RefCell<Option<Box<dyn Skin>>>> = Rc::new(RefCell::new(None));
        if let Some(skin) = skins::find_skin(&skin_name) {
            content_box.add(&skin.create_widget(&config.borrow().appearance));
            *current_skin.borrow_mut() = Some(skin);
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

        // ── Drag ──
        let wd = window.clone();
        event_box.connect_button_press_event(move |_, ev| {
            if ev.button() == 1 {
                wd.begin_move_drag(1, ev.root().0 as i32, ev.root().1 as i32, ev.time());
            }
            gtk::glib::Propagation::Proceed
        });

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
        // Set visual for proper transparency (fixes white corners on undecorated windows)
        if let Some(screen) = gtk::prelude::GtkWindowExt::screen(&window) {
            if let Some(visual) = screen.rgba_visual() {
                window.set_visual(Some(&visual));
            }
        }

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
    pub fn set_keep_above(&self, ka: bool) { self.window.set_keep_above(ka); }
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

