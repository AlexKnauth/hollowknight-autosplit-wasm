use core::str::FromStr;
use alloc::vec::Vec;

use roxmltree::{Children, Node};

pub fn asr_settings_from_xml_nodes(xml_nodes: Vec<Node>) -> Option<asr::settings::Map> {
    let custom_settings = xml_nodes.into_iter().find_map(xml_node_find_custom_settings)?;
    Some(parse_settings_map(custom_settings))
}

fn xml_node_find_custom_settings<'a>(xml: Node<'a, 'a>) -> Option<Children<'a, 'a>> {
    if !xml.is_element() { return None; }
    if xml.tag_name().name() != "CustomSettings" {
        return None;
    }
    Some(xml.children())
}

fn parse_settings_map<'a>(xml_nodes: Children<'a, 'a>) -> asr::settings::Map {
    let settings_map = asr::settings::Map::new();
    for xml_node in xml_nodes {
        if let (Some(id), Some(value)) = parse_settings_entry(xml_node) {
            settings_map.insert(id, value);
        }
    }
    settings_map
}

fn parse_settings_list(xml_nodes: Children) -> asr::settings::List {
    let settings_list = asr::settings::List::new();
    for xml_node in xml_nodes {
        if let (_, Some(value)) = parse_settings_entry(xml_node) {
            settings_list.push(value);
        }
    }
    settings_list
}

fn parse_settings_entry<'a>(xml_node: Node<'a, 'a>) -> (Option<&'a str>, Option<asr::settings::Value>) {
    if !xml_node.is_element() {
        return (None, None);
    }
    let children = xml_node.children();
    let id = xml_node.attribute("id");
    let Some(setting_type) = xml_node.attribute("type") else {
        return (id, None);
    };
    let value = match setting_type {
        "bool" => Some(parse_bool(children).unwrap_or_default().into()),
        "i64" => Some(parse_fromstr::<i64>(children).unwrap_or_default().into()),
        "f64" => Some(parse_fromstr::<f64>(children).unwrap_or_default().into()),
        "string" => {
            if let Some(string_value) = xml_node.attribute("value") {
                Some(string_value.into())
            } else {
                Some(parse_text(children).unwrap_or_default().into())
            }
        }
        "map" => Some(parse_settings_map(children).into()),
        "list" => Some(parse_settings_list(children).into()),
        _ => None,
    };
    (id, value)
}

fn parse_text<'a>(xml_nodes: Children<'a, 'a>) -> Option<&'a str> {
    match xml_nodes.collect::<Vec<Node>>()[..] {
        [] => Some(""),
        [n] if n.is_text() => n.text(),
        _ => None,
    }
}

fn parse_bool<'a>(xml_nodes: Children<'a, 'a>) -> Option<bool> {
    match parse_text(xml_nodes)?.trim() {
        "True" => Some(true),
        "False" => Some(false),
        _ => None,
    }
}

fn parse_fromstr<'a, F: FromStr>(xml_nodes: Children<'a, 'a>) -> Option<F> {
    parse_text(xml_nodes)?.trim().parse().ok()
}
