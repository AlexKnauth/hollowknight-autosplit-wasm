// #![no_std]

mod splits;

use std::collections::BTreeMap;
use std::string::String;
use asr::future::{next_tick, retry};
use asr::Process;
use asr::game_engine::unity::{SceneManager, get_scene_name};
use asr::string::ArrayCString;
// use asr::time::Duration;
// use asr::timer::TimerState;
use asr::watcher::Pair;
use serde::{Deserialize, Serialize};

asr::async_main!(stable);
// asr::panic_handler!();

const SCENE_PATH_SIZE: usize = 64;

const HOLLOW_KNIGHT_NAMES: [&str; 2] = [
    "hollow_knight.exe", // Windows
    "Hollow Knight", // Mac
];

#[derive(Deserialize, Serialize)]
struct SceneInfo {
    name: String,
    path: String
}

async fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        asr::print_message(&panic_info.to_string());
    }));

    // TODO: Set up some general state and settings.

    asr::print_message("Hello, World!");

    loop {
        let process = retry(|| {
            HOLLOW_KNIGHT_NAMES.into_iter().find_map(Process::attach)
        }).await;
        process
            .until_closes(async {
                // TODO: Load some initial information from the process.
                let scene_manager = SceneManager::wait_attach(&process).await;
                let mut scene_name = get_scene_name_string(wait_get_current_scene_path::<SCENE_PATH_SIZE>(&process, &scene_manager).await);
                let mut scene_table: BTreeMap<i32, SceneInfo> = serde_json::from_str(include_str!("scene_table.json")).unwrap_or_default();
                on_scene(&process, &scene_manager, &mut scene_table);

                let splits = splits::default_splits();
                let mut i = 0;
                loop {
                    let current_split = &splits[i];
                    if let Ok(next_scene_name) = scene_manager.get_current_scene_path::<SCENE_PATH_SIZE>(&process).map(get_scene_name_string) {
                        if next_scene_name != scene_name {
                            on_scene(&process, &scene_manager, &mut scene_table);
                            // asr::print_message(&next_scene_name);
                            let scene_pair: Pair<&str> = Pair{old: &scene_name.clone(), current: &next_scene_name.clone()};
                            scene_name = next_scene_name;
                            if splits::transition_splits(current_split, &scene_pair) {
                                if i == 0 {
                                    asr::timer::start();
                                } else {
                                    asr::timer::split();
                                }
                                i += 1;
                                if splits.len() <= i {
                                    i = 0;
                                }
                            }
                        }
                    }
                    next_tick().await;
                }
            })
            .await;
    }
}

async fn wait_get_current_scene_path<const N: usize>(process: &Process, scene_manager: &SceneManager) -> ArrayCString<N> {
    retry(|| scene_manager.get_current_scene_path(&process)).await
}

fn get_scene_name_string<const N: usize>(scene_path: ArrayCString<N>) -> String {
    String::from_utf8(get_scene_name(&scene_path).to_vec()).unwrap()
}

// --------------------------------------------------------

fn log_scene_table(scene_table: &BTreeMap<i32, SceneInfo>) {
    // Log scene_table as json
    if let Ok(j) = serde_json::to_string_pretty(&scene_table) {
        asr::print_message("begin scene_table.json");
        asr::print_message(&j);
    }
}

fn on_scene(process: &Process, scene_manager: &SceneManager, scene_table: &mut BTreeMap<i32, SceneInfo>) {
    let si = scene_manager.get_current_scene_index(&process).unwrap_or(-1);
    let sp: ArrayCString<SCENE_PATH_SIZE> = scene_manager.get_current_scene_path(&process).unwrap_or_default();
    let sn = get_scene_name_string(sp);
    scene_table.insert(si, SceneInfo{name: sn.clone(), path: String::from_utf8(sp.to_vec()).unwrap()});
    if sn == "Menu_Title" {
        log_scene_table(scene_table);
    }
}
