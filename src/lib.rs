#![cfg_attr(not(target_os = "wasi"), no_std)]

/// Global allocator
#[cfg(not(target_os = "wasi"))]
#[global_allocator]
pub static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

extern crate alloc;

mod asr_xml;
mod auto_splitter_settings;
#[cfg(target_os = "wasi")]
mod file;
mod game_time;
mod hit_counter;
mod hollow_knight_memory;
mod legacy_xml;
mod load_remover;
mod settings_gui;
mod splits;
mod timer;
mod unstable;

use alloc::boxed::Box;
use alloc::format;
use alloc::vec::Vec;

use asr::future::{next_tick, retry};
use asr::game_engine::unity::SceneManager;
use asr::Process;
use game_time::{GameTime, GameTimePlusVars};
use hit_counter::{HitCounter, DASH};
use hollow_knight_memory::*;
use load_remover::LoadRemover;
use settings_gui::{HitsMethod, SettingsGui, TimingMethod};
use splits::Split;
use timer::{Resettable, SplitterAction, Timer};
use ugly_widget::store::StoreGui;

asr::async_main!(stable);
#[cfg(not(target_os = "wasi"))]
asr::panic_handler!();

const TICKS_PER_GUI: usize = 0x100;

struct AutoSplitterState {
    timing_method: TimingMethod,
    hits_method: HitsMethod,
    splits: Vec<Split>,
    load_remover: GameTimePlusVars,
    timer: Timer,
}

impl AutoSplitterState {
    fn new(
        timing_method: TimingMethod,
        hits_method: HitsMethod,
        splits: Vec<Split>,
    ) -> AutoSplitterState {
        let load_remover = timing_method_game_time(splits.len(), timing_method, hits_method);
        let timer = Timer::new(splits.len(), splits::auto_reset_safe(&splits));
        AutoSplitterState {
            timing_method,
            hits_method,
            splits,
            load_remover,
            timer,
        }
    }
}

async fn main() {
    #[cfg(target_os = "wasi")]
    std::panic::set_hook(Box::new(|panic_info| {
        asr::print_message(&panic_info.to_string());
    }));

    // TODO: Set up some general state and settings.

    asr::print_message("Hello, World!");

    // register the variables on start
    asr::timer::set_variable_int("hits", 0);
    asr::timer::set_variable_int("segment hits", 0);
    asr::timer::set_variable("pb hits", DASH);
    asr::timer::set_variable("comparison hits", DASH);
    asr::timer::set_variable("delta hits", DASH);
    asr::timer::set_variable("item", "");

    let mut gui = Box::new(SettingsGui::wait_load_merge_register().await);

    let mut ticks_since_gui = 0;
    let mut state = Box::new(AutoSplitterState::new(
        gui.get_timing_method(),
        gui.get_hit_counter(),
        gui.get_splits(),
    ));
    asr::print_message(&format!("timing_method: {:?}", state.timing_method));
    asr::print_message(&format!("hit_counter: {:?}", state.hits_method));
    asr::print_message(&format!("splits: {:?}", state.splits));

    loop {
        let process = wait_attach_hollow_knight(&mut *gui, &mut state).await;
        process
            .until_closes(async {
                // TODO: Load some initial information from the process.
                let mut scene_store = Box::new(SceneStore::new());

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

                next_tick().await;
                // Initialize pointers for load-remover before timer is running
                game_manager_finder
                    .init_load_removal_pointers(&process)
                    .await;
                next_tick().await;
                asr::print_message("Initialized load removal pointers");
                next_tick().await;

                let scene_manager = SceneManager::attach(&process);
                if scene_manager.is_some() {
                    asr::print_message("Attached SceneManager");
                } else {
                    asr::print_message("Warning: SceneManager not found");
                }
                next_tick().await;

                loop {
                    tick_action(
                        &process,
                        &mut state,
                        scene_manager.as_ref(),
                        &game_manager_finder,
                        &mut scene_store,
                        &mut player_data_store,
                        &mut scene_data_store,
                    )
                    .await;

                    state.load_remover.update_game_time(
                        &state.timer,
                        &process,
                        &game_manager_finder,
                    );

                    #[cfg(debug_assertions)]
                    let new_scenes_grub_rescued = game_manager_finder.scenes_grub_rescued(&process);
                    #[cfg(debug_assertions)]
                    if new_scenes_grub_rescued != scenes_grub_rescued {
                        scenes_grub_rescued = new_scenes_grub_rescued;
                        asr::print_message(&format!(
                            "scenes_grub_rescued: {:?}",
                            scenes_grub_rescued
                        ));
                    }

                    ticks_since_gui += 1;
                    if TICKS_PER_GUI <= ticks_since_gui && gui.load_update_store_if_unchanged() {
                        check_state_change(&mut gui, &mut state);
                        ticks_since_gui = 0;
                    }

                    next_tick().await;
                }
            })
            .await;
    }
}

