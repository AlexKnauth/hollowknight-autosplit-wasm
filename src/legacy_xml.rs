use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use ugly_widget::radio_button::{options_str, options_value};
use roxmltree::Node;

use crate::{
    settings_gui::{HitsMethod, TimingMethod},
    splits::Split,
};

pub fn asr_settings_from_xml_nodes(xml_nodes: Vec<Node>) -> Option<asr::settings::Map> {
    let xml_settings = XMLSettings::from_xml_nodes(xml_nodes, &[("Splits", "Split")]);
    let splits = splits_from_settings(&xml_settings)?;
    // new empty map, which will only include the new splits
    let settings_map = asr::settings::Map::new();
    settings_map.insert("splits", asr_list_from_splits(&splits));
    if let Some(timing_method) = xml_settings.dict_get("TimingMethod") {
        let tm = timing_method_from_settings_str(timing_method).unwrap_or_default();
        settings_map.insert("timing_method", options_str(&tm));
    }
    if let Some(hit_counter) = xml_settings.dict_get("HitCounter") {
        let hm = hits_method_from_settings_str(hit_counter).unwrap_or_default();
        settings_map.insert("hit_counter", options_str(&hm));
    }
    Some(settings_map)
}

fn asr_list_from_splits(splits: &[Split]) -> asr::settings::List {
    let l = asr::settings::List::new();
    for split in splits.iter() {
        l.push(options_str(split));
    }
    l
}

#[derive(Clone, Debug)]
struct XMLSettings<'a> {
    name: Option<String>,
    children: Vec<Node<'a, 'a>>,
    list_items: Vec<(String, String)>,
}

impl<'a> Default for XMLSettings<'a> {
    fn default() -> Self {
        XMLSettings {
            name: None,
            children: vec![],
            list_items: Vec::new(),
        }
    }
}

impl<'a> XMLSettings<'a> {
    fn from_xml_nodes(children: Vec<Node<'a, 'a>>, list_items: &[(&str, &str)]) -> Self {
        let list_items = list_items
            .into_iter()
            .map(|(l, i)| (l.to_string(), i.to_string()))
            .collect();
        XMLSettings {
            name: None,
            children,
            list_items,
        }
    }

    fn is_list_get_item_name(&self) -> Option<&str> {
        let n = self.name.as_deref()?;
        for (l, i) in self.list_items.iter() {
            if n == l {
                return Some(i);
            }
        }
        None
    }

    fn as_string(&self) -> Option<String> {
        match &self.children[..] {
            [] => Some("".to_string()),
            [n] if n.is_text() => Some(n.text()?.to_string()),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self.as_string()?.trim() {
            "True" => Some(true),
            "False" => Some(false),
            _ => None,
        }
    }

    fn as_list(&self) -> Option<Vec<Self>> {
        let i = self.is_list_get_item_name()?;
        Some(
            self.children
                .iter()
                .filter_map(|c| if c.is_element() && c.has_tag_name(i) {
                    Some(XMLSettings {
                        name: Some(c.tag_name().name().to_string()),
                        children: c.children().collect(),
                        list_items: self.list_items.clone(),
                    })
                } else {
                    None
                })
                .collect(),
        )
    }

    fn dict_get(&self, key: &str) -> Option<Self> {
        for c in self.children.iter() {
            if c.is_element() && c.has_tag_name(key) {
                return Some(XMLSettings {
                    name: Some(c.tag_name().name().to_string()),
                    children: c.children().collect(),
                    list_items: self.list_items.clone(),
                });
            }
        }
        None
    }
}

fn splits_from_settings(s: &XMLSettings) -> Option<Vec<Split>> {
    let maybe_ordered = s.dict_get("Ordered");
    let maybe_start = s.dict_get("AutosplitStartRuns");
    let maybe_end = s.dict_get("AutosplitEndRuns");
    let maybe_splits = s.dict_get("Splits");
    if maybe_ordered.is_some() || maybe_start.is_some() || maybe_end.is_some() {
        // Splits files from up through version 3 of ShootMe/LiveSplit.HollowKnight
        let start = maybe_start
            .and_then(split_from_settings_str)
            .unwrap_or(Split::LegacyStart);
        let end = maybe_end.and_then(|s| s.as_bool()).unwrap_or_default();
        let mut result = vec![start];
        if let Some(splits) = maybe_splits {
            result.append(&mut splits_from_settings_split_list(&splits));
        }
        if !end {
            result.push(Split::EndingSplit);
        }
        Some(result)
    } else if let Some(splits) = maybe_splits {
        // Splits files from after version 4 of mayonnaisical/LiveSplit.HollowKnight
        Some(splits_from_settings_split_list(&splits))
    } else {
        None
    }
}

fn splits_from_settings_split_list(s: &XMLSettings) -> Vec<Split> {
    s.as_list()
        .unwrap_or_default()
        .into_iter()
        .filter_map(split_from_settings_split)
        .collect()
}

fn split_from_settings_split(s: XMLSettings) -> Option<Split> {
    split_from_settings_str(s.dict_get("Split").unwrap_or(s))
}

fn split_from_settings_str(s: XMLSettings) -> Option<Split> {
    options_value(&s.as_string()?)
}

fn timing_method_from_settings_str(s: XMLSettings) -> Option<TimingMethod> {
    options_value(&s.as_string()?)
}

fn hits_method_from_settings_str(s: XMLSettings) -> Option<HitsMethod> {
    options_value(&s.as_string()?)
}
