use asr::{settings::gui::{FileSelect, Gui, Title, Widget}, watcher::Pair};

use ugly_widget::{
    ugly_list::{UglyList, UglyListArgs},
    store::{StoreWidget, StoreGui},
    args::SetHeadingLevel,
    radio_button::options_str,
};

use crate::{
    auto_splitter_settings::{wait_asr_settings_load_merge_store, asr_settings_from_file},
    legacy_xml::{splits_from_settings, XMLSettings},
    splits::Split,
};

#[derive(Gui)]
pub struct SettingsGui {
    /// Import Splits
    #[filter((_, "*.lss *.lsl"))]
    import: Pair<FileSelect>,
    /// General Settings
    _general_settings: Title,
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
        let settings1 = asr::settings::Map::load();
        let auto_splitter_settings = include_str!("AutoSplitterSettings.txt");
        let settings2 = XMLSettings::from_xml_string(auto_splitter_settings, &[("Splits", "Split")]).unwrap_or_default();
        let splits2 = splits_from_settings(&settings2);
        if settings1.get("splits").is_some_and(|v| v.get_list().is_some_and(|l| !l.is_empty())) {
            asr::print_message("settings1: from asr::settings::Map::load");
        } else {
            asr::print_message("settings2: from AutoSplitterSettings.txt");
            let settings3 = asr::settings::Map::new();
            let l = asr::settings::List::new();
            for split in splits2.iter() {
                l.push(options_str(split));
            }
            settings3.insert("splits", &l);
            wait_asr_settings_load_merge_store(&settings3).await;
        }
        let mut gui = SettingsGui::register();
        gui.loop_load_update_store();
        let splits1 = gui.get_splits();
        if splits2 != splits1 {
            asr::print_message("WARNING: splits from asr::settings::Map::load differ from AutoSplitterSettings.txt");
                asr::print_message("assuming AutoSplitterSettings.txt is out of date, using asr::settings::Map::load");
        }
        gui
    }
}
