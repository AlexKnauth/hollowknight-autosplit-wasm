// #![no_std]

extern crate alloc;

mod auto_splitter_settings;
mod file;
mod hollow_knight_memory;
mod legacy_xml;
mod settings_gui;
mod splits;

use asr::future::{next_tick, retry};
use asr::Process;
use asr::time::Duration;
use asr::timer::TimerState;
use settings_gui::{SettingsGui, TimingMethod};
use hollow_knight_memory::*;
use splits::{Split, SplitterAction};
use ugly_widget::store::StoreGui;

asr::async_main!(stable);
// asr::panic_handler!();

const TICKS_PER_GUI: usize = 0x100;

async fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        asr::print_message(&panic_info.to_string());
    }));

    // TODO: Set up some general state and settings.

    asr::print_message("Hello, World!");

    let mut gui = Box::new(SettingsGui::wait_load_merge_register().await);

    let mut ticks_since_gui = 0;
    let mut timing_method = gui.get_timing_method();
    let mut splits = gui.get_splits();
    asr::print_message(&format!("timing_method: {:?}", timing_method));
    asr::print_message(&format!("splits: {:?}", splits));

    loop {
        let process = wait_attach_hollow_knight(&mut *gui, &mut timing_method, &mut splits).await;
        process
            .until_closes(async {
                // TODO: Load some initial information from the process.
                let mut scene_store = Box::new(SceneStore::new());
                let mut load_remover = Box::new(TimingMethodLoadRemover::new(timing_method));
                let mut auto_reset: &'static [TimerState] = splits::auto_reset_safe(&splits);

                next_tick().await;
                let game_manager_finder = Box::new(GameManagerFinder::wait_attach(&process).await);
                let mut player_data_store = Box::new(PlayerDataStore::new());
                let mut scene_data_store = Box::new(SceneDataStore::new());

                #[cfg(debug_assertions)]
                asr::print_message(&format!("geo: {:?}", game_manager_finder.get_geo(&process)));

                #[cfg(debug_assertions)]
                let mut scenes_grub_rescued = game_manager_finder.scenes_grub_rescued(&process);
                #[cfg(debug_assertions)]
                asr::print_message(&format!("scenes_grub_rescued: {:?}", scenes_grub_rescued));

                let mut last_timer_state = TimerState::Unknown;
                let mut i = 0;
                loop {
                    tick_action(&process, &splits, &mut last_timer_state, &mut i, auto_reset, &game_manager_finder, &mut scene_store, &mut player_data_store, &mut scene_data_store, &mut load_remover).await;

                    load_remover.load_removal(&process, &game_manager_finder, i);

                    #[cfg(debug_assertions)]
                    let new_scenes_grub_rescued = game_manager_finder.scenes_grub_rescued(&process);
                    #[cfg(debug_assertions)]
                    if new_scenes_grub_rescued != scenes_grub_rescued {
                        scenes_grub_rescued = new_scenes_grub_rescued;
                        asr::print_message(&format!("scenes_grub_rescued: {:?}", scenes_grub_rescued));
                    }

                    ticks_since_gui += 1;
                    if TICKS_PER_GUI <= ticks_since_gui && gui.load_update_store_if_unchanged() {
                        if i == 0 && [TimerState::NotRunning, TimerState::Ended].contains(&asr::timer::state()) {
                            if let Some(new_timing_method) = gui.check_timing_method(&mut timing_method) {
                                *load_remover = TimingMethodLoadRemover::new(new_timing_method);
                            }
                        }
                        if let Some(new_splits) = gui.check_splits(&mut splits) {
                            auto_reset = splits::auto_reset_safe(new_splits);
                        }
                        ticks_since_gui = 0;
                    }

                    next_tick().await;
                }
            })
            .await;
    }
}

async fn wait_attach_hollow_knight(gui: &mut SettingsGui, timing_method: &mut TimingMethod, splits: &mut Vec<Split>) -> Process {
    retry(|| {
        gui.loop_load_update_store();
        gui.check_timing_method(timing_method);
        gui.check_splits(splits);
        attach_hollow_knight()
    }).await
}

