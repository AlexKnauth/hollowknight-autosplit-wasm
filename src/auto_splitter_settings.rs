use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use asr::future::retry;
use std::{path::Path, fs::File, io::{self, Read}};
use xmltree::{Element, XMLNode};

#[derive(Clone, Debug)]
pub struct XMLSettings {
    name: Option<String>,
    children: Vec<XMLNode>,
    list_items: Vec<(String, String)>,
}

impl Default for XMLSettings {
    fn default() -> Self { XMLSettings { name: None, children: vec![], list_items: Vec::new() } }
}

impl XMLSettings {
    pub fn from_file<P: AsRef<Path>>(path: P, list_items: &[(&str, &str)]) -> Option<Self> {
        let list_items = list_items.into_iter().map(|(l, i)| (l.to_string(), i.to_string())).collect();
        let bs = file_read_all_bytes(path).ok()?;
        let es = Element::parse_all(bs.as_slice()).ok()?;
        let auto_splitter_settings = es.iter().find_map(xml_find_auto_splitter_settings)?;
        Some(XMLSettings { name: None, children: auto_splitter_settings, list_items })
    }
    pub fn from_xml_string(s: &str, list_items: &[(&str, &str)]) -> Result<Self, xmltree::ParseError> {
        let list_items = list_items.into_iter().map(|(l, i)| (l.to_string(), i.to_string())).collect();
        Ok(XMLSettings { name: None, children: Element::parse_all(s.as_bytes())?, list_items })
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

    pub fn as_string(&self) -> Option<String> {
        match &self.children[..] {
            [] => Some("".to_string()),
            [XMLNode::Text(s)] => Some(s.to_string()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self.as_string()?.trim() {
            "True" => Some(true),
            "False" => Some(false),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<Vec<Self>> {
        let i = self.is_list_get_item_name()?;
        Some(self.children.iter().filter_map(|c| {
            match c.as_element() {
                Some(e) if e.name == i => {
                    Some(XMLSettings {
                        name: Some(e.name.clone()),
                        children: e.children.clone(),
                        list_items: self.list_items.clone(),
                    })
                },
                _ => None,
            }
        }).collect())
    }

    pub fn dict_get(&self, key: &str) -> Option<Self> {
        for c in self.children.iter() {
            match c.as_element() {
                Some(e) if e.name == key => {
                    return Some(XMLSettings {
                        name: Some(e.name.clone()),
                        children: e.children.clone(),
                        list_items: self.list_items.clone(),
                    });
                },
                _ => (),
            }
        }
        None
    }
}

fn xml_find_auto_splitter_settings(xml: &XMLNode) -> Option<Vec<XMLNode>> {
    let e = xml.as_element()?;
    match e.name.as_str() {
        "AutoSplitterSettings" => Some(e.children.clone()),
        "Run" => Some(e.get_child("AutoSplitterSettings")?.children.clone()),
        "Layout" => e.get_child("Components")?.children.iter().find_map(xml_find_auto_splitter_settings),
        "Component" if component_is_asr(e) => Some(e.get_child("Settings")?.children.clone()),
        _ => None,
    }
}

fn component_is_asr(e: &Element) -> bool {
    let Some(p) = e.get_child("Path") else { return false; };
    let [c] = &p.children[..] else { return false; };
    let Some(s) = c.as_text() else { return false; };
    s.contains("LiveSplit.AutoSplittingRuntime")
}

// --------------------------------------------------------

pub async fn wait_asr_settings_load_merge_store(new: &asr::settings::Map) -> asr::settings::Map {
    retry(|| asr_settings_load_merge_store(new)).await
}

pub fn asr_settings_load_merge_store(new: &asr::settings::Map) -> Option<asr::settings::Map> {
    let old = asr::settings::Map::load();
    let merged = maybe_asr_settings_map_merge(Some(old.clone()), new);
    if merged.store_if_unchanged(&old) {
        Some(merged)
    } else {
        None
    }
}

fn maybe_asr_settings_map_merge(old: Option<asr::settings::Map>, new: &asr::settings::Map) -> asr::settings::Map {
    let om = if let Some(om) = old { om } else { asr::settings::Map::new() };
    let mut keys: BTreeSet<String> = om.keys().collect();
    keys.extend(new.keys());
    for key in keys {
        if let Some(new_v) = new.get(&key) {
            om.insert(&key, &maybe_asr_settings_value_merge(om.get(&key), &new_v));
        }
    }
    om
}

fn maybe_asr_settings_value_merge(old: Option<asr::settings::Value>, new: &asr::settings::Value) -> asr::settings::Value {
    if let Some(b) = new.get_bool() {
        asr::settings::Value::from(b)
    } else if let Some(s) = new.get_string() {
        asr::settings::Value::from(s.as_str())
    } else if let Some(l) = new.get_list() {
        asr::settings::Value::from(&maybe_asr_settings_list_merge(old.and_then(|o| o.get_list()), &l))
    } else if let Some(m) = new.get_map() {
        asr::settings::Value::from(&maybe_asr_settings_map_merge(old.and_then(|o| o.get_map()), &m))
    } else {
        new.clone()
    }
}

fn is_asr_settings_list_length(l: &asr::settings::List, n: u64) -> bool {
    l.len() == n
}

fn maybe_asr_settings_list_merge(old: Option<asr::settings::List>, new: &asr::settings::List) -> asr::settings::List {
    let ol = if let Some(ol) = old { ol } else { asr::settings::List::new() };
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

fn file_read_all_bytes<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut f = File::open(path)?;
    let mut buffer: Vec<u8> = Vec::new();
    f.read_to_end(&mut buffer)?;
    Ok(buffer)
}
