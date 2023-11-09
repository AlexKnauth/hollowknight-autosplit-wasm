use alloc::vec::Vec;
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
}

impl Default for XMLSettings {
    fn default() -> Self { XMLSettings { children: vec![] } }
}

impl XMLSettings {
    pub fn from_xml_string(s: &str) -> Result<Self, xmltree::ParseError> {
        Ok(XMLSettings { children: Element::parse_all(s.as_bytes())? })
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
        Some(self.children.iter().filter_map(|c| {
            if c.as_element().is_some() {
                Some(XMLSettings { 
                    children: vec![c.clone()]
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
                return Some(XMLSettings { children: e.children.clone() });
            }
        }
        None
    }
}
