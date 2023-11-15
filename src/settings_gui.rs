use asr::settings::gui::{Gui, Title};

use crate::ugly_widget::ugly_list::UglyList;

#[derive(Gui)]
pub struct ListItemActionGui {
    /// General Settings
    _general_settings: Title,
    /// An Ugly List
    /// 
    /// This is a tooltip.
    #[heading_level = 1]
    aul: UglyList<bool>,
}
