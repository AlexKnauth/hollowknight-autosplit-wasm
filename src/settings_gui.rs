use asr::{settings::gui::{FileSelect, Gui, Title, Widget}, watcher::Pair};

use serde::{Deserialize, Serialize};
use ugly_widget::{
    args::SetHeadingLevel,
    radio_button::{options_str, RadioButtonOptions},
    store::{StoreGui, StoreWidget},
    ugly_list::{UglyList, UglyListArgs}
};

use crate::{
    auto_splitter_settings::{asr_settings_from_file, wait_asr_settings_init},
    splits::Split,
};

#[derive(Gui)]
pub struct SettingsGui {
    /// Import Splits
    #[filter((_, "*.lss *.lsl"))]
    import: Pair<FileSelect>,
    /// General Settings
    _general_settings: Title,
    /// Timing Method
    timing_method: TimingMethod,
    /// Splits
    #[heading_level = 1]
    splits: UglyList<Split>,
}

impl StoreGui for SettingsGui {
    fn post_update(&mut self) {
        if self.import.changed() {
            asr::print_message(&format!("import {}", self.import.current.path));
            if let Some(settings_map) = asr_settings_from_file(&self.import.current.path) {
                let timing_method_args = <TimingMethod as Widget>::Args::default();
                self.timing_method.update_from(&settings_map, "timing_method", timing_method_args);
                let mut splits_args = UglyListArgs::default();
                splits_args.set_heading_level(1);
                self.splits.update_from(&settings_map, "splits", splits_args);
            }
        }
    }

    fn insert_into(&self, settings_map: &asr::settings::Map) -> bool {
        let a = self.timing_method.insert_into(settings_map, "timing_method");
        let b = self.splits.insert_into(settings_map, "splits");
        a || b
    }
}


impl SettingsGui {
    pub fn get_timing_method(&self) -> TimingMethod {
        self.timing_method
    }
    pub fn get_splits(&self) -> Vec<Split> {
        self.splits.get_list().into_iter().map(|rb| rb.clone()).collect()
    }

    pub async fn wait_load_merge_register() -> SettingsGui {
        wait_asr_settings_init().await;
        let mut gui = SettingsGui::register();
        gui.loop_load_update_store();
        gui
    }

    pub fn check_timing_method(&self, timing_method: &mut TimingMethod) -> Option<TimingMethod> {
        let new_timing_method = self.get_timing_method();
        if new_timing_method != *timing_method {
            *timing_method = new_timing_method;
            asr::print_message(&format!("timing_method: {:?}", timing_method));
            Some(new_timing_method)
        } else {
            None
        }
    }

    pub fn check_splits<'a>(&self, splits: &'a mut Vec<Split>) -> Option<&'a [Split]> {
        let new_splits = self.get_splits();
        if new_splits != *splits {
            *splits = new_splits;
            asr::print_message(&format!("splits: {:?}", splits));
            Some(splits)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Gui, Ord, PartialEq, PartialOrd, RadioButtonOptions, Serialize)]
pub enum TimingMethod {
    /// Load Removed Time
    #[default]
    LoadRemovedTime,
    /// Hits / dream falls
    HitsDreamFalls,
    /// Hits / damage
    HitsDamage,
}

impl StoreWidget for TimingMethod {
    fn insert_into(&self, settings_map: &asr::settings::Map, key: &str) -> bool {
        let new_s = options_str(self);
        if settings_map.get(key).is_some_and(|old_v| old_v.get_string().is_some_and(|old_s| old_s == new_s)) {
            return false;
        }
        settings_map.insert(key, new_s);
        true
    }
}
