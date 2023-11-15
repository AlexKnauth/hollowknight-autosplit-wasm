use asr::settings::gui::{Gui, Title};

use ugly_widget::{ugly_list::UglyList, radio_button::RadioButton};

use crate::{splits::{Split, self}, auto_splitter_settings::Settings};

#[derive(Gui)]
pub struct SettingsGui {
    /// General Settings
    _general_settings: Title,
    /// Splits
    #[heading_level = 1]
    splits: UglyList<RadioButton<Split>>,
}


pub fn splits_from_settings<S: Settings>(s: &S) -> Vec<Split> {
    let maybe_ordered = s.dict_get("Ordered");
    let maybe_start = s.dict_get("AutosplitStartRuns");
    let maybe_end = s.dict_get("AutosplitEndRuns");
    let maybe_splits = s.dict_get("Splits");
    if maybe_ordered.is_some() || maybe_start.is_some() || maybe_end.is_some() {
        // Splits files from up through version 3 of ShootMe/LiveSplit.HollowKnight
        let start = maybe_start.and_then(Split::from_settings_str).unwrap_or(Split::StartNewGame);
        let end = maybe_end.and_then(|s| s.as_bool()).unwrap_or_default();
        let mut result = vec![start];
        if let Some(splits) = maybe_splits {
            result.append(&mut splits_from_settings_split_list(&splits));
        }
        if !end {
            result.push(Split::EndingSplit);
        }
        result
    } else if let Some(splits) = maybe_splits {
        // Splits files from after version 4 of mayonnaisical/LiveSplit.HollowKnight
        splits_from_settings_split_list(&splits)
    } else {
        splits::default_splits()
    }
}

fn splits_from_settings_split_list<S: Settings>(s: &S) -> Vec<Split> {
    s.as_list().unwrap_or_default().into_iter().filter_map(Split::from_settings_split).collect()
}
