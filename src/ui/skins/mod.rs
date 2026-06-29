pub mod compact;
pub mod horizontal;
pub mod vertical;

use crate::config::AppearanceConfig;
use crate::monitor::SystemSnapshot;

pub trait Skin {
    fn name(&self) -> &str;
    fn display_name(&self) -> &str;
    fn create_widget(&self, config: &AppearanceConfig) -> gtk::Widget;
    fn update(&self, snapshot: &SystemSnapshot, config: &AppearanceConfig);
}

pub fn available_skins() -> Vec<Box<dyn Skin>> {
    vec![
        Box::new(horizontal::HorizontalSkin::new()),
        Box::new(vertical::VerticalSkin::new()),
        Box::new(compact::CompactSkin::new()),
    ]
}

pub fn find_skin(name: &str) -> Option<Box<dyn Skin>> {
    match name {
        "horizontal" => Some(Box::new(horizontal::HorizontalSkin::new())),
        "vertical" => Some(Box::new(vertical::VerticalSkin::new())),
        "compact" => Some(Box::new(compact::CompactSkin::new())),
        _ => None,
    }
}
