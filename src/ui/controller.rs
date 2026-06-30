use crate::config::AppConfig;
use crate::engine::MonitorEngine;
use crate::locale::L;
use crate::ui::floating_window::FloatingWindow;
use crate::ui::history_window::HistoryWindow;
use crate::ui::settings::SettingsWindow;
use crate::ui::skins;
use crate::plugin::PluginManager;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub struct AppController {
    floating: Rc<FloatingWindow>,
    config: Rc<RefCell<AppConfig>>,
    app: gtk::Application,
    history_store: Option<Arc<crate::history::store::HistoryStore>>,
    plugin_manager: Option<Arc<PluginManager>>,
}

impl AppController {
    pub fn new(
        app: &gtk::Application, floating: &Rc<FloatingWindow>,
        _engine: Rc<RefCell<MonitorEngine>>, config: Rc<RefCell<AppConfig>>,
        history_store: Option<Arc<crate::history::store::HistoryStore>>,
        plugin_manager: Option<Arc<PluginManager>>,
    ) -> Rc<Self> {
        let ctrl = Rc::new(Self {
            floating: floating.clone(),
            config: config.clone(), app: app.clone(),
            history_store, plugin_manager,
        });

        let c = ctrl.clone();
        let ta = gtk::gio::SimpleAction::new("toggle-window", None);
        ta.connect_activate(move |_, _| c.toggle_window());
        app.add_action(&ta);
        app.set_accels_for_action("app.toggle-window", &["<Control><Shift>T"]);

        let c = ctrl.clone();
        let sa = gtk::gio::SimpleAction::new("settings", None);
        sa.connect_activate(move |_, _| c.show_settings());
        app.add_action(&sa);
        app.set_accels_for_action("app.settings", &["<Control><Shift>S"]);

        if ctrl.history_store.is_some() {
            let c = ctrl.clone();
            let ha = gtk::gio::SimpleAction::new("history", None);
            ha.connect_activate(move |_, _| c.show_history());
            app.add_action(&ha);
            app.set_accels_for_action("app.history", &["<Control><Shift>H"]);
        }

        for skin in skins::available_skins() {
            let sn = skin.name().to_string();
            let c = ctrl.clone();
            let a = gtk::gio::SimpleAction::new(&format!("skin-{}", sn), None);
            a.connect_activate(move |_, _| c.switch_skin(&sn));
            app.add_action(&a);
        }

        ctrl
    }

    pub fn toggle_window(&self) {
        if self.floating.is_visible() { self.floating.hide() } else { self.floating.show() }
    }
    pub fn show_settings(&self) { let _ = SettingsWindow::new(self.config.clone(), self.floating.get_window(), Some(&self.floating), self.plugin_manager.clone()); }
    pub fn show_history(&self) { if let Some(ref s) = self.history_store { let _ = HistoryWindow::new(s.clone(), self.floating.get_window()); } }

    pub fn switch_skin(&self, skin_name: &str) {
        self.config.borrow_mut().appearance.skin = skin_name.to_string();
        if let Some(skin) = skins::find_skin(skin_name) {
            let w = skin.create_widget(&self.config.borrow().appearance);
            let content = self.floating.get_content();
            let children = content.children();
            for child in &children { content.remove(child); }
            content.add(&w);
            content.show_all();
            self.floating.set_skin(skin);
        }
        // Resize: compact gets slim height
        let (w, h) = if skin_name == "compact" {
            (420, 24)
        } else {
            (self.config.borrow().window.width, self.config.borrow().window.height)
        };
        self.floating.get_window().resize(w, h);
    }
}
