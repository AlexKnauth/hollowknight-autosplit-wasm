// #![no_std]

mod hollow_knight_memory;
mod splits;

use asr::future::next_tick;
// use asr::time::Duration;
// use asr::timer::TimerState;
use hollow_knight_memory::*;

asr::async_main!(stable);
// asr::panic_handler!();

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

                next_tick().await;
                let mut game_manager_finder = GameManagerFinder::wait_attach(&process, &scene_finder).await;
                let mut player_data_store = PlayerDataStore::new();

                #[cfg(debug_assertions)]
                asr::print_message(&format!("geo: {:?}", game_manager_finder.get_geo(&process)));
                #[cfg(debug_assertions)]
                update_scene_table(&process, &scene_finder, &mut scene_table);

                let mut i = 0;
                let n = splits.len();
                loop {
                    let current_split = &splits[i];
                    if splits::continuous_splits(current_split, &process, &game_manager_finder, &mut player_data_store) {
                        split_index(&mut i, n);
                        next_tick().await;
                        continue;
                    }
                    let gmf = game_manager_finder.get_scene_name(&process);
                    let sf = scene_finder.get_current_scene_name(&process).ok();
                    #[cfg(debug_assertions)]
                    let new_curr_scene = sf.as_ref().is_some_and(|s| s != scene_store.curr_scene_name());

                    let (gmf_dirty, sf_dirty) = scene_store.new_curr_scene_name2(gmf.clone(), sf.clone());
                    if gmf_dirty && !game_manager_finder.is_dirty() {
                        asr::print_message(&format!("GameManagerFinder dirty:\n  SceneFinder: {:?}\n  GameManagerFinder: {:?}", sf, gmf));
                        game_manager_finder.set_dirty();
                    }
                    if sf_dirty {
                        asr::print_message(&format!("SceneFinder dirty:\n  SceneFinder: {:?}\n  GameManagerFinder: {:?}", sf, gmf));
                    }
                    let gmfn = game_manager_finder.get_next_scene_name(&process);
                    let gmfn_dirty = scene_store.new_next_scene_name1(gmfn.clone());
                    if gmfn_dirty && !game_manager_finder.is_dirty() {
                        asr::print_message(&format!("GameManagerFinder dirty next_scene_name: {:?}", gmfn));
                        game_manager_finder.set_dirty();
                    }
                    if let Some(scene_pair) = scene_store.transition_pair() {
                        if splits::transition_splits(current_split, &scene_pair, &process, &game_manager_finder, &mut player_data_store) {
                            split_index(&mut i, n);
                        }

                        if scene_pair.old == MENU_TITLE {
                            player_data_store.reset();
                        }

                        #[cfg(debug_assertions)]
                        asr::print_message(&format!("{} -> {}", scene_pair.old, scene_pair.current));
                        #[cfg(debug_assertions)]
                        asr::print_message(&format!("fireballLevel: {:?}", game_manager_finder.get_fireball_level(&process)));
                        #[cfg(debug_assertions)]
                        asr::print_message(&format!("geo: {:?}", game_manager_finder.get_geo(&process)));
                    }
                    #[cfg(debug_assertions)]
                    if new_curr_scene {
                        update_scene_table(&process, &scene_finder, &mut scene_table);
                    }
                    let gs = game_manager_finder.get_game_state(&process);
                    if gmf.is_none() && gmfn.is_none() && !game_manager_finder.is_dirty() && sf.is_some_and(|s| is_play_scene(&s)) {
                        asr::print_message(&format!("GameManagerFinder not found: game state {:?}", gs));
                        game_manager_finder.set_dirty();
                    }
                    game_manager_finder.attempt_clean(&process, &scene_finder).await.unwrap_or_default();
                    next_tick().await;
                }
            })
            .await;
    }
}

fn split_index(i: &mut usize, n: usize) {
    if *i == 0 {
        asr::timer::reset();
        asr::timer::start();
    } else {
        asr::timer::split();
    }
    *i += 1;
    if n <= *i {
        *i = 0;
    }
}
