use asr::{settings::gui::{FileSelect, Gui, Title, Widget}, watcher::Pair};

use ugly_widget::{
    ugly_list::{UglyList, UglyListArgs},
    store::{StoreWidget, StoreGui},
    args::SetHeadingLevel,
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
                let mut splits_args = UglyListArgs::default();
                splits_args.set_heading_level(1);
                self.splits.update_from(&settings_map, "splits", splits_args);
                let new_splits2 = self.splits.get_list();
                asr::print_message(&format!("splits: {:?}", new_splits2));
            }
        }
    }

    fn insert_into(&self, settings_map: &asr::settings::Map) -> bool {
        self.splits.insert_into(settings_map, "splits")
    }
}


impl SettingsGui {
    pub fn get_splits(&self) -> Vec<Split> {
        self.splits.get_list().into_iter().map(|rb| rb.clone()).collect()
    }

    pub async fn wait_load_merge_register() -> SettingsGui {
        wait_asr_settings_init().await;
        let mut gui = SettingsGui::register();
        gui.loop_load_update_store();
        gui
    }
}

#[derive(Default, Gui)]
enum TimingMethod {
    /// Load Removed Time
    #[default]
    LoadRemovedTime,
    /// Hits / dream falls
    HitsDreamFalls,
    /// Hits / damage
    HitsDamage,
}
