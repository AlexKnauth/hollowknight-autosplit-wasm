use asr::settings::gui::{add_bool, add_title, Gui, Title, Widget};

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
    fn from_bools(bools: (bool, bool, bool, bool, bool, bool)) -> Self {
        match bools {
            (false, true, false, false, false, false) => ListItemAction::Remove,
            (false, false, true, false, false, false) => ListItemAction::MoveBefore,
            (false, false, false, true, false, false) => ListItemAction::MoveAfter,
            (false, false, false, false, true, false) => ListItemAction::InsertBefore,
            (false, false, false, false, false, true) => ListItemAction::InsertAfter,
            _ => ListItemAction::None,
        }
    }
    fn to_bools(&self) -> (bool, bool, bool, bool, bool, bool) {
        match self {
            ListItemAction::Remove => (false, true, false, false, false, false),
            ListItemAction::MoveBefore => (false, false, true, false, false, false),
            ListItemAction::MoveAfter => (false, false, false, true, false, false),
            ListItemAction::InsertBefore => (false, false, false, false, true, false),
            ListItemAction::InsertAfter => (false, false, false, false, false, true),
            ListItemAction::None => (true, false, false, false, false, false),
        }
    }

    fn bool_keys(key: &str) -> (String, String, String, String, String, String) {
        (
            format!("{}_none", key),
            format!("{}_remove", key),
            format!("{}_move_before", key),
            format!("{}_move_after", key),
            format!("{}_insert_before", key),
            format!("{}_insert_after", key),
        )
    }

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

    fn from_settings_value(v: asr::settings::Value) -> Option<Self> {
        ListItemAction::from_string(v.get_string()?.as_str())
    }
}

#[derive(Default)]
#[non_exhaustive]
pub struct ListItemActionArgs {
    /// The default value of the setting, in case the user didn't set it yet.
    pub default: ListItemAction,
}

impl Widget for ListItemAction {
    type Args = ListItemActionArgs;

    fn register(key: &str, description: &str, args: Self::Args) -> Self {
        add_title(key, description, 1);
        let (key_none, key_remove, key_move_before, key_move_after, key_insert_before, key_insert_after) = ListItemAction::bool_keys(key);
        let is_none = add_bool(&key_none, "None", args.default == ListItemAction::None);
        let is_remove = add_bool(&key_remove, "Remove", args.default == ListItemAction::Remove);
        let is_move_before = add_bool(&key_move_before, "Move before", args.default == ListItemAction::MoveBefore);
        let is_move_after = add_bool(&key_move_after, "Move after", args.default == ListItemAction::MoveAfter);
        let is_insert_before = add_bool(&key_insert_before, "Insert before", args.default == ListItemAction::InsertBefore);
        let is_insert_after = add_bool(&key_insert_after, "Insert after", args.default == ListItemAction::InsertAfter);
        ListItemAction::from_bools((is_none, is_remove, is_move_before, is_move_after, is_insert_before, is_insert_after))
    }

    fn update_from(&mut self, settings_map: &asr::settings::Map, key: &str, args: Self::Args) {
        let (key_none, key_remove, key_move_before, key_move_after, key_insert_before, key_insert_after) = ListItemAction::bool_keys(key);
        let old = settings_map.get(key).and_then(ListItemAction::from_settings_value).unwrap_or(args.default);
        let (old_none, old_remove, old_move_before, old_move_after, old_insert_before, old_insert_after) = old.to_bools();
        let map_none = settings_map.get(&key_none).and_then(|v| v.get_bool()).unwrap_or_default();
        let map_remove = settings_map.get(&key_remove).and_then(|v| v.get_bool()).unwrap_or_default();
        let map_move_before = settings_map.get(&key_move_before).and_then(|v| v.get_bool()).unwrap_or_default();
        let map_move_after = settings_map.get(&key_move_after).and_then(|v| v.get_bool()).unwrap_or_default();
        let map_insert_before = settings_map.get(&key_insert_before).and_then(|v| v.get_bool()).unwrap_or_default();
        let map_insert_after = settings_map.get(&key_insert_after).and_then(|v| v.get_bool()).unwrap_or_default();
        let new1_none = map_none && !old_none;
        let new1_remove = map_remove && !old_remove;
        let new1_move_before = map_move_before && !old_move_before;
        let new1_move_after = map_move_after && !old_move_after;
        let new1_insert_before = map_insert_before && !old_insert_before;
        let new1_insert_after = map_insert_after && !old_insert_after;
        let new = if new1_none || new1_remove || new1_move_before || new1_move_after || new1_insert_before || new1_insert_after {
            ListItemAction::from_bools((new1_none, new1_remove, new1_move_before, new1_move_after, new1_insert_before, new1_insert_after))
        } else {
            old.clone()
        };
        if new != old {
            asr::print_message(&new.to_string());
        }
        let new_string = new.to_string();
        let (new2_none, new2_remove, new2_move_before, new2_move_after, new2_insert_before, new2_insert_after) = new.to_bools();
        *self = new;
        settings_map.insert(key, &new_string.as_str().into());
        settings_map.insert(&key_none, &new2_none.into());
        settings_map.insert(&key_remove, &new2_remove.into());
        settings_map.insert(&key_move_before, &new2_move_before.into());
        settings_map.insert(&key_move_after, &new2_move_after.into());
        settings_map.insert(&key_insert_before, &new2_insert_before.into());
        settings_map.insert(&key_insert_after, &new2_insert_after.into());
        settings_map.store();
    }
}

#[derive(Gui)]
pub struct ListItemActionGui {
    /// General Settings
    _general_settings: Title,
    /// Choose an Action
    /// 
    /// This is a tooltip.
    lia: ListItemAction,
}
