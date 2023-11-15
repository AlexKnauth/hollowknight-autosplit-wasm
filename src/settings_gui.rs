use asr::settings::gui::{Gui, Title};

use crate::ugly_widget::ugly_list::UglyList;

#[derive(Gui)]
pub struct SettingsGui {
    /// General Settings
    _general_settings: Title,
    /// An Ugly List
    #[heading_level = 1]
    aul: UglyList<bool>,
}