async fn wait_attach_hollow_knight(
    gui: &mut SettingsGui,
    state: &mut AutoSplitterState,
) -> Process {
    retry(|| {
        gui.loop_load_update_store();
        state.timer.update(&mut state.load_remover);
        check_state_change(gui, state);
        attach_hollow_knight()
    })
    .await
}

fn check_state_change(gui: &mut SettingsGui, state: &mut AutoSplitterState) {
    if state.timer.is_timer_state_between_runs() {
        match (
            gui.check_timing_method(&mut state.timing_method),
            gui.check_hit_counter(&mut state.hits_method),
        ) {
            (None, None) => (),
            _ => {
                state.load_remover = timing_method_game_time(
                    state.timer.n(),
                    state.timing_method,
                    state.hits_method,
                );
            }
        }
    }
    if let Some(new_splits) = gui.check_splits(&mut state.splits) {
        state
            .timer
            .renew(new_splits.len(), splits::auto_reset_safe(new_splits));
    }
}

async fn tick_action(
    process: &Process,
    state: &mut AutoSplitterState,
    scene_manager: Option<&SceneManager>,
    game_manager_finder: &GameManagerFinder,
    scene_store: &mut SceneStore,
    player_data_store: &mut PlayerDataStore,
    scene_data_store: &mut SceneDataStore,
) {
    state.timer.update(&mut state.load_remover);

    let trans_now = scene_store.transition_now(&process, scene_manager, &game_manager_finder);
    loop {
        let Some(s) = state.splits.get(state.timer.i()) else {
            break;
        };
        let a = splits::splits(
            s,
            &process,
            game_manager_finder,
            trans_now,
            scene_store,
            player_data_store,
            scene_data_store,
        );
        match a {
            SplitterAction::Split | SplitterAction::ManualSplit => {
                state.timer.action(a, &mut state.load_remover);
                next_tick().await;
                asr::timer::set_variable("item", "");
                break;
            }
            SplitterAction::Skip | SplitterAction::Reset => {
                state.timer.action(a, &mut state.load_remover);
                next_tick().await;
                // no break, allow other actions after a skip or reset
            }
            SplitterAction::Pass => {
                if state.timer.is_auto_reset_safe() {
                    let a0 = splits::splits(
                        &state.splits[0],
                        &process,
                        game_manager_finder,
                        trans_now,
                        scene_store,
                        player_data_store,
                        scene_data_store,
                    );
                    match a0 {
                        SplitterAction::Split | SplitterAction::Reset => {
                            state.timer.reset();
                            state.timer.action(a0, &mut state.load_remover);
                        }
                        _ => (),
                    }
                }
                break;
            }
        }
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

fn timing_method_game_time(
    n: usize,
    timing_method: TimingMethod,
    hits_method: HitsMethod,
) -> GameTimePlusVars {
    match timing_method {
        TimingMethod::LoadRemovedTime => match hits_method {
            HitsMethod::None => GameTimePlusVars::new(Box::new(LoadRemover::new())),
            HitsMethod::HitsDreamFalls => GameTimePlusVars::new(Box::new(LoadRemover::new()))
                .with_var(Box::new(HitCounter::new(n, true))),
            HitsMethod::HitsDamage => GameTimePlusVars::new(Box::new(LoadRemover::new()))
                .with_var(Box::new(HitCounter::new(n, false))),
        },
        TimingMethod::HitsDreamFalls => GameTimePlusVars::new(Box::new(HitCounter::new(n, true))),
        TimingMethod::HitsDamage => GameTimePlusVars::new(Box::new(HitCounter::new(n, false))),
    }
}
