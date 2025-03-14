use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use asr::future::retry;
use std::path::Path;
use xmltree::{Element, XMLNode};

use crate::{asr_xml, file, legacy_xml};

pub async fn wait_asr_settings_init() -> asr::settings::Map {
    let settings1 = asr::settings::Map::load();
    if settings1
        .get("splits")
        .is_some_and(|v| v.get_list().is_some_and(|l| !l.is_empty()))
    {
        asr::print_message("Settings from asr::settings::Map::load");
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

pub fn asr_settings_from_file<P: AsRef<Path>>(path: P) -> Option<asr::settings::Map> {
    let xml_nodes = file_find_auto_splitter_settings(path)?;
    asr_settings_from_xml_nodes(xml_nodes)
}

fn asr_settings_from_xml_string(xml_string: &str) -> Option<asr::settings::Map> {
    let xml_nodes = Element::parse_all(xml_string.as_bytes()).ok()?;
    asr_settings_from_xml_nodes(xml_nodes)
}

fn asr_settings_from_xml_nodes(xml_nodes: Vec<XMLNode>) -> Option<asr::settings::Map> {
    if any_xml_nodes_from_asr(&xml_nodes) {
        asr_xml::asr_settings_from_xml_nodes(xml_nodes)
    } else {
        legacy_xml::asr_settings_from_xml_nodes(xml_nodes)
    }
}

fn file_find_auto_splitter_settings<P: AsRef<Path>>(path: P) -> Option<Vec<XMLNode>> {
    let bs = file::file_read_all_bytes(path).ok()?;
    let es = Element::parse_all(bs.as_slice()).ok()?;
    let auto_splitter_settings = es.iter().find_map(xml_find_auto_splitter_settings)?;
    Some(auto_splitter_settings)
}

fn xml_find_auto_splitter_settings(xml: &XMLNode) -> Option<Vec<XMLNode>> {
    let e = xml.as_element()?;
    match e.name.as_str() {
        "AutoSplitterSettings" => Some(e.children.clone()),
        "Run" => Some(e.get_child("AutoSplitterSettings")?.children.clone()),
        "Layout" => e
            .get_child("Components")?
            .children
            .iter()
            .find_map(xml_find_auto_splitter_settings),
        "Component" if component_is_asr(e) => Some(e.get_child("Settings")?.children.clone()),
        _ => None,
    }
}

fn component_is_asr(e: &Element) -> bool {
    let Some(p) = e.get_child("Path") else {
        return false;
    };
    let [c] = &p.children[..] else {
        return false;
    };
    let Some(s) = c.as_text() else {
        return false;
    };
    s.contains("LiveSplit.AutoSplittingRuntime")
}

fn any_xml_nodes_from_asr(xml_nodes: &[XMLNode]) -> bool {
    xml_nodes.iter().any(|n| {
        n.as_element()
            .is_some_and(|e| ["Version", "ScriptPath", "CustomSettings"].contains(&e.name.as_str()))
    })
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
