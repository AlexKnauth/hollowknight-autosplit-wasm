
use alloc::vec::Vec;

use asr::settings::gui::{add_bool, add_title, set_tooltip, Gui, Widget};

use crate::store::StoreWidget;

use super::args::SetHeadingLevel;
use super::radio_button::{options_str, RadioButton, RadioButtonArgs, RadioButtonOption, RadioButtonOptions};

// --------------------------------------------------------

// #[derive(Gui)]
#[derive(Clone, Default, Eq, Gui, Ord, PartialEq, PartialOrd, RadioButtonOptions)]
pub enum ListItemAction {
    /// None
    #[default]
    None,
    /// Remove
    Remove,
    /// Move before
    MoveBefore,
    /// Move after
    MoveAfter,
    /// Insert before
    InsertBefore,
    /// Insert after
    InsertAfter,
}

impl StoreWidget for ListItemAction {
    fn insert_into(&self, settings_map: &asr::settings::Map, key: &str) -> bool {
        let new_s = options_str(self);
        if settings_map.get(key).is_some_and(|old_v| old_v.get_string().is_some_and(|old_s| old_s == new_s)) {
            return false;
        }
        settings_map.insert(key, new_s);
        true
    }
}

/*
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
*/

// --------------------------------------------------------

#[derive(Clone, Default, SetHeadingLevel)]
#[non_exhaustive]
pub struct UglyListArgs {
    pub heading_level: u32,
}

#[derive(Clone)]
struct UglyListItem<T> {
    item: T,
    action: ListItemAction,
}

impl<T: Widget> Widget for UglyListItem<T> where T::Args: SetHeadingLevel {
    type Args = UglyListArgs;

    fn register(key: &str, description: &str, args: Self::Args) -> Self {
        add_title(key, description, args.heading_level + 1);
        let key_item = format!("{}_item", key);
        let mut t_args = T::Args::default();
        t_args.set_heading_level(args.heading_level + 2);
        let item = T::register(&key_item, "", t_args);
        let key_action = format!("{}_action", key);
        let mut rb_args = <ListItemAction as Widget>::Args::default();
        rb_args.set_heading_level(args.heading_level + 2);
        let action = ListItemAction::register(&key_action, "Action", rb_args);
        UglyListItem { item, action }
    }

    fn update_from(&mut self, settings_map: &asr::settings::Map, key: &str, args: Self::Args) {
        let key_item = format!("{}_item", key);
        let mut t_args = T::Args::default();
        t_args.set_heading_level(args.heading_level + 1);
        self.item.update_from(settings_map, &key_item, t_args);
        let key_action = format!("{}_action", key);
        let mut rb_args = <ListItemAction as Widget>::Args::default();
        rb_args.set_heading_level(args.heading_level + 1);
        self.action.update_from(settings_map, &key_action, rb_args);
    }
}

impl<T: StoreWidget> StoreWidget for UglyListItem<T> where T::Args: SetHeadingLevel {
    fn insert_into(&self, settings_map: &asr::settings::Map, key: &str) -> bool {
        let key_item = format!("{}_item", key);
        let a = self.item.insert_into(settings_map, &key_item);
        let key_action = format!("{}_action", key);
        let b = self.action.insert_into(settings_map, &key_action);
        a || b
    }
}

pub struct UglyList<T> {
    len: usize,
    ulis: Vec<UglyListItem<T>>,
}

impl<T> UglyList<T> {
    pub fn get_list(&self) -> Vec<&T> {
        self.ulis[0..self.len].iter().map(|uli| &uli.item).collect()
    }
}

impl<T: Clone + Widget> Widget for UglyList<T> where T::Args: SetHeadingLevel {
    type Args = UglyListArgs;

    fn register(key: &str, description: &str, args: Self::Args) -> Self {
        add_title(key, description, args.heading_level);
        add_bool(&format!("{}_insert_0", key), "Insert at 0", false);
        UglyList { len: 0, ulis: vec![] }
    }

