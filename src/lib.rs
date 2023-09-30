// #![no_std]

mod hollow_knight_memory;
mod splits;

use asr::{future::next_tick, Process};
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

    let splits: Vec<splits::Split> = serde_json::from_str(include_str!("splits.json")).ok().unwrap_or_else(splits::default_splits);
    let auto_reset = splits::auto_reset_safe(&splits);

    loop {
        let process = wait_attach_hollow_knight().await;
        process
            .until_closes(async {
                // TODO: Load some initial information from the process.
                let mut scene_store = SceneStore::new();
                let mut load_remover = LoadRemover::new();

                next_tick().await;
                let game_manager_finder = GameManagerFinder::wait_attach(&process).await;
                let mut player_data_store = PlayerDataStore::new();

                #[cfg(debug_assertions)]
                asr::print_message(&format!("geo: {:?}", game_manager_finder.get_geo(&process)));

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

                    scene_store.new_curr_scene_name1(gmf.clone());
                    let gmfn = game_manager_finder.get_next_scene_name(&process);
                    scene_store.new_next_scene_name1(gmfn.clone());
                    if let Some(scene_pair) = scene_store.transition_pair() {
                        if splits::transition_splits(current_split, &scene_pair, &process, &game_manager_finder, &mut player_data_store) {
                            split_index(&mut i, n);
                        } else if auto_reset && splits::transition_splits(&splits[0], &scene_pair, &process, &game_manager_finder, &mut player_data_store) {
                            i = 0;
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

                    load_remover.load_removal(&process, &game_manager_finder);

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

struct LoadRemover {
    look_for_teleporting: bool,
    last_game_state: i32,
    #[cfg(debug_assertions)]
    last_paused: bool,
    last_ui_state: Option<i32>,
    last_scene_name: Option<String>,
    last_next_scene: Option<String>,
    last_teleporting: Option<bool>,
    last_hazard_respawning: Option<bool>,
    last_accepting_input: Option<bool>,
    last_hero_transition_state: Option<i32>,
    last_tile_map_dirty: Option<bool>,
}

impl LoadRemover {
    fn new() -> LoadRemover {
        LoadRemover { 
            look_for_teleporting: false,
            last_game_state: GAME_STATE_INACTIVE,
            #[cfg(debug_assertions)]
            last_paused: false,
            last_ui_state: None,
            last_scene_name: None,
            last_next_scene: None,
            last_teleporting: None,
            last_hazard_respawning: None,
            last_accepting_input: None,
            last_hero_transition_state: None,
            last_tile_map_dirty: None,
        }
    }

    fn load_removal(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> Option<()> {

        // only remove loads if timer is running
        if asr::timer::state() != asr::timer::TimerState::Running { return Some(()); }

        let maybe_ui_state = game_manager_finder.get_ui_state(process);
        let ui_state = maybe_ui_state.unwrap_or_default();
        if self.last_ui_state != maybe_ui_state {
            asr::print_message(&format!("ui_state: {:?}", maybe_ui_state));
            self.last_ui_state = maybe_ui_state;
        }

        let maybe_scene_name =  game_manager_finder.get_scene_name(process);
        let scene_name = maybe_scene_name.clone().unwrap_or_default();
        if self.last_scene_name != maybe_scene_name {
            asr::print_message(&format!("scene_name: {:?}", maybe_scene_name));
            self.last_scene_name = maybe_scene_name;
        }
        let maybe_next_scene = game_manager_finder.get_next_scene_name(process);
        if self.last_next_scene != maybe_next_scene {
            asr::print_message(&format!("maybe_next_scene: {:?}", maybe_next_scene));
            self.last_next_scene = maybe_next_scene.clone();
        }
        fn is_none_or_empty(ms: Option<&str>) -> bool {
            match ms {  None | Some("") => true, Some(_) => false }
        }
        let loading_menu = (scene_name != "Menu_Title" && is_none_or_empty(maybe_next_scene.as_deref()))
            || (scene_name != "Menu_Title" && maybe_next_scene.as_deref() == Some("Menu_Title")
                || (scene_name == "Quit_To_Menu"));

        let maybe_teleporting = game_manager_finder.camera_teleporting(process);
        let teleporting = maybe_teleporting.unwrap_or_default();
        if self.last_teleporting != maybe_teleporting {
            asr::print_message(&format!("teleporting: {:?}", maybe_teleporting));
            self.last_teleporting = maybe_teleporting;
        }

        let game_state = game_manager_finder.get_game_state(process)?;
        if game_state == GAME_STATE_PLAYING && self.last_game_state == GAME_STATE_MAIN_MENU {
            self.look_for_teleporting = true;
        }
        if self.look_for_teleporting && (teleporting || (game_state != GAME_STATE_PLAYING && game_state != GAME_STATE_ENTERING_LEVEL)) {
            self.look_for_teleporting = false;
        }

        // TODO: look into Current Patch quitout issues. // might have been fixed? cerpin you broke them in a way that made them work, right?
        let maybe_hazard_respawning = game_manager_finder.hazard_respawning(process);
        let hazard_respawning = maybe_hazard_respawning.unwrap_or_default();
        if self.last_hazard_respawning != maybe_hazard_respawning {
            asr::print_message(&format!("hazard_respawning: {:?}", maybe_hazard_respawning));
            self.last_hazard_respawning = maybe_hazard_respawning;
        }
        let maybe_accepting_input = game_manager_finder.accepting_input(process);
        let accepting_input = maybe_accepting_input.unwrap_or_default();
        if self.last_accepting_input != maybe_accepting_input {
            asr::print_message(&format!("accepting_input: {:?}", maybe_accepting_input));
            self.last_accepting_input = maybe_accepting_input;
        }
        let maybe_hero_transition_state = game_manager_finder.hero_transition_state(process);
        let hero_transition_state = maybe_hero_transition_state.unwrap_or_default();
        if self.last_hero_transition_state != maybe_hero_transition_state {
            asr::print_message(&format!("hero_transition_state: {:?}", maybe_hero_transition_state));
            self.last_hero_transition_state = maybe_hero_transition_state;
        }
        let maybe_tile_map_dirty = game_manager_finder.tile_map_dirty(process);
        let tile_map_dirty = maybe_tile_map_dirty.unwrap_or_default();
        if self.last_tile_map_dirty != maybe_tile_map_dirty {
            asr::print_message(&format!("tile_map_dirty: {:?}", maybe_tile_map_dirty));
            self.last_tile_map_dirty = maybe_tile_map_dirty;
        }
        let uses_scene_transition_routine = game_manager_finder.uses_scene_transition_routine()?;
        let is_game_time_paused =
            (game_state == GAME_STATE_PLAYING && teleporting && !hazard_respawning)
            || (self.look_for_teleporting)
            || ((game_state == GAME_STATE_PLAYING || game_state == GAME_STATE_ENTERING_LEVEL) && ui_state != UI_STATE_PLAYING)
            || (game_state != GAME_STATE_PLAYING && !accepting_input)
            || (game_state == GAME_STATE_EXITING_LEVEL || game_state == GAME_STATE_LOADING)
            || (hero_transition_state == HERO_TRANSITION_STATE_WAITING_TO_ENTER_LEVEL)
            || (ui_state != UI_STATE_PLAYING
                && (loading_menu || (ui_state != UI_STATE_PAUSED && (!is_none_or_empty(maybe_next_scene.as_deref()) || scene_name == "_test_charms")))
                && maybe_next_scene != Some(scene_name))
            || (tile_map_dirty && !uses_scene_transition_routine);
        if is_game_time_paused {
            asr::timer::pause_game_time();
        } else {
            asr::timer::resume_game_time();
        }

        self.last_game_state = game_state;
        #[cfg(debug_assertions)]
        {
            if is_game_time_paused != self.last_paused {
                asr::print_message(&format!("is_game_time_paused: {}", is_game_time_paused));
            }
            self.last_paused = is_game_time_paused;
        }
        Some(())
    }
}