async fn tick_action(
    process: &Process,
    splits: &[splits::Split],
    last_timer_state: &mut TimerState,
    i: &mut usize,
    auto_reset: &'static [TimerState],
    game_manager_finder: &GameManagerFinder,
    scene_store: &mut SceneStore,
    player_data_store: &mut PlayerDataStore,
    scene_data_store: &mut SceneDataStore,
    load_remover: &mut TimingMethodLoadRemover,
) {
    let n = splits.len();
    let timer_state = asr::timer::state();
    match timer_state {
        // detect manual resets
        TimerState::NotRunning if 0 < *i => {
            *i = 0;
            load_remover.reset();
            asr::print_message("Detected a manual reset.");
        }
        // detect manual starts
        TimerState::Running if *i == 0 && is_timer_state_between_runs(*last_timer_state) => {
            *i = 1;
            load_remover.reset();
            asr::print_message("Detected a manual start.");
        }
        // detect manual end-splits
        TimerState::Ended if 0 < *i && *i < n => {
            *i = n;
            asr::print_message("Detected a manual end-split.");
        }
        s => {
            *last_timer_state = s;
        }
    }

    let trans_now = scene_store.transition_now(&process, &game_manager_finder);
    loop {
        let Some(s) = splits.get(*i) else {
            break;
        };
        let a = splits::splits(s, &process, game_manager_finder, trans_now, scene_store, player_data_store, scene_data_store);
        match a {
            SplitterAction::Split | SplitterAction::ManualSplit => {
                splitter_action(a, i, n, load_remover);
                next_tick().await;
                break;
            }
            SplitterAction::Skip | SplitterAction::Reset => {
                splitter_action(a, i, n, load_remover);
                next_tick().await;
                // no break, allow other actions after a skip or reset
            }
            SplitterAction::Pass => {
                // TODO: this timer_state may be out of date
                if auto_reset.contains(&timer_state) {
                    let a0 = splits::splits(&splits[0], &process, game_manager_finder, trans_now, scene_store, player_data_store, scene_data_store);
                    match a0 {
                        SplitterAction::Split | SplitterAction::Reset => {
                            *i = 0;
                            splitter_action(a0, i, n, load_remover);
                        }
                        _ => (),
                    }
                }
                break;
            }
        }
    }

    // TODO: this timer_state may be out of date
    if auto_reset.contains(&timer_state) && n <= *i {
        *i = 0;
    }

    if trans_now {
        if scene_store.pair().old == MENU_TITLE {
            player_data_store.reset();
            scene_data_store.reset();
        } else {
            player_data_store.clean_on_entry();
        }
    }
}

fn is_timer_state_between_runs(s: TimerState) -> bool {
    s == TimerState::NotRunning || s == TimerState::Ended
}

fn splitter_action(a: SplitterAction, i: &mut usize, n: usize, load_remover: &mut TimingMethodLoadRemover) {
    match a {
        SplitterAction::Pass => (),
        SplitterAction::Reset => {
            asr::timer::reset();
            load_remover.reset();
            *i = 0;
        }
        SplitterAction::Skip => {
            asr::timer::skip_split();
            *i += 1;
        }
        SplitterAction::Split if *i == 0 => {
            asr::timer::reset();
            asr::timer::start();
            load_remover.reset();
            *i += 1;
        }
        SplitterAction::Split => {
            asr::timer::split();
            *i += 1;
        }
        SplitterAction::ManualSplit => {
            if 0 < *i && *i + 1 < n {
                *i += 1;
            }
        }
    }
}

enum TimingMethodLoadRemover {
    LoadRemover(LoadRemover),
    HitCounter(HitCounter),
}

impl TimingMethodLoadRemover {
    fn new(timing_method: TimingMethod) -> TimingMethodLoadRemover {
        match timing_method {
            TimingMethod::LoadRemovedTime => TimingMethodLoadRemover::LoadRemover(LoadRemover::new()),
            TimingMethod::HitsDreamFalls => TimingMethodLoadRemover::HitCounter(HitCounter::new(true)),
            TimingMethod::HitsDamage => TimingMethodLoadRemover::HitCounter(HitCounter::new(false)),
        }
    }

    fn reset(&mut self) {
        match self {
            TimingMethodLoadRemover::LoadRemover(lr) => lr.reset(),
            TimingMethodLoadRemover::HitCounter(hc) => hc.reset(),
        }
    }

    fn load_removal(&mut self, process: &Process, game_manager_finder: &GameManagerFinder, i: usize) -> Option<()> {
        match self {
            TimingMethodLoadRemover::LoadRemover(lr) => lr.load_removal(process, game_manager_finder, i),
            TimingMethodLoadRemover::HitCounter(hc) => hc.load_removal(process, game_manager_finder, i),
        }
    }
}

