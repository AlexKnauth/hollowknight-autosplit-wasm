
use core::cmp::max;

use alloc::vec::Vec;

use asr::settings::gui::{add_bool, add_title, set_tooltip, Widget};

use crate::radio_button::SetHeadingLevel;

// --------------------------------------------------------

#[derive(Default)]
#[non_exhaustive]
pub struct UglyListArgs {
    pub heading_level: u32,
}

impl SetHeadingLevel for UglyListArgs {
    fn set_heading_level(&mut self, heading_level: u32) {
        self.heading_level = heading_level;
    }
}

pub struct UglyList<T>(Vec<T>);

impl<T: Widget> Widget for UglyList<T> where T::Args: SetHeadingLevel {
    type Args = UglyListArgs;

    fn register(key: &str, description: &str, args: Self::Args) -> Self {
        add_title(key, description, args.heading_level);
        add_bool(&format!("{}_insert_0", key), "Insert 0", false);
        UglyList(vec![])
    }

    fn update_from(&mut self, settings_map: &asr::settings::Map, key: &str, args: Self::Args) {
        let len_old = settings_map.get(&format!("{}_len", key)).and_then(|v| v.get_i64()).unwrap_or(0);
        let cap_old = settings_map.get(&format!("{}_cap", key)).and_then(|v| v.get_i64()).unwrap_or(0);
        let insert_0 = settings_map.get(&format!("{}_insert_0", key)).and_then(|v| v.get_bool()).unwrap_or(false);
        let len_new = if insert_0 {
            asr::print_message("insert_0");
            len_old + 1
        } else {
            len_old
        };
        let cap_new = max(cap_old, len_new);
        for i in cap_old..cap_new {
            add_title(&format!("{}_{}", key, i), &format!("Item {}", i), args.heading_level + 1);
            let mut t_args = T::Args::default();
            t_args.set_heading_level(args.heading_level + 2);
            T::register(&format!("{}_{}_item", key, i), "", t_args);
        }
        settings_map.insert(&format!("{}_len", key), &len_new.into());
        settings_map.insert(&format!("{}_cap", key), &cap_new.into());
        settings_map.insert(&format!("{}_insert_0", key), &false.into());
        settings_map.store();
    }
}
