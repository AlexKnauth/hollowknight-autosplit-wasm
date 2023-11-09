use alloc::vec::Vec;
use asr::future::retry;
use xmltree::{XMLNode, Element};

pub trait Settings: Sized {
    fn as_string(&self) -> Option<String>;
    fn as_bool(&self) -> Option<bool>;
    fn as_list(&self) -> Option<Vec<Self>>;
    fn dict_get(&self, key: &str) -> Option<Self>;
}

pub enum SettingsObject {
    Map(asr::settings::Map),
    Value(asr::settings::Value),
}

impl SettingsObject {
    pub fn load() -> Self {
        SettingsObject::Map(asr::settings::Map::load())
    }
    fn as_value(&self) -> Option<&asr::settings::Value> {
        let SettingsObject::Value(v) = self else { return None; };
        Some(v)
    }
    fn as_map(&self) -> Option<asr::settings::Map> {
        Some(match self {
            SettingsObject::Map(m) => m.clone(),
            SettingsObject::Value(v) => v.get_map()?,
        })
    }
    pub fn load_merge_store<S: Settings>(new: &S, keys: &[&str], elems: &[&str]) -> Option<SettingsObject> {
        let old = asr::settings::Map::load();
        let merged = maybe_asr_settings_map_merge(Some(old.clone()), new, keys, elems);
        if merged.store_if_unchanged(&old) {
            Some(SettingsObject::Map(merged))
        } else {
            None
        }
    }
    async fn wait_load_merge_store<S: Settings>(new: &S, keys: &[&str], elems: &[&str]) -> SettingsObject {
        retry(|| SettingsObject::load_merge_store(new, keys, elems)).await
    }
}

impl Settings for SettingsObject {
    fn as_string(&self) -> Option<String> {
        self.as_value()?.get_string()
    }

    fn as_bool(&self) -> Option<bool> {
        self.as_value()?.get_bool()
    }

    fn as_list(&self) -> Option<Vec<Self>> {
        let m = self.as_map()?;
        let mut l = Vec::new();
        for i in 0.. {
            let k = i.to_string();
            let Some(v) = m.get(&k) else { break; };
            l.push(SettingsObject::Value(v));
        }
        Some(l)
    }

    fn dict_get(&self, key: &str) -> Option<Self> {
        Some(SettingsObject::Value(self.as_map()?.get(key)?))
    }
}

#[derive(Clone, Debug)]
pub struct XMLSettings {
    children: Vec<XMLNode>,
    is_list: bool,
}

impl Default for XMLSettings {
    fn default() -> Self { XMLSettings { children: vec![], is_list: true } }
}

impl XMLSettings {
    pub fn from_xml_string(s: &str) -> Result<Self, xmltree::ParseError> {
        Ok(XMLSettings { children: Element::parse_all(s.as_bytes())?, is_list: true })
    }
}

impl Settings for XMLSettings {
    fn as_string(&self) -> Option<String> {
        match &self.children[..] {
            [] => Some("".to_string()),
            [XMLNode::Text(s)] => Some(s.to_string()),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self.as_string()?.trim() {
            "True" => Some(true),
            "False" => Some(false),
            _ => None
        }
    }

    fn as_list(&self) -> Option<Vec<Self>> {
        if !self.is_list { return None; }
        Some(self.children.iter().filter_map(|c| {
            if c.as_element().is_some() {
                Some(XMLSettings { 
                    children: vec![c.clone()],
                    is_list: false,
                })
            } else {
                None
            }
        }).collect())
    }

    fn dict_get(&self, key: &str) -> Option<Self> {
        let l = self.as_list()?;
        for c in l.into_iter() {
            let e = c.children.first()?.as_element()?;
            if e.name == key {
                return Some(XMLSettings { children: e.children.clone(), is_list: true });
            }
        }
        None
    }
}

// --------------------------------------------------------

fn maybe_asr_settings_map_merge<S: Settings>(old: Option<asr::settings::Map>, new: &S, keys: &[&str], elems: &[&str]) -> asr::settings::Map {
    let om = if let Some(om) = old { om } else { asr::settings::Map::new() };
    for key in keys {
        if let Some(new_v) = new.dict_get(key) {
            om.insert(key, &maybe_asr_settings_value_merge(om.get(key), &new_v, keys, elems));
        }
    }
    om
}

fn maybe_asr_settings_value_merge<S: Settings>(old: Option<asr::settings::Value>, new: &S, keys: &[&str], elems: &[&str]) -> asr::settings::Value {
    if let Some(b) = new.as_bool() {
        asr::settings::Value::from(b)
    } else if let Some(s) = new.as_string() {
        asr::settings::Value::from(s.as_str())
    } else {
        match new.as_list() {
            Some(l) if new.dict_get("0").is_some() || elems.iter().any(|e| new.dict_get(e).is_some()) => {
                let l2 = l.into_iter().enumerate().map(|(i, v)| {
                    if let Some(v2) = v.dict_get(&i.to_string()) {
                        v2
                    } else if let Some (v2) = elems.iter().find_map(|e| v.dict_get(e)) {
                        v2
                    } else {
                        v
                    }
                }).collect();
                asr::settings::Value::from(&maybe_asr_settings_list_merge(old.and_then(|o| o.get_map()), l2, keys, elems))
            },
            _ => {
                asr::settings::Value::from(&maybe_asr_settings_map_merge(old.and_then(|o| o.get_map()), new, keys, elems))
            },
        }
    }
}

fn is_asr_settings_list_length(m: &asr::settings::Map, n: usize) -> bool {
    m.get(&n.to_string()).is_none() && (n < 1 || m.get(&(n - 1).to_string()).is_some())
}

fn maybe_asr_settings_list_merge<S: Settings>(old: Option<asr::settings::Map>, new: Vec<S>, keys: &[&str], elems: &[&str]) -> asr::settings::Map {
    let om = if let Some(old_m) = old { old_m } else { asr::settings::Map::new() };
    let nn = new.len();
    if is_asr_settings_list_length(&om, nn) {
        // same length, merge elements
        for (i, ne) in new.into_iter().enumerate() {
            let key = i.to_string();
            om.insert(&key, &maybe_asr_settings_value_merge(om.get(&key), &ne, keys, elems));
        }
        om
    } else {
        // different length, replace the whole thing
        let mm = asr::settings::Map::new();
        for (i, ne) in new.into_iter().enumerate() {
            let key = i.to_string();
            mm.insert(&key, &maybe_asr_settings_value_merge(None, &ne, keys, elems));
        }
        mm
    }
}
