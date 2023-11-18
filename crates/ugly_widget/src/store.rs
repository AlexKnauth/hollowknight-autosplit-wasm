
use asr::settings::gui::{Gui, Widget};

pub trait StoreGui: Gui {
    fn insert_into(&self, settings_map: &asr::settings::Map);

    fn load_update_store_if_unchanged(&mut self) -> bool {
        let settings_map = asr::settings::Map::load();
        let old = settings_map.clone();
        self.update_from(&settings_map);
        self.insert_into(&settings_map);
        settings_map.store_if_unchanged(&old)
    }

    fn loop_load_update_store(&mut self) {
        loop {
            if self.load_update_store_if_unchanged() {
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
