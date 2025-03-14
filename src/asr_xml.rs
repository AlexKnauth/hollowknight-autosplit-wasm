use std::str::FromStr;

use xmltree::{Element, XMLNode};

pub fn asr_settings_from_xml_nodes(xml_nodes: Vec<XMLNode>) -> Option<asr::settings::Map> {
    let custom_settings = xml_nodes.iter().find_map(xml_node_find_custom_settings)?;
    Some(parse_settings_map(custom_settings))
}

fn xml_node_find_custom_settings(xml: &XMLNode) -> Option<&[XMLNode]> {
    let Element { name, children, .. } = xml.as_element()?;
    if name != "CustomSettings" {
        return None;
    }
    Some(children)
}

fn parse_settings_map(xml_nodes: &[XMLNode]) -> asr::settings::Map {
    let settings_map = asr::settings::Map::new();
    for xml_node in xml_nodes {
        if let (Some(id), Some(value)) = parse_settings_entry(xml_node) {
            settings_map.insert(id, value);
        }
    }
    settings_map
}

fn parse_settings_list(xml_nodes: &[XMLNode]) -> asr::settings::List {
    let settings_list = asr::settings::List::new();
    for xml_node in xml_nodes {
        if let (_, Some(value)) = parse_settings_entry(xml_node) {
            settings_list.push(value);
        }
    }
    settings_list
}

fn parse_settings_entry(xml_node: &XMLNode) -> (Option<&String>, Option<asr::settings::Value>) {
    let Some(Element {
        attributes,
        children,
        ..
    }) = xml_node.as_element()
    else {
        return (None, None);
    };
    let id = attributes.get("id");
    let Some(setting_type) = attributes.get("type") else {
        return (id, None);
    };
    let value = match setting_type.as_str() {
        "bool" => Some(parse_bool(&children).unwrap_or_default().into()),
        "i64" => Some(parse_fromstr::<i64>(&children).unwrap_or_default().into()),
        "f64" => Some(parse_fromstr::<f64>(&children).unwrap_or_default().into()),
        "string" => {
            if let Some(string_value) = attributes.get("value") {
                Some(string_value.as_str().into())
            } else {
                Some(parse_text(&children).unwrap_or_default().into())
            }
        }
        "map" => Some(parse_settings_map(&children).into()),
        "list" => Some(parse_settings_list(&children).into()),
        _ => None,
    };
    (id, value)
}

fn parse_text(xml_nodes: &[XMLNode]) -> Option<&str> {
    match xml_nodes {
        [] => Some(""),
        [XMLNode::Text(s)] => Some(s),
        _ => None,
    }
}

fn parse_bool(xml_nodes: &[XMLNode]) -> Option<bool> {
    match parse_text(xml_nodes)?.trim() {
        "True" => Some(true),
        "False" => Some(false),
        _ => None,
    }
}

fn parse_fromstr<F: FromStr>(xml_nodes: &[XMLNode]) -> Option<F> {
    parse_text(xml_nodes)?.trim().parse().ok()
}
