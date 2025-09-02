// cargo run --example splits --target aarch64-apple-darwin
// cargo run --example splits --target x86_64-apple-darwin

extern crate asr;
extern crate hollowknight_autosplit_wasm;
extern crate serde_json;
extern crate std;

use hollowknight_autosplit_wasm::splits::Split;
use serde_json::json;
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};
use ugly_widget::radio_button::{RadioButtonOption, RadioButtonOptions};

fn main() -> io::Result<()> {
    let splits_json = Path::new(file!()).parent().unwrap().join("splits.json");

    let j = serde_json::Value::Array(
        Split::radio_button_options()
            .iter()
            .map(|o| {
                let RadioButtonOption {
                    key,
                    description,
                    tooltip,
                    ..
                } = o;
                json!({ "key": key, "description": description, "tooltip": tooltip })
            })
            .collect(),
    );

    let file = File::create(splits_json)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &j)?;
    writer.flush()?;
    Ok(())
}
