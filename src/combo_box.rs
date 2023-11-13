use core::str::FromStr;

use alloc::collections::BTreeMap;

use asr::settings::gui::{add_bool, add_title, Gui, Title, Widget, set_tooltip};

fn single_from_bool_map<'a>(bool_map: &BTreeMap<&'a str, bool>) -> Option<&'a str> {
    let trues: Vec<&str> = bool_map.into_iter().filter_map(|(&k, &v)| {
        if v { Some(k) } else { None }
    }).collect();
    match &trues[..] {
        [t] => Some(t),
        _ => None,
    }
}

#[derive(Clone)]
struct RadioButtonOption<'a> {
    key: &'a str,
    description: &'a str,
    tooltip: Option<&'a str>,
}

impl RadioButtonOption<'_> {
    fn bool_key(&self, key: &str) -> String {
        format!("{}_{}", key, self.key)
    }
}

#[derive(Default)]
#[non_exhaustive]
pub struct RadioButtonOptionsArgs<'a> {
    heading_level: u32,
    default: &'a str,
}

impl RadioButtonOptionsArgs<'_> {
    fn default_value<T: RadioButtonOptions>(&self) -> T {
        T::from_str(self.default).unwrap_or_default()
    }
}

trait RadioButtonOptions: ToString + FromStr + Default {
    fn radio_button_options() -> Vec<RadioButtonOption<'static>>;
}

struct RadioButton<T>(T);

impl<T: RadioButtonOptions> Widget for RadioButton<T> {
    type Args = RadioButtonOptionsArgs<'static>;

    fn register(key: &str, description: &str, args: Self::Args) -> Self {
        add_title(key, description, args.heading_level);
        let default = args.default_value::<T>();
        let default_s = default.to_string();
        let bool_map: BTreeMap<&str, bool> = T::radio_button_options().into_iter().map(|o| {
            let bool_key = o.bool_key(key);
            let b = add_bool(&bool_key, &o.description, o.key == &default_s);
            if let Some(t) = o.tooltip {
                set_tooltip(&bool_key, &t);
            }
            (o.key, b)
        }).collect();
        RadioButton(T::from_str(single_from_bool_map(&bool_map).unwrap_or(&default_s)).unwrap_or(default))
    }

    fn update_from(&mut self, settings_map: &asr::settings::Map, key: &str, args: Self::Args) {
        let default = args.default_value::<T>();
        let default_s = default.to_string();
        let old = settings_map.get(key).and_then(|v| v.get_string()).unwrap_or(default_s.clone());
        let new_bools: Vec<(&str, bool)> = T::radio_button_options().iter().filter_map(|o| {
            let bool_key = o.bool_key(key);
            let old_b = old == o.key;
            let map_b = settings_map.get(&bool_key).and_then(|v| v.get_bool()).unwrap_or_default();
            if map_b != old_b {
                Some((o.key, map_b))
            } else {
                None
            }
        }).collect();
        let new = match &new_bools[..] {
            [(v, true)] => *v,
            [(_, false)] => default_s.as_str(),
            _ => old.as_str(),
        };
        if new != old.as_str() {
            asr::print_message(&new);
        }
        settings_map.insert(key, &new.into());
        for o in T::radio_button_options() {
            let bool_key = o.bool_key(key);
            let new_b = new == o.key;
            settings_map.insert(&bool_key, &new_b.into());
        }
        self.0 = T::from_str(new).unwrap_or_default();
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

impl ToString for ListItemAction {
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
}

impl FromStr for ListItemAction {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "None" => Ok(ListItemAction::None),
            "Remove" => Ok(ListItemAction::Remove),
            "MoveBefore" => Ok(ListItemAction::MoveBefore),
            "MoveAfter" => Ok(ListItemAction::MoveAfter),
            "InsertBefore" => Ok(ListItemAction::InsertBefore),
            "InsertAfter" => Ok(ListItemAction::InsertAfter),
            _ => Err(()),
        }
    }
}

impl RadioButtonOptions for ListItemAction {
    fn radio_button_options() -> Vec<RadioButtonOption<'static>> {
        vec![
            RadioButtonOption { key: "None", description: "None", tooltip: None },
            RadioButtonOption { key: "Remove", description: "Remove", tooltip: None },
            RadioButtonOption { key: "MoveBefore", description: "Move before", tooltip: None },
            RadioButtonOption { key: "MoveAfter", description: "Move after", tooltip: None },
            RadioButtonOption { key: "InsertBefore", description: "Insert before", tooltip: None },
            RadioButtonOption { key: "InsertAfter", description: "Insert after", tooltip: None },
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
