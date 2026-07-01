pub mod ambient;
pub mod aurora;
pub mod compact;
pub mod draw;
pub mod glass;
pub mod horizontal;
pub mod refined;
pub mod rings;
pub mod template;
pub mod vertical;

use crate::config::AppearanceConfig;
use crate::monitor::SystemSnapshot;

pub trait Skin {
    fn name(&self) -> &str;
    fn display_name(&self) -> &str;
    fn create_widget(&self, config: &AppearanceConfig) -> gtk::Widget;
    fn update(&self, snapshot: &SystemSnapshot, config: &AppearanceConfig);
    /// True when the skin paints its own full card background (e.g. glass).
    /// The host then drops the shared `.monitor-window` card so no dark bezel
    /// shows around the skin's panel.
    fn self_backed(&self) -> bool {
        false
    }
}

pub fn available_skins() -> Vec<Box<dyn Skin>> {
    let mut v: Vec<Box<dyn Skin>> = vec![
        Box::new(horizontal::HorizontalSkin::new()),
        Box::new(vertical::VerticalSkin::new()),
        Box::new(compact::CompactSkin::new()),
        Box::new(refined::RefinedSkin::new()),
        Box::new(rings::RingsSkin::new()),
        Box::new(glass::GlassSkin::new()),
        Box::new(ambient::AmbientSkin::new()),
        Box::new(aurora::AuroraSkin::new()),
    ];
    // Append user-defined declarative skins from ~/.config/linux-monitor/skins/
    for s in template::load_all() {
        v.push(Box::new(s));
    }
    v
}

pub fn find_skin(name: &str) -> Option<Box<dyn Skin>> {
    match name {
        "horizontal" => Some(Box::new(horizontal::HorizontalSkin::new())),
        "vertical" => Some(Box::new(vertical::VerticalSkin::new())),
        "compact" => Some(Box::new(compact::CompactSkin::new())),
        "refined" => Some(Box::new(refined::RefinedSkin::new())),
        "rings" => Some(Box::new(rings::RingsSkin::new())),
        "glass" => Some(Box::new(glass::GlassSkin::new())),
        "ambient" => Some(Box::new(ambient::AmbientSkin::new())),
        "aurora" => Some(Box::new(aurora::AuroraSkin::new())),
        other => other
            .strip_prefix("user-")
            .and_then(template::load_one)
            .map(|s| Box::new(s) as Box<dyn Skin>),
    }
}
