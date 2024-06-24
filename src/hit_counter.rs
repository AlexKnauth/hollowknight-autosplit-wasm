
use std::cmp::max;

use asr::Process;
use asr::time::Duration;
use asr::timer::TimerState;

use crate::game_time::GameTime;
use crate::hollow_knight_memory::*;
use crate::timer::{Resettable, Timer};

pub struct HitCounter {
    count_dream_falling: bool,
    hits: i64,
    segments_hits: Vec<i64>,
    i: usize,
    last_recoil: bool,
    last_hazard: bool,
    last_dead_or_0: bool,
    last_exiting_level: Option<String>,
}

impl Resettable for HitCounter {
    fn reset(&mut self) {
        self.hits = 0;
        self.segments_hits = Vec::new();
        self.i = 0;
        asr::timer::set_variable_int("hits", 0);
        asr::timer::set_variable_int("segment hits", 0);
    }
}

#[allow(unused)]
impl HitCounter {
    pub fn new(count_dream_falling: bool) -> HitCounter {
        asr::timer::set_variable_int("hits", 0);
        asr::timer::set_variable_int("segment hits", 0);
        HitCounter {
            count_dream_falling,
            hits: 0,
            segments_hits: Vec::new(),
            i: 0,
            last_recoil: false,
            last_hazard: false,
            last_dead_or_0: false,
            last_exiting_level: None,
        }
    }

    fn add_hit(&mut self) {
        self.hits += 1;
        asr::timer::set_variable_int("hits", self.hits);
        self.segments_hits.resize(max(self.segments_hits.len(), self.i + 1), 0);
        self.segments_hits[self.i] += 1;
        asr::timer::set_variable_int("segment hits", self.segments_hits[self.i]);
    }
}

impl GameTime for HitCounter {
    /// Sets hits variable, but does not set game time
    fn update_variables(&mut self, timer: &Timer, process: &Process, game_manager_finder: &GameManagerFinder) {
        let i = timer.i();
        if i != self.i {
            self.i = i;
            self.segments_hits.resize(max(self.segments_hits.len(), i + 1), 0);
            asr::timer::set_variable_int("segment hits", self.segments_hits[i]);
        }

        // only count hits if timer is running
        if asr::timer::state() != TimerState::Running { return; }

        // new state
        let maybe_recoil = game_manager_finder.hero_recoil(process);
        let maybe_hazard = game_manager_finder.hazard_death(process);
        let maybe_dead = game_manager_finder.hero_dead(process);
        let maybe_health = game_manager_finder.get_health(process);
        let maybe_scene_name = game_manager_finder.get_scene_name(process);
        let maybe_game_state = game_manager_finder.get_game_state(process);

        if let Some(r) = maybe_recoil {
            if !self.last_recoil && r {
                self.add_hit();
                asr::print_message(&format!("hit: {}, from recoil", self.hits));
            }
            self.last_recoil = r;
        }

        if let Some(h) = maybe_hazard {
            if !self.last_hazard && h {
                self.add_hit();
                asr::print_message(&format!("hit: {}, from hazard", self.hits));
            }
            self.last_hazard = h;
        }


        {
            let d = maybe_dead == Some(true) || (maybe_health == Some(0) && maybe_game_state == Some(GAME_STATE_PLAYING));
            if !self.last_dead_or_0 && d {
                self.add_hit();
                asr::print_message(&format!("hit: {}, from dead", self.hits));
            }
            self.last_dead_or_0 = d;
        }

        if self.count_dream_falling {
            if let Some(s) = maybe_scene_name {
                if maybe_game_state == Some(GAME_STATE_ENTERING_LEVEL) && self.last_exiting_level.as_deref() == Some(&s) && is_dream(&s) {
                    self.add_hit();
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
    }

    /// Sets game time to hits
    fn update_game_time(&mut self, timer: &Timer, process: &Process, game_manager_finder: &GameManagerFinder) {

        asr::timer::pause_game_time();

        self.update_variables(timer, process, game_manager_finder);

        // Even set hits when it hasn't incremented,
        // in case this auto-splitter is fighting with something else trying to advance the timer.
        // https://github.com/AlexKnauth/hollowknight-autosplit-wasm/issues/83
        asr::timer::set_game_time(Duration::seconds(self.hits));
    }
}
