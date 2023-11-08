use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use xmltree::{XMLNode, Element};

pub trait Settings: Clone {
    fn as_str(&self) -> Option<&str>;
    fn as_bool(&self) -> Option<bool>;
    fn as_list(&self) -> Option<Vec<Self>>;
    fn as_dict(&self) -> Option<BTreeMap<String, Self>>;

    fn list_get(&self, index: usize) -> Option<Self> {
        self.as_list()?.get(index).cloned()
    }
    fn dict_get(&self, key: &str) -> Option<Self> {
        self.as_dict()?.get(key).cloned()
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
    fn as_str(&self) -> Option<&str> {
        match &self.children[..] {
            [] => Some(""),
            [XMLNode::Text(s)] => Some(s),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self.as_str()?.trim() {
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

    fn as_dict(&self) -> Option<BTreeMap<String, Self>> {
        let l = self.as_list()?;
        Some(BTreeMap::from_iter(l.into_iter().filter_map(|c| -> Option<(String, XMLSettings)> {
            let e = c.children.first()?.as_element()?;
            Some((e.name.clone(), XMLSettings { children: e.children.clone() }))
        })))
    }
}
