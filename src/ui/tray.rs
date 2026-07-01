//! System-tray / StatusNotifierItem indicator (top-right status area).
//!
//! Uses libayatana-appindicator via the `libappindicator` crate so the icon
//! lives in the tray instead of the taskbar. The menu mirrors the right-click
//! menu, driving the same `GAction`s registered on the application.

use crate::locale::L;
use crate::ui::skins;
use gtk::prelude::*;
use libappindicator::{AppIndicator, AppIndicatorStatus};

/// Build the tray indicator. The returned value must be kept alive for the
/// icon to persist (dropping it removes the tray item).
pub fn build(app: &gtk::Application) -> AppIndicator {
    // Ensure the icon PNG exists in its own directory, then point the
    // indicator's icon-theme path there and reference it by name ("icon").
    let _ = crate::ui::icon::ensure_png();
    let dir = crate::ui::icon::png_path()
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let mut indicator = AppIndicator::with_path("linux-monitor", "icon", &dir);
    indicator.set_status(AppIndicatorStatus::Active);
    indicator.set_icon_full("icon", "LinuxMonitor");

    let mut menu = gtk::Menu::new();
    menu.append(&item(app, L.menu_hide(), "toggle-window"));

    let skin_menu = gtk::Menu::new();
    for s in skins::available_skins() {
        skin_menu.append(&item(app, s.display_name(), &format!("skin-{}", s.name())));
    }
    let skin_item = gtk::MenuItem::with_label(L.menu_skins());
    skin_item.set_submenu(Some(&skin_menu));
    menu.append(&skin_item);

    menu.append(&item(app, L.menu_history(), "history"));
    menu.append(&item(app, L.menu_settings(), "settings"));
    menu.append(&gtk::SeparatorMenuItem::new());
    menu.append(&item(app, L.menu_quit(), "quit"));

    menu.show_all();
    indicator.set_menu(&mut menu);
    indicator
}

fn item(app: &gtk::Application, label: &str, action: &str) -> gtk::MenuItem {
    let mi = gtk::MenuItem::with_label(label);
    let a = action.to_string();
    let app = app.clone();
    mi.connect_activate(move |_| app.activate_action(&a, None));
    mi
}
