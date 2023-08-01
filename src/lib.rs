// #![no_std]

mod splits;

use std::collections::HashMap;
use std::string::String;
use asr::future::{next_tick, retry};
use asr::Process;
use asr::game_engine::unity::{SceneManager, get_scene_name};
use asr::string::ArrayCString;
// use asr::time::Duration;
// use asr::timer::TimerState;
use asr::watcher::Pair;

asr::async_main!(stable);
// asr::panic_handler!();

const SCENE_PATH_SIZE: usize = 64;

const HOLLOW_KNIGHT_NAMES: [&str; 2] = [
    "hollow_knight.exe", // Windows
    "Hollow Knight", // Mac
];

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
                let mut scene_table: HashMap<i32, SceneInfo> = HashMap::new();
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

fn log_scene_table(scene_table: &HashMap<i32, SceneInfo>) {
    // Log scene_table as rows and tab-separated columns
    asr::print_message("begin scene_table");
    let mut ks = scene_table.keys().collect::<Vec<&i32>>();
    ks.sort();
    let wi = 1 + ks.last().unwrap_or(&&0).to_string().len();
    let wn = scene_table.values().map(|s| s.name.len()).max().unwrap_or(1);
    for k in ks.iter() {
        let v = scene_table.get(k).unwrap();
        asr::print_message(&format!("  {1:0$}\t{3:2$}\t{4}", wi,  k, wn, v.name, v.path));
    }
    asr::print_message("end scene_table");
}

fn on_scene(process: &Process, scene_manager: &SceneManager, scene_table: &mut HashMap<i32, SceneInfo>) {
    let si = scene_manager.get_current_scene_index(&process).unwrap_or(-1);
    let sp: ArrayCString<SCENE_PATH_SIZE> = scene_manager.get_current_scene_path(&process).unwrap_or_default();
    let sn = get_scene_name_string(sp);
    scene_table.insert(si, SceneInfo{name: sn.clone(), path: String::from_utf8(sp.to_vec()).unwrap()});
    if sn == "Menu_Title" {
        log_scene_table(scene_table);
    }
}