    fn update_from(&mut self, settings_map: &asr::settings::Map, key: &str, args: Self::Args) {
        let map_list: Vec<asr::settings::Value> = settings_map.get(key).and_then(|v| v.get_list()).map(|l| l.iter().collect()).unwrap_or_default();
        let map_len = map_list.len();
        for i in self.ulis.len()..map_len {
            let key_i = format!("{}_{}", key, i);
            self.ulis.push(UglyListItem::register(&key_i, &format!("Item {}", i), args.clone()))
        }
        // --------------------------
        // map_len <= self.ulis.len()
        // --------------------------
        let insert_0 = settings_map.get(&format!("{}_insert_0", key)).and_then(|v| v.get_bool()).unwrap_or(false);
        for i in 0..map_len {
            let key_i = format!("{}_{}", key, i);
            let key_i_item = format!("{}_item", key_i);
            if settings_map.get(&key_i_item).is_none() {
                settings_map.insert(&key_i_item, &map_list[i]);
            }
            self.ulis[i].update_from(settings_map, &key_i, args.clone());
        }
        // --------------------
        // Actions in the Queue
        // --------------------
        let mut index_new_to_old: Vec<i64> = (0 .. (map_len as i64)).collect();
        if insert_0 {
            index_new_to_old.insert(0, -1);
        }
        for old_i in 0 .. map_len {
            let new_i = index_of(&index_new_to_old, &(old_i as i64)).unwrap_or_default();
            match self.ulis[old_i].action {
                ListItemAction::None => (),
                ListItemAction::Remove => { index_new_to_old.remove(new_i); () },
                ListItemAction::InsertBefore => index_new_to_old.insert(new_i, -1),
                ListItemAction::InsertAfter => index_new_to_old.insert(new_i + 1, -1),
                ListItemAction::MoveBefore => if 1 <= new_i { index_new_to_old.swap(new_i, new_i - 1) },
                ListItemAction::MoveAfter => if new_i + 1 < index_new_to_old.len() { index_new_to_old.swap(new_i, new_i + 1) },
            }
        }
        let new_len = index_new_to_old.len();
        for i in self.ulis.len()..new_len {
            let key_i = format!("{}_{}", key, i);
            self.ulis.push(UglyListItem::register(&key_i, &format!("Item {}", i), args.clone()));
        }
        // ---------------
        // Space Allocated
        // ---------------
        let old_items: Vec<T> = self.ulis.iter().map(|uli| uli.item.clone()).collect();
        for (new_i, old_i) in index_new_to_old.into_iter().enumerate() {
            if 0 <= old_i && new_i as i64 != old_i {
                self.ulis[new_i].item = old_items[old_i as usize].clone();
            }
            self.ulis[new_i].action = ListItemAction::None;
        }
        self.len = new_len;
    }
}

impl<T: Clone + StoreWidget> StoreWidget for UglyList<T> where T::Args: SetHeadingLevel {
    fn insert_into(&self, settings_map: &asr::settings::Map, key: &str) -> bool {
        let mut changed = false;
        let key_insert_0 = &format!("{}_insert_0", key);
        if !settings_map.get(&key_insert_0).is_some_and(|v| v.get_bool().is_some_and(|b| !b)) {
            settings_map.insert(&key_insert_0, false);
            changed = true;
        }
        let old_len = settings_map.get(key).and_then(|old_v| old_v.get_list()).map(|old_list| old_list.len()).unwrap_or(0);
        let new_list = asr::settings::List::new();
        for i in 0..self.len {
            let key_i = format!("{}_{}", key, i);
            let key_i_item = format!("{}_item", key_i);
            changed = self.ulis[i].insert_into(&settings_map, &key_i) || changed;
            let new_v = settings_map.get(&key_i_item).unwrap_or(false.into());
            new_list.push(&new_v);
            if old_len != self.len as u64 {
                set_tooltip(&key_i, &format!("Item exists: {} < {}\n{:?}", i, self.len, new_v));
            }
        }
        if old_len != self.len as u64 {
            changed = true;
            for i in self.len..self.ulis.len() {
                let key_i = format!("{}_{}", key, i);
                set_tooltip(&key_i, &format!("DOES NOT EXIST"));
            }
        }
        if changed {
            settings_map.insert(key, &new_list);
            set_tooltip(key, &format!("{:?}", new_list));    
        }
        changed
    }
}

// --------------------------------------------------------

fn index_of<T>(slice: &[T], v: &T) -> Option<usize> where T: PartialEq<T> {
    for (i, e) in slice.into_iter().enumerate() {
        if e == v {
            return Some(i);
        }
    }
    None
}
