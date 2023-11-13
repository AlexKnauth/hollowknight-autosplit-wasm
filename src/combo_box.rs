use alloc::collections::BTreeMap;

use asr::settings::gui::{add_bool, add_title, Gui, Title, Widget, set_tooltip};

#[derive(Clone)]
struct RadioButtonOption {
    key: String,
    description: String,
    tooltip: Option<String>,
}

impl RadioButtonOption {
    fn bool_key(&self, key: &str) -> String {
        format!("{}_{}", key, self.key)
    }
}

#[derive(Clone, Default)]
pub struct RadioButtonArgs {
    heading_level: u32,
    options: Vec<RadioButtonOption>,
    default: String,
}

pub struct RadioButton(String);

impl RadioButton {
    fn from_bool_map(bool_map: &BTreeMap<String, bool>) -> Option<Self> {
        let trues: Vec<&String> = bool_map.into_iter().filter_map(|(k, &v)| {
            if v { Some(k) } else { None }
        }).collect();
        match &trues[..] {
            [t] => Some(RadioButton(t.to_string())),
            _ => None,
        }
    }
}

impl Widget for RadioButton {
    type Args = RadioButtonArgs;

    fn register(key: &str, description: &str, args: Self::Args) -> Self {
        add_title(key, description, args.heading_level);
        let bool_map: BTreeMap<String, bool> = args.options.into_iter().map(|o| {
            let bool_key = o.bool_key(key);
            let b = add_bool(&bool_key, &o.description, o.key == args.default);
            if let Some(t) = o.tooltip {
                set_tooltip(&bool_key, &t);
            }
            (o.key, b)
        }).collect();
        RadioButton::from_bool_map(&bool_map).unwrap_or(RadioButton(args.default))
    }

    fn update_from(&mut self, settings_map: &asr::settings::Map, key: &str, args: Self::Args) {
        let old = settings_map.get(key).and_then(|v| v.get_string()).unwrap_or(args.default.to_string());
        let new_bools: Vec<(&String, bool)> = args.options.iter().filter_map(|o| {
            let bool_key = o.bool_key(key);
            let old_b = old == o.key;
            let map_b = settings_map.get(&bool_key).and_then(|v| v.get_bool()).unwrap_or_default();
            if map_b != old_b {
                Some((&o.key, map_b))
            } else {
                None
            }
        }).collect();
        let new = match &new_bools[..] {
            [(v, true)] => v.to_string(),
            [(_, false)] => args.default,
            _ => old.to_string(),
        };
        if new != old {
            asr::print_message(&new);
        }
        settings_map.insert(key, &new.as_str().into());
        for o in args.options {
            let bool_key = o.bool_key(key);
            let new_b = new == o.key;
            settings_map.insert(&bool_key, &new_b.into());
        }
        self.0 = new;
        settings_map.store();
    }
}

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

impl ListItemAction {
    fn to_string(&self) -> String {
        match self {
            ListItemAction::None => "None",
            ListItemAction::Remove => "Remove",
            ListItemAction::MoveBefore => "MoveBefore",
            ListItemAction::MoveAfter => "MoveAfter",
            ListItemAction::InsertBefore => "InsertBefore",
            ListItemAction::InsertAfter => "InsertAfter",
        }.to_string()
    }

    fn from_string(s: &str) -> Option<Self> {
        match s {
            "None" => Some(ListItemAction::None),
            "Remove" => Some(ListItemAction::Remove),
            "MoveBefore" => Some(ListItemAction::MoveBefore),
            "MoveAfter" => Some(ListItemAction::MoveAfter),
            "InsertBefore" => Some(ListItemAction::InsertBefore),
            "InsertAfter" => Some(ListItemAction::InsertAfter),
            _ => None,
        }
    }
}

#[derive(Default)]
#[non_exhaustive]
pub struct ListItemActionArgs<'a> {
    heading_level: u32,
    /// The default value of the setting, in case the user didn't set it yet.
    pub default: &'a str,
}

impl ListItemActionArgs<'_> {
    fn default_value(&self) -> ListItemAction {
        ListItemAction::from_string(self.default).unwrap_or_default()
    }
    
    fn radio_button_args(&self) -> RadioButtonArgs {
        RadioButtonArgs {
            heading_level: self.heading_level,
            options: vec![
                RadioButtonOption { key: "None".to_string(), description: "None".to_string(), tooltip: None },
                RadioButtonOption { key: "Remove".to_string(), description: "Remove".to_string(), tooltip: None },
                RadioButtonOption { key: "MoveBefore".to_string(), description: "Move before".to_string(), tooltip: None },
                RadioButtonOption { key: "MoveAfter".to_string(), description: "Move after".to_string(), tooltip: None },
                RadioButtonOption { key: "InsertBefore".to_string(), description: "Insert before".to_string(), tooltip: None },
                RadioButtonOption { key: "InsertAfter".to_string(), description: "Insert after".to_string(), tooltip: None },
            ],
            default: self.default_value().to_string(),
        }
    }
}

impl Widget for ListItemAction {
    type Args = ListItemActionArgs<'static>;

    fn register(key: &str, description: &str, args: Self::Args) -> Self {
        let rb = RadioButton::register(key, description, args.radio_button_args());
        ListItemAction::from_string(&rb.0).unwrap_or_else(|| args.default_value())
    }

    fn update_from(&mut self, settings_map: &asr::settings::Map, key: &str, args: Self::Args) {
        let mut rb = RadioButton(self.to_string());
        rb.update_from(settings_map, key, args.radio_button_args());
        *self =  ListItemAction::from_string(&rb.0).unwrap_or_else(|| args.default_value())
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
    lia: ListItemAction,
}
