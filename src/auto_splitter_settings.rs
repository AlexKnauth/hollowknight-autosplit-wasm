use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use asr::future::retry;
use ugly_widget::radio_button::options_normalize;
use core::str;
use roxmltree::Node;
#[cfg(not(target_os = "unknown"))]
use std::path::Path;

#[cfg(not(target_os = "unknown"))]
use crate::file;
use crate::{asr_xml, legacy_xml, splits::Split};

pub async fn wait_asr_settings_init() -> asr::settings::Map {
    let settings1 = asr::settings::Map::load();
    if settings1
        .get("splits")
        .is_some_and(|v| v.get_list().is_some_and(|l| !l.is_empty()))
    {
        asr::print_message("Settings from asr::settings::Map::load");
        if let Some(l) = settings1.get("splits").unwrap().get_list() {
            asr::print_message(&l.get(l.len() - 1).unwrap().get_string().unwrap());
        }
        if asr_settings_normalize(&settings1).is_some() {
            asr::print_message("Settings normalized");
            if let Some(l) = settings1.get("splits").unwrap().get_list() {
                asr::print_message(&l.get(l.len() - 1).unwrap().get_string().unwrap());
            }
            settings1.store();
        }
        return settings1;
    }
    if let Some(legacy_raw_xml) = settings1.get("legacy_raw_xml").and_then(|v| v.get_string()) {
        if let Some(settings2) = asr_settings_from_xml_string(&legacy_raw_xml) {
            asr::print_message("Settings from legacy_raw_xml");
            settings2.store();
            return settings2;
        }
    }
    let auto_splitter_settings_txt = include_str!("AutoSplitterSettings.txt");
    if let Some(settings3) = asr_settings_from_xml_string(auto_splitter_settings_txt) {
        asr::print_message("Settings from AutoSplitterSettings.txt");
        return wait_asr_settings_load_merge_store(&settings3).await;
    }
    settings1
}

// --------------------------------------------------------

#[cfg(not(target_os = "unknown"))]
pub fn asr_settings_from_file<P: AsRef<Path>>(path: P) -> Option<asr::settings::Map> {
    let bs = file::file_read_all_bytes(path).ok()?;
    let d = roxmltree::Document::parse(str::from_utf8(bs.as_slice()).ok()?).ok()?;
    let xml_nodes = xml_find_auto_splitter_settings(d.root_element())?;
    let m = asr_settings_from_xml_nodes(xml_nodes)?;
    asr_settings_normalize(&m);
    Some(m)
}

fn asr_settings_from_xml_string(xml_string: &str) -> Option<asr::settings::Map> {
    let wrapped = format!(
        "<AutoSplitterSettings>{}</AutoSplitterSettings>",
        xml_string
    );
    let d = roxmltree::Document::parse(&wrapped).ok()?;
    let xml_nodes = d.root_element().children().collect();
    asr_settings_from_xml_nodes(xml_nodes)
}

fn asr_settings_from_xml_nodes(xml_nodes: Vec<Node>) -> Option<asr::settings::Map> {
    if any_xml_nodes_from_asr(&xml_nodes) {
        asr_xml::asr_settings_from_xml_nodes(xml_nodes)
    } else {
        legacy_xml::asr_settings_from_xml_nodes(xml_nodes)
    }
}

fn asr_settings_normalize(m: &asr::settings::Map) -> Option<()> {
    let old_splits = m.get("splits")?.get_list()?;
    let new_splits = asr::settings::List::new();
    let mut changed = false;
    for (i, old_split) in old_splits.iter().enumerate() {
        let old_string = old_split.get_string()?;
        let new_string = options_normalize::<Split>(&old_string);
        new_splits.push(new_string.as_str());
        if old_string != new_string {
            changed = true;
            m.insert(&format!("splits_{}_item", i), new_string.as_str());
        }
    }
    if changed {
        m.insert("splits", new_splits);
        Some(())
    } else {
        None
    }
}

