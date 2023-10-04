use alloc::vec::Vec;
use asr::user_settings::SettingsObject;

pub fn settings_object_as_list(obj: SettingsObject) -> Option<Vec<SettingsObject>> {
    let n = obj.list_len()?;
    let mut l = Vec::new();
    for i in 0..n {
        l.push(obj.list_get(i)?);
    }
    Some(l)
}
