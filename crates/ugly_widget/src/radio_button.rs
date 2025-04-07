
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use asr::settings::gui::{add_bool, add_title, set_tooltip, Widget};

pub use ugly_widget_derive::RadioButtonOptions;

use super::args::SetHeadingLevel;
use super::store::StoreWidget;

// --------------------------------------------------------

#[derive(Clone)]
pub struct RadioButtonOption<'a, T> {
    pub value: T,
    pub key: &'a str,
    pub alias: Option<&'a str>,
    pub description: &'a str,
    pub tooltip: Option<&'a str>,
}

#[derive(Clone, Default, SetHeadingLevel)]
#[non_exhaustive]
pub struct RadioButtonArgs<'a> {
    pub heading_level: u32,
    pub default: &'a str,
}

pub trait RadioButtonOptions: Clone + Default + Ord {
    fn radio_button_options() -> Vec<RadioButtonOption<'static, Self>>;
}

#[derive(Clone)]
pub struct RadioButton<T>(pub T);

impl<T: RadioButtonOptions> Widget for RadioButton<T> {
    type Args = RadioButtonArgs<'static>;

    fn register(key: &str, description: &str, args: Self::Args) -> Self {
        add_title(key, description, args.heading_level);
        let default = args.default_value::<T>();
        let bool_map: BTreeMap<T, bool> = T::radio_button_options().into_iter().map(|o| {
            let bool_key = o.bool_key(key);
            let b = add_bool(&bool_key, &o.description, o.value == default);
            if let Some(t) = o.tooltip {
                set_tooltip(&bool_key, &t);
            }
            (o.value, b)
        }).collect();
        RadioButton(single_from_bool_map(&bool_map).cloned().unwrap_or(default))
    }

    fn update_from(&mut self, settings_map: &asr::settings::Map, key: &str, args: Self::Args) {
        let default = args.default_value::<T>();
        let old = settings_map.get(key).and_then(|v| v.get_string()).and_then(|s| options_value::<T>(&s)).unwrap_or(default.clone());
        let options = T::radio_button_options();
        let new_bools: Vec<(&T, bool)> = options.iter().filter_map(|o| {
            let bool_key = o.bool_key(key);
            let old_b = old == o.value;
            let map_b = settings_map.get(&bool_key).and_then(|v| v.get_bool()).unwrap_or(old_b);
            if map_b != old_b {
                Some((&o.value, map_b))
            } else {
                None
            }
        }).collect();
        let new = match &new_bools[..] {
            [(v, true)] => v,
            [(_, false)] => &default,
            _ => &old,
        };
        self.0 = new.clone();
    }
}

impl<T: RadioButtonOptions> StoreWidget for RadioButton<T> {
    fn insert_into(&self, settings_map: &asr::settings::Map, key: &str) -> bool {
        let new_s = options_str(&self.0);
        if settings_map.get(key).is_some_and(|old_v| old_v.get_string().is_some_and(|old_s| old_s == new_s)) {
            return false;
        }
        settings_map.insert(key, new_s);
        set_tooltip(key, new_s);
        for o in T::radio_button_options() {
            let bool_key = o.bool_key(key);
            let new_b = &self.0 == &o.value;
            new_b.insert_into(settings_map, &bool_key);
        }
        true
    }
}

// --------------------------------------------------------

impl<T> RadioButtonOption<'_, T> {
    fn bool_key(&self, key: &str) -> String {
        format!("{}_{}", key, self.key)
    }
}

impl RadioButtonArgs<'_> {
    fn default_value<T: RadioButtonOptions>(&self) -> T {
        options_value::<T>(self.default).unwrap_or_default()
    }
}

pub fn options_str<T: RadioButtonOptions>(v: &T) -> &'static str {
    T::radio_button_options().into_iter().find_map(|o| {
        if &o.value == v {
            Some(o.key)
        } else {
            None
        }
    }).unwrap_or_default()
}

pub fn options_value<T: RadioButtonOptions>(s: &str) -> Option<T> {
    T::radio_button_options().into_iter().find_map(|o| {
        if o.key == s || o.alias.is_some_and(|a| a == s) {
            Some(o.value)
        } else {
            None
        }
    })
}

fn single_from_bool_map<K>(bool_map: &BTreeMap<K, bool>) -> Option<&K> {
    let trues: Vec<&K> = bool_map.into_iter().filter_map(|(k, &v)| {
        if v { Some(k) } else { None }
    }).collect();
    match &trues[..] {
        [t] => Some(t),
        _ => None,
    }
}
