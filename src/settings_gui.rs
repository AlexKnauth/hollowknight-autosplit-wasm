use alloc::format;
use alloc::vec;
use alloc::vec::Vec;

use asr::settings::gui::{Gui, Title};
#[cfg(target_os = "wasi")]
use asr::{
    settings::gui::{FileSelect, Widget},
    watcher::Pair,
};

#[cfg(target_os = "wasi")]
use ugly_widget::{args::SetHeadingLevel, ugly_list::UglyListArgs};
use ugly_widget::{
    radio_button::{options_str, RadioButtonOptions},
    store::{StoreGui, StoreWidget},
    ugly_list::UglyList,
};

#[cfg(target_os = "wasi")]
use crate::auto_splitter_settings::asr_settings_from_file;
use crate::{auto_splitter_settings::wait_asr_settings_init, splits::Split};

#[derive(Gui)]
pub struct SettingsGui {
    /// Import Splits
    #[cfg(target_os = "wasi")]
    #[filter((_, "*.lss *.lsl"))]
    import: Pair<FileSelect>,
    /// General Settings
    _general_settings: Title,
    /// Timing Method
    timing_method: TimingMethod,
    /// Hit Counter
    hit_counter: HitsMethod,
    /// Splits
    #[heading_level = 1]
    splits: UglyList<Split>,
}

impl StoreGui for SettingsGui {
    fn post_update(&mut self) {
        #[cfg(target_os = "wasi")]
        if self.import.changed() {
            asr::print_message(&format!("import {}", self.import.current.path));
            if let Some(settings_map) = asr_settings_from_file(&self.import.current.path) {
                let timing_method_args = <TimingMethod as Widget>::Args::default();
                self.timing_method
                    .update_from(&settings_map, "timing_method", timing_method_args);
                let hit_counter_args = <HitsMethod as Widget>::Args::default();
                self.hit_counter
                    .update_from(&settings_map, "hit_counter", hit_counter_args);
                let mut splits_args = UglyListArgs::default();
                splits_args.set_heading_level(1);
                self.splits
                    .update_from(&settings_map, "splits", splits_args);
            }
        }
    }

    fn insert_into(&self, settings_map: &asr::settings::Map) -> bool {
        let a = self
            .timing_method
            .insert_into(settings_map, "timing_method");
        let b = self.hit_counter.insert_into(settings_map, "hit_counter");
        let c = self.splits.insert_into(settings_map, "splits");
        a || b || c
    }
}

impl SettingsGui {
    pub fn get_timing_method(&self) -> TimingMethod {
        self.timing_method
    }
    pub fn get_hit_counter(&self) -> HitsMethod {
        self.hit_counter
    }
    pub fn get_splits(&self) -> Vec<Split> {
        self.splits
            .get_list()
            .into_iter()
            .map(|rb| rb.clone())
            .collect()
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

    pub fn check_hit_counter(&self, hit_counter: &mut HitsMethod) -> Option<HitsMethod> {
        let new_hit_counter = self.get_hit_counter();
        if new_hit_counter != *hit_counter {
            *hit_counter = new_hit_counter;
            asr::print_message(&format!("hit_counter: {:?}", hit_counter));
            Some(new_hit_counter)
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

#[derive(Clone, Copy, Debug, Default, Eq, Gui, Ord, PartialEq, PartialOrd, RadioButtonOptions)]
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
        if settings_map
            .get(key)
            .is_some_and(|old_v| old_v.get_string().is_some_and(|old_s| old_s == new_s))
        {
            return false;
        }
        settings_map.insert(key, new_s);
        true
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Gui, Ord, PartialEq, PartialOrd, RadioButtonOptions)]
pub enum HitsMethod {
    /// None
    #[default]
    None,
    /// Hits / dream falls
    HitsDreamFalls,
    /// Hits / damage
    HitsDamage,
}

impl StoreWidget for HitsMethod {
    fn insert_into(&self, settings_map: &asr::settings::Map, key: &str) -> bool {
        let new_s = options_str(self);
        if settings_map
            .get(key)
            .is_some_and(|old_v| old_v.get_string().is_some_and(|old_s| old_s == new_s))
        {
            return false;
        }
        settings_map.insert(key, new_s);
        true
    }
}
