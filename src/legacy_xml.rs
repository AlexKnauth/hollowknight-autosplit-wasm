
use crate::{
    auto_splitter_settings::{Settings, XMLSettings},
    splits,
    splits::Split,
};

pub fn splits_from_settings(s: &XMLSettings) -> Vec<Split> {
    let maybe_ordered = s.dict_get("Ordered");
    let maybe_start = s.dict_get("AutosplitStartRuns");
    let maybe_end = s.dict_get("AutosplitEndRuns");
    let maybe_splits = s.dict_get("Splits");
    if maybe_ordered.is_some() || maybe_start.is_some() || maybe_end.is_some() {
        // Splits files from up through version 3 of ShootMe/LiveSplit.HollowKnight
        let start = maybe_start.and_then(split_from_settings_str).unwrap_or(Split::StartNewGame);
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

fn splits_from_settings_split_list(s: &XMLSettings) -> Vec<Split> {
    s.as_list().unwrap_or_default().into_iter().filter_map(split_from_settings_split).collect()
}

fn split_from_settings_split(s: XMLSettings) -> Option<Split> {
    split_from_settings_str(s.dict_get("Split").unwrap_or(s))
}

fn split_from_settings_str(s: XMLSettings) -> Option<Split> {
    serde_json::value::from_value(serde_json::Value::String(s.as_string()?)).ok()
}
