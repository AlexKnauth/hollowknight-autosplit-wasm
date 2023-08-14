// #![no_std]

mod hollow_knight_memory;
mod splits;

use asr::future::next_tick;
use asr::Process;
// use asr::time::Duration;
// use asr::timer::TimerState;
use hollow_knight_memory::*;

#[cfg(debug_assertions)]
use std::string::String;
#[cfg(debug_assertions)]
use asr::string::ArrayCString;
#[cfg(debug_assertions)]
use std::collections::BTreeMap;
#[cfg(debug_assertions)]
use serde::{Deserialize, Serialize};

asr::async_main!(stable);
// asr::panic_handler!();

#[cfg(debug_assertions)]
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct SceneInfo {
    name: String,
    path: String
}

#[cfg(debug_assertions)]
type SceneTable = BTreeMap<i32, SceneInfo>;

async fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        asr::print_message(&panic_info.to_string());
    }));

    // TODO: Set up some general state and settings.

    asr::print_message("Hello, World!");

    #[cfg(debug_assertions)]
    let mut scene_table: SceneTable = serde_json::from_str(include_str!("scene_table.json")).unwrap_or_default();

    let splits: Vec<splits::Split> = serde_json::from_str(include_str!("splits.json")).ok().unwrap_or_else(splits::default_splits);

    loop {
        let process = wait_attach_hollow_knight().await;
        process
            .until_closes(async {
                // TODO: Load some initial information from the process.
                let scene_finder = SceneFinder::wait_attach(&process).await;
                let mut scene_store = SceneStore::new(scene_finder.wait_get_current_scene_name(&process).await);
                #[cfg(debug_assertions)]
                asr::print_message(&scene_store.curr_scene_name);

                next_tick().await;
                let mut game_manager_finder = GameManagerFinder::wait_attach(&process, &scene_finder).await;

                #[cfg(debug_assertions)]
                asr::print_message(&format!("geo: {:?}", game_manager_finder.get_geo(&process)));
                #[cfg(debug_assertions)]
                on_scene(&process, &scene_finder, &mut scene_table);

                let mut i = 0;
                let n = splits.len();
                loop {
                    let current_split = &splits[i];
                    if splits::continuous_splits(current_split, &process, &game_manager_finder) {
                        split_index(&mut i, n);
                        next_tick().await;
                        continue;
                    }
                    scene_store.new_curr_scene_name(check_get_scene_name(&process, &mut game_manager_finder, &scene_finder, &scene_store));
                    scene_store.new_next_scene_name(game_manager_finder.get_next_scene_name(&process));
                    if let Some(scene_pair) = scene_store.transition_pair() {
                        if splits::transition_splits(current_split, &scene_pair) {
                            split_index(&mut i, n);
                        }

                        #[cfg(debug_assertions)]
                        asr::print_message(&format!("{} -> {}", scene_pair.old, scene_pair.current));
                        #[cfg(debug_assertions)]
                        asr::print_message(&format!("fireballLevel: {:?}", game_manager_finder.get_fireball_level(&process)));
                        #[cfg(debug_assertions)]
                        asr::print_message(&format!("geo: {:?}", game_manager_finder.get_geo(&process)));
                        #[cfg(debug_assertions)]
                        on_scene(&process, &scene_finder, &mut scene_table);
                    }
                    game_manager_finder.attempt_clean(&process, &scene_finder).unwrap_or_default();
                    next_tick().await;
                }
            })
            .await;
    }
}

fn split_index(i: &mut usize, n: usize) {
    if *i == 0 {
        asr::timer::start();
    } else {
        asr::timer::split();
    }
    *i += 1;
    if n <= *i {
        *i = 0;
    }
}

fn check_get_scene_name(process: &Process, game_manager_finder: &mut GameManagerFinder, scene_finder: &SceneFinder, scene_store: &SceneStore) -> Option<String> {
    let gmf = game_manager_finder.get_scene_name(&process);
    let sf = scene_finder.get_current_scene_name(&process).ok();
    if sf.is_none() || gmf == sf {
        return gmf
    } else if gmf.is_none() {
        return sf
    }
    let g = gmf.as_ref().unwrap();
    let s = sf.as_ref().unwrap();
    // A is at least as up-to-date as B if: B == prev || (B == curr && A != curr && A != prev)
    if s == &scene_store.prev_scene_name || (s == &scene_store.curr_scene_name && g != &scene_store.curr_scene_name && g != &scene_store.prev_scene_name) {
        gmf
    } else if g == &scene_store.prev_scene_name || (g == &scene_store.curr_scene_name && s != &scene_store.curr_scene_name && s != &scene_store.prev_scene_name) {
        sf
    } else {
        if !game_manager_finder.is_dirty() {
            asr::print_message(&format!("scene name mismatch:\n  SceneFinder: {:?}\n  GameManagerFinder: {:?}", sf, gmf));
        }
        game_manager_finder.set_dirty();
        sf
    }
}

// --------------------------------------------------------

// Logging in debug_assertions mode

#[cfg(debug_assertions)]
fn log_scene_table(scene_table: &BTreeMap<i32, SceneInfo>) {
    // Log scene_table as json
    if let Ok(j) = serde_json::to_string_pretty(&scene_table) {
        asr::print_message(&format!("begin scene_table.json\n{}", j));
    }
}

#[cfg(debug_assertions)]
fn on_scene(process: &Process, scene_finder: &SceneFinder, scene_table: &mut BTreeMap<i32, SceneInfo>) {
    let si = scene_finder.get_current_scene_index(&process).unwrap_or(-1);
    let sp: ArrayCString<SCENE_PATH_SIZE> = scene_finder.get_current_scene_path(&process).unwrap_or_default();
    let sn = scene_path_to_name_string(sp);
    let sv = SceneInfo{name: sn.clone(), path: String::from_utf8(sp.to_vec()).unwrap()};
    if let Some(tv) = scene_table.get(&si) {
        assert_eq!(&sv, tv);
    } else if si == -1 {
        assert_eq!(sv, SceneInfo{name: "".to_string(), path: "".to_string()});
    } else {
        scene_table.insert(si, sv);
        log_scene_table(scene_table);
    }
}
