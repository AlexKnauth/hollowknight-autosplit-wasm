
use asr::settings::gui::{Gui, Widget};

pub trait StoreGui: Gui {
    fn loop_load_store(&self);
}

pub trait StoreWidget: Widget {
    fn insert_into(&self, settings_map: &asr::settings::Map, key: &str);

    fn loop_load_store(&self, key: &str) {
        loop {
            let settings_map = asr::settings::Map::load();
            let old = settings_map.clone();
            self.insert_into(&settings_map, key);
            if settings_map.store_if_unchanged(&old) {
                break;
            }
        }
    }
}

impl StoreWidget for bool {
    fn insert_into(&self, settings_map: &asr::settings::Map, key: &str) {
        settings_map.insert(key, &(*self).into());
    }
}
