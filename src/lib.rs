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
    hazard_respawning_none: bool,
    accepting_input_none: bool,
    hero_transition_state_none: bool,
    tile_map_dirty_none: bool,
}

impl LoadRemover {
    fn new() -> LoadRemover {
        LoadRemover { 
            look_for_teleporting: false,
            last_game_state: GAME_STATE_INACTIVE,
            #[cfg(debug_assertions)]
            last_paused: false,
            hazard_respawning_none: false,
            accepting_input_none: false,
            hero_transition_state_none: false,
            tile_map_dirty_none: false,
        }
    }

    fn load_removal(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> Option<()> {

        // only remove loads if timer is running
        if asr::timer::state() != asr::timer::TimerState::Running { return Some(()); }

        let ui_state = game_manager_finder.get_ui_state(process)?;

        let scene_name = game_manager_finder.get_scene_name(process)?;
        let maybe_next_scene = game_manager_finder.get_next_scene_name(process);
        fn is_none_or_empty(ms: Option<&str>) -> bool {
            match ms {  None | Some("") => true, Some(_) => false }
        }
        let loading_menu = (scene_name != "Menu_Title" && is_none_or_empty(maybe_next_scene.as_deref()))
            || (scene_name != "Menu_Title" && maybe_next_scene.as_deref() == Some("Menu_Title")
                || (scene_name == "Quit_To_Menu"));

        let teleporting = game_manager_finder.camera_teleporting(process)?;

        let game_state = game_manager_finder.get_game_state(process)?;
        if game_state == GAME_STATE_PLAYING && self.last_game_state == GAME_STATE_MAIN_MENU {
            self.look_for_teleporting = true;
        }
        if self.look_for_teleporting && (teleporting || (game_state != GAME_STATE_PLAYING && game_state != GAME_STATE_ENTERING_LEVEL)) {
            self.look_for_teleporting = false;
        }

        // TODO: look into Current Patch quitout issues. // might have been fixed? cerpin you broke them in a way that made them work, right?
        let hazard_respawning = if let Some(hr) = game_manager_finder.hazard_respawning(process) {
            if self.hazard_respawning_none {
                asr::print_message("hazard_respawning Some");
                self.hazard_respawning_none = false;
            }
            hr
        } else {
            if !self.hazard_respawning_none {
                asr::print_message("hazard_respawning None");
                self.hazard_respawning_none = true;
            }
            bool::default()
        };
        let accepting_input = if let Some(ai) = game_manager_finder.accepting_input(process) {
            if self.accepting_input_none {
                asr::print_message("accepting_input Some");
                self.accepting_input_none = false;
            }
            ai
        } else {
            if !self.accepting_input_none {
                asr::print_message("accepting_input None");
                self.accepting_input_none = true;
            }
            bool::default()
        };
        let hero_transition_state = if let Some(hts) = game_manager_finder.hero_transition_state(process) {
            if self.hero_transition_state_none {
                asr::print_message("hero_transition_state Some");
                self.hero_transition_state_none = false;
            }
            hts
        } else {
            if !self.hero_transition_state_none {
                asr::print_message("hero_transition_state None");
                self.hero_transition_state_none = true;
            }
            i32::default()
        };
        let tile_map_dirty = if let Some(tmd) = game_manager_finder.tile_map_dirty(process) {
            if self.tile_map_dirty_none {
                asr::print_message("tile_map_dirty Some");
                self.tile_map_dirty_none = false;
            }
            tmd
        } else {
            if !self.tile_map_dirty_none {
                asr::print_message("tile_map_dirty None");
                self.tile_map_dirty_none = true;
            }
            bool::default()
        };
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
