
use alloc::collections::BTreeMap;

use asr::settings::gui::{add_bool, add_title, BoolArgs, set_tooltip, TitleArgs, Widget};

// --------------------------------------------------------

pub trait SetHeadingLevel {
    fn set_heading_level(&mut self, heading_level: u32);
}

impl SetHeadingLevel for TitleArgs {
    fn set_heading_level(&mut self, heading_level: u32) {
        self.heading_level = heading_level;
    }
}

impl SetHeadingLevel for BoolArgs {
    fn set_heading_level(&mut self, _: u32) {
        ()
    }
}

// --------------------------------------------------------

#[derive(Clone)]
pub struct RadioButtonOption<'a, T> {
    pub value: T,
    pub key: &'a str,
    pub description: &'a str,
    pub tooltip: Option<&'a str>,
}

#[derive(Clone, Default)]
#[non_exhaustive]
pub struct RadioButtonArgs<'a> {
    pub heading_level: u32,
    pub default: &'a str,
}

impl SetHeadingLevel for RadioButtonArgs<'_> {
    fn set_heading_level(&mut self, heading_level: u32) {
        self.heading_level = heading_level;
    }
}

pub trait RadioButtonOptions: Clone + Default + Ord {
    fn radio_button_options() -> Vec<RadioButtonOption<'static, Self>>;
}

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
        let new_s = options_str(new);
        settings_map.insert(key, &new_s.into());
        set_tooltip(key, new_s);
        for o in T::radio_button_options() {
            let bool_key = o.bool_key(key);
            let new_b = new == &o.value;
            settings_map.insert(&bool_key, &new_b.into());
        }
        self.0 = new.clone();
        settings_map.store();
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

fn options_str<T: RadioButtonOptions>(v: &T) -> &'static str {
    T::radio_button_options().into_iter().find_map(|o| {
        if &o.value == v {
            Some(o.key)
        } else {
            None
        }
    }).unwrap_or_default()
}

fn options_value<T: RadioButtonOptions>(s: &str) -> Option<T> {
    T::radio_button_options().into_iter().find_map(|o| {
        if o.key == s {
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