struct LoadRemover {
    look_for_teleporting: bool,
    last_game_state: i32,
    #[cfg(debug_assertions)]
    last_paused: bool,
}

#[allow(unused)]
impl LoadRemover {
    fn new() -> LoadRemover {
        LoadRemover { 
            look_for_teleporting: false,
            last_game_state: GAME_STATE_INACTIVE,
            #[cfg(debug_assertions)]
            last_paused: false,
        }
    }

    fn reset(&mut self) {}

    fn load_removal(&mut self, process: &Process, game_manager_finder: &GameManagerFinder, _i: usize) -> Option<()> {
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
            return Some(());
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
        Some(())
    }
}

struct HitCounter {
    count_dream_falling: bool,
    hits: i64,
    last_recoil: bool,
    last_hazard: bool,
    last_dead_or_0: bool,
    last_exiting_level: Option<String>,
    last_index: usize,
}

#[allow(unused)]
impl HitCounter {
    fn new(count_dream_falling: bool) -> HitCounter {
        asr::timer::set_variable_int("hits", 0);
        HitCounter {
            count_dream_falling,
            hits: 0,
            last_recoil: false,
            last_hazard: false,
            last_dead_or_0: false,
            last_exiting_level: None,
            last_index: 0,
        }
    }

    fn reset(&mut self) {
        self.hits = 0;
        asr::timer::pause_game_time();
        asr::timer::set_game_time(Duration::seconds(0));
        asr::timer::set_variable_int("hits", 0);
    }

    fn load_removal(&mut self, process: &Process, game_manager_finder: &GameManagerFinder, i: usize) -> Option<()> {

        asr::timer::pause_game_time();

        // detect resets
        if i == 0 && 0 < self.last_index {
            self.reset();
        }
        self.last_index = i;

        // only count hits if timer is running
        if asr::timer::state() != TimerState::Running { return Some(()); }

        // new state
        let maybe_recoil = game_manager_finder.hero_recoil(process);
        let maybe_hazard = game_manager_finder.hazard_death(process);
        let maybe_dead = game_manager_finder.hero_dead(process);
        let maybe_health = game_manager_finder.get_health(process);
        let maybe_scene_name = game_manager_finder.get_scene_name(process);
        let maybe_game_state = game_manager_finder.get_game_state(process);

        if let Some(r) = maybe_recoil {
            if !self.last_recoil && r {
                self.hits += 1;
                asr::timer::set_game_time(Duration::seconds(self.hits));
                asr::timer::set_variable_int("hits", self.hits);
                asr::print_message(&format!("hit: {}, from recoil", self.hits));
            }
            self.last_recoil = r;
        }

        if let Some(h) = maybe_hazard {
            if !self.last_hazard && h {
                self.hits += 1;
                asr::timer::set_game_time(Duration::seconds(self.hits));
                asr::timer::set_variable_int("hits", self.hits);
                asr::print_message(&format!("hit: {}, from hazard", self.hits));
            }
            self.last_hazard = h;
        }


        {
            let d = maybe_dead == Some(true) || (maybe_health == Some(0) && maybe_game_state == Some(GAME_STATE_PLAYING));
            if !self.last_dead_or_0 && d {
                self.hits += 1;
                asr::timer::set_game_time(Duration::seconds(self.hits));
                asr::timer::set_variable_int("hits", self.hits);
                asr::print_message(&format!("hit: {}, from dead", self.hits));
            }
            self.last_dead_or_0 = d;
        }

        if self.count_dream_falling {
            if let Some(s) = maybe_scene_name {
                if maybe_game_state == Some(GAME_STATE_ENTERING_LEVEL) && self.last_exiting_level.as_deref() == Some(&s) && is_dream(&s) {
                    self.hits += 1;
                    asr::timer::set_game_time(Duration::seconds(self.hits));
                    asr::timer::set_variable_int("hits", self.hits);
                    asr::print_message(&format!("hit: {}, from dream falling", self.hits));
                }
                if maybe_game_state == Some(GAME_STATE_EXITING_LEVEL) {
                    if self.last_exiting_level.is_none() {
                        self.last_exiting_level = Some(s);
                    }
                } else {
                    self.last_exiting_level = None;
                }
            }
        }

        Some(())
    }
}
