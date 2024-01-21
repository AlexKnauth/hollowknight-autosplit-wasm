
use xmltree::XMLNode;

use crate::splits::Split;

#[derive(Clone, Debug)]
struct XMLSettings {
    name: Option<String>,
    children: Vec<XMLNode>,
    list_items: Vec<(String, String)>,
}

impl Default for XMLSettings {
    fn default() -> Self { XMLSettings { name: None, children: vec![], list_items: Vec::new() } }
}

impl XMLSettings {
    fn from_xml_nodes(children: Vec<XMLNode>, list_items: &[(&str, &str)]) -> Self {
        let list_items = list_items.into_iter().map(|(l, i)| (l.to_string(), i.to_string())).collect();
        XMLSettings { name: None, children, list_items }
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
            [XMLNode::Text(s)] => Some(s.to_string()),
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

    fn dict_get(&self, key: &str) -> Option<Self> {
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

pub fn splits_from_xml_nodes(xml_nodes: Vec<XMLNode>) -> Option<Vec<Split>> {
    let xml_settings = XMLSettings::from_xml_nodes(xml_nodes, &[("Splits", "Split")]);
    splits_from_settings(&xml_settings)
}

fn splits_from_settings(s: &XMLSettings) -> Option<Vec<Split>> {
    let maybe_ordered = s.dict_get("Ordered");
    let maybe_start = s.dict_get("AutosplitStartRuns");
    let maybe_end = s.dict_get("AutosplitEndRuns");
    let maybe_splits = s.dict_get("Splits");
    if maybe_ordered.is_some() || maybe_start.is_some() || maybe_end.is_some() {
        // Splits files from up through version 3 of ShootMe/LiveSplit.HollowKnight
        let start = maybe_start.and_then(split_from_settings_str).unwrap_or(Split::LegacyStart);
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
    s.as_list().unwrap_or_default().into_iter().filter_map(split_from_settings_split).collect()
}

fn split_from_settings_split(s: XMLSettings) -> Option<Split> {
    split_from_settings_str(s.dict_get("Split").unwrap_or(s))
}

fn split_from_settings_str(s: XMLSettings) -> Option<Split> {
    serde_json::value::from_value(serde_json::Value::String(s.as_string()?)).ok()
}
