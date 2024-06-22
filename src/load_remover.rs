
use asr::Process;
use asr::timer::TimerState;

use crate::game_time::GameTime;
use crate::hollow_knight_memory::*;
use crate::timer::{Resettable, Timer};

pub struct LoadRemover {
    look_for_teleporting: bool,
    last_game_state: i32,
    #[cfg(debug_assertions)]
    last_paused: bool,
}

impl Resettable for LoadRemover {
    fn reset(&mut self) {}
}

#[allow(unused)]
impl LoadRemover {
    pub fn new() -> LoadRemover {
        LoadRemover { 
            look_for_teleporting: false,
            last_game_state: GAME_STATE_INACTIVE,
            #[cfg(debug_assertions)]
            last_paused: false,
        }
    }
}

impl GameTime for LoadRemover {
    fn update_game_time(&mut self, _: &Timer, process: &Process, game_manager_finder: &GameManagerFinder) {
        // Initialize pointers for load-remover before timer is running
        let maybe_ui_state = game_manager_finder.get_ui_state(process);
        let maybe_scene_name =  game_manager_finder.get_scene_name(process);
        let maybe_next_scene = game_manager_finder.get_next_scene_name(process);
        let maybe_teleporting = game_manager_finder.camera_teleporting(process);
        let maybe_game_state = game_manager_finder.get_game_state(process);
        let maybe_hazard_respawning = game_manager_finder.hazard_respawning(process);
        let maybe_accepting_input = game_manager_finder.accepting_input(process);
        let maybe_hero_transition_state = game_manager_finder.hero_transition_state(process);
        let maybe_tile_map_dirty = game_manager_finder.tile_map_dirty(process);
        let maybe_uses_scene_transition_routine = game_manager_finder.uses_scene_transition_routine(process);

        // only remove loads if timer is running
        if asr::timer::state() != TimerState::Running {
            asr::timer::pause_game_time();
            return;
        }

        let ui_state = maybe_ui_state.unwrap_or_default();

        let scene_name = maybe_scene_name.clone().unwrap_or_default();
        fn is_none_or_empty(ms: Option<&str>) -> bool {
            match ms {  None | Some("") => true, Some(_) => false }
        }
        let loading_menu = (scene_name != "Menu_Title" && is_none_or_empty(maybe_next_scene.as_deref()))
            || (scene_name != "Menu_Title" && maybe_next_scene.as_deref() == Some("Menu_Title")
                || (scene_name == "Quit_To_Menu"));

        let teleporting = maybe_teleporting.unwrap_or_default();

        let game_state = maybe_game_state.unwrap_or_default();
        if game_state == GAME_STATE_PLAYING && self.last_game_state == GAME_STATE_MAIN_MENU {
            self.look_for_teleporting = true;
        }
        if self.look_for_teleporting && (teleporting || (game_state != GAME_STATE_PLAYING && game_state != GAME_STATE_ENTERING_LEVEL)) {
            self.look_for_teleporting = false;
        }

        // TODO: look into Current Patch quitout issues. // might have been fixed? cerpin you broke them in a way that made them work, right?
        let hazard_respawning = maybe_hazard_respawning.unwrap_or_default();
        let accepting_input = maybe_accepting_input.unwrap_or_default();
        let hero_transition_state = maybe_hero_transition_state.unwrap_or_default();
        let tile_map_dirty = maybe_tile_map_dirty.unwrap_or_default();
        let uses_scene_transition_routine = maybe_uses_scene_transition_routine.unwrap_or_default();
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
    }
}
