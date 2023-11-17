
use asr::settings::gui::{Gui, Widget};

pub trait StoreGui: Gui {
    fn insert_into(&self, settings_map: &asr::settings::Map);

    fn loop_load_update_store(&mut self) {
        loop {
            let settings_map = asr::settings::Map::load();
            let old = settings_map.clone();
            self.update_from(&settings_map);
            self.insert_into(&settings_map);
            if settings_map.store_if_unchanged(&old) {
                break;
            }
        }
    }
}

pub trait StoreWidget: Widget {
    fn insert_into(&self, settings_map: &asr::settings::Map, key: &str);
}

impl StoreWidget for bool {
    fn insert_into(&self, settings_map: &asr::settings::Map, key: &str) {
        settings_map.insert(key, &(*self).into());
    }
}
