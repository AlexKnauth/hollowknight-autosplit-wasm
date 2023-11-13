use asr::settings::gui::{Gui, Title};

use super::radio_button::{RadioButton, RadioButtonOption, RadioButtonOptions};

// #[derive(Gui)]
#[derive(Clone, Default, PartialEq)]
pub enum ListItemAction {
    // None
    #[default]
    None,
    // Remove
    Remove,
    // Move before
    MoveBefore,
    // Move after
    MoveAfter,
    // Insert before
    InsertBefore,
    // Insert after
    InsertAfter,
}

impl RadioButtonOptions for ListItemAction {
    fn radio_button_options() -> Vec<RadioButtonOption<'static, Self>> {
        vec![
            RadioButtonOption { value: ListItemAction::None, key: "None", description: "None", tooltip: None },
            RadioButtonOption { value: ListItemAction::Remove, key: "Remove", description: "Remove", tooltip: None },
            RadioButtonOption { value: ListItemAction::MoveBefore, key: "MoveBefore", description: "Move before", tooltip: None },
            RadioButtonOption { value: ListItemAction::MoveAfter, key: "MoveAfter", description: "Move after", tooltip: None },
            RadioButtonOption { value: ListItemAction::InsertBefore, key: "InsertBefore", description: "Insert before", tooltip: None },
            RadioButtonOption { value: ListItemAction::InsertAfter, key: "InsertAfter", description: "Insert after", tooltip: None },
        ]
    }
}

#[derive(Gui)]
pub struct ListItemActionGui {
    /// General Settings
    _general_settings: Title,
    /// Choose an Action
    /// 
    /// This is a tooltip.
    #[heading_level = 1]
    lia: RadioButton<ListItemAction>,
}