#[cfg(not(target_os = "unknown"))]
fn xml_find_auto_splitter_settings<'a>(xml: Node<'a, 'a>) -> Option<Vec<Node<'a, 'a>>> {
    if !xml.is_element() {
        return None;
    }
    match xml.tag_name().name() {
        "AutoSplitterSettings" => Some(xml.children().collect()),
        "Run" => Some(
            xml.children()
                .find(|c| c.has_tag_name("AutoSplitterSettings"))?
                .children()
                .collect(),
        ),
        "Layout" => xml
            .children()
            .find(|c| c.has_tag_name("Components"))?
            .children()
            .find_map(xml_find_auto_splitter_settings),
        "Component" if component_is_asr(xml) => Some(
            xml.children()
                .find(|c| c.has_tag_name("Settings"))?
                .children()
                .collect(),
        ),
        _ => None,
    }
}

#[cfg(not(target_os = "unknown"))]
fn component_is_asr(e: Node) -> bool {
    let Some(p) = e.children().find(|c| c.has_tag_name("Path")) else {
        return false;
    };
    let [c] = &p.children().collect::<Vec<_>>()[..] else {
        return false;
    };
    let Some(s) = c.text() else {
        return false;
    };
    s.contains("LiveSplit.AutoSplittingRuntime")
}

fn any_xml_nodes_from_asr(xml_nodes: &[Node]) -> bool {
    xml_nodes
        .iter()
        .any(|n| ["Version", "ScriptPath", "CustomSettings"].contains(&n.tag_name().name()))
}

// --------------------------------------------------------

async fn wait_asr_settings_load_merge_store(new: &asr::settings::Map) -> asr::settings::Map {
    retry(|| asr_settings_load_merge_store(new)).await
}

fn asr_settings_load_merge_store(new: &asr::settings::Map) -> Option<asr::settings::Map> {
    let old = asr::settings::Map::load();
    let merged = maybe_asr_settings_map_merge(Some(old.clone()), new);
    if merged.store_if_unchanged(&old) {
        Some(merged)
    } else {
        None
    }
}

fn maybe_asr_settings_map_merge(
    old: Option<asr::settings::Map>,
    new: &asr::settings::Map,
) -> asr::settings::Map {
    let om = if let Some(om) = old {
        om
    } else {
        asr::settings::Map::new()
    };
    let mut keys: BTreeSet<String> = om.keys().collect();
    keys.extend(new.keys());
    for key in keys {
        if let Some(new_v) = new.get(&key) {
            om.insert(&key, &maybe_asr_settings_value_merge(om.get(&key), &new_v));
        }
    }
    om
}

fn maybe_asr_settings_value_merge(
    old: Option<asr::settings::Value>,
    new: &asr::settings::Value,
) -> asr::settings::Value {
    if let Some(b) = new.get_bool() {
        asr::settings::Value::from(b)
    } else if let Some(s) = new.get_string() {
        asr::settings::Value::from(s.as_str())
    } else if let Some(l) = new.get_list() {
        asr::settings::Value::from(&maybe_asr_settings_list_merge(
            old.and_then(|o| o.get_list()),
            &l,
        ))
    } else if let Some(m) = new.get_map() {
        asr::settings::Value::from(&maybe_asr_settings_map_merge(
            old.and_then(|o| o.get_map()),
            &m,
        ))
    } else {
        new.clone()
    }
}

fn is_asr_settings_list_length(l: &asr::settings::List, n: u64) -> bool {
    l.len() == n
}

fn maybe_asr_settings_list_merge(
    old: Option<asr::settings::List>,
    new: &asr::settings::List,
) -> asr::settings::List {
    let ol = if let Some(ol) = old {
        ol
    } else {
        asr::settings::List::new()
    };
    let nn = new.len();
    if is_asr_settings_list_length(&ol, nn) {
        // same length, merge elements
        let ml = asr::settings::List::new();
        for (i, ne) in new.iter().enumerate() {
            ml.push(&maybe_asr_settings_value_merge(ol.get(i as u64), &ne));
        }
        ml
    } else {
        // different length, replace the whole thing
        let ml = asr::settings::List::new();
        for ne in new.iter() {
            ml.push(&maybe_asr_settings_value_merge(None, &ne));
        }
        ml
    }
}

// --------------------------------------------------------
