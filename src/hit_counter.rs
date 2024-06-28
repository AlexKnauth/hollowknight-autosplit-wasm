
use std::cmp::{max, min};

use asr::Process;
use asr::time::Duration;
use asr::timer::TimerState;

use crate::game_time::GameTime;
use crate::hollow_knight_memory::*;
use crate::timer::{Resettable, Timer};

/// The dash symbol to use for generic dashes in text.
const DASH: &str = "—";
/// The minus symbol to use for negative numbers.
const MINUS: &str = "−";
/// The plus symbol to use for positive numbers.
const PLUS: &str = "+";

pub struct HitCounter {
    count_dream_falling: bool,
    hits: i64,
    /// Vector up to length n. Entries:
    /// 0: 0
    /// [1,n): number of hits in the i_th segment of the active attempt, where
    /// n-1: number of hits on the last segment
    segments_hits: Vec<i64>,
    /// Vector up to length n. Entries:
    /// 0: 0
    /// [1,n): cumulative number of hits up through the end of segment i, where
    /// n-1: pb hits
    comparison_hits: Vec<i64>,
    /// Index into the list of autosplits including start and end.
    /// Not a segment index, since start is not a segment.
    /// 0: either NotRunning, or Ended with auto-reset safe
    /// [1,n): Running
    /// n: Ended without knowing auto-reset safe
    i: usize,
    /// Number of autosplits including both start and end.
    /// One more than the number of segments.
    n: Option<usize>,
    last_recoil: bool,
    last_hazard: bool,
    last_dead_or_0: bool,
    last_exiting_level: Option<String>,
}

impl Resettable for HitCounter {
    fn ended(&mut self) {
        let Some(n) = self.n else {
            return;
        };
        self.comparison_hits.resize(max(self.comparison_hits.len(), n), self.hits);
        if 1 <= n {
            self.comparison_hits[n - 1] = min(self.comparison_hits[n - 1], self.hits);
            asr::timer::set_variable_int("pb hits", self.comparison_hits[n - 1]);
        }
    }
    fn reset(&mut self) {
        self.hits = 0;
        self.segments_hits = Vec::new();
        store_comparison_hits(&self.comparison_hits);
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
        let comparison_hits = load_comparison_hits().unwrap_or_default();
        HitCounter {
            count_dream_falling,
            hits: 0,
            segments_hits: Vec::new(),
            comparison_hits,
            i: 0,
            n: None,
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
        if let Some(cmp) = self.comparison_hits.get(self.i) {
            asr::timer::set_variable("delta hits", &delta_string(self.hits - *cmp));
        } else {
            asr::timer::set_variable("delta hits", DASH);
        }
    }

    fn add_pace(&mut self) {
        self.comparison_hits.resize(max(self.comparison_hits.len(), self.i), self.hits);
        if 1 <= self.i {
            self.comparison_hits[self.i - 1] = min(self.comparison_hits[self.i - 1], self.hits);
        }
    }
}

impl GameTime for HitCounter {
    /// Sets hits variable, but does not set game time
    fn update_variables(&mut self, timer: &Timer, process: &Process, game_manager_finder: &GameManagerFinder) {
        let i = timer.i();
        if i != self.i {
            if self.i == 0 {
                self.comparison_hits = load_comparison_hits().unwrap_or_default();
            } else if i == 0 {
                store_comparison_hits(&self.comparison_hits);
            }
            self.i = i;
            self.n = Some(timer.n());
            self.segments_hits.resize(max(self.segments_hits.len(), i + 1), 0);
            asr::timer::set_variable_int("segment hits", self.segments_hits[i]);
            if let Some(cmp) = self.comparison_hits.get(self.i) {
                asr::timer::set_variable_int("comparison hits", *cmp);
                asr::timer::set_variable("delta hits", &delta_string(self.hits - *cmp));
            } else {
                asr::timer::set_variable("comparison hits", DASH);
                asr::timer::set_variable("delta hits", DASH);
            }
            self.add_pace();
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

fn load_comparison_hits() -> Option<Vec<i64>> {
    let v = asr::settings::Map::load().get("comparison_hits")?;
    let l = v.get_list()?;
    let mut r = Vec::new();
    for e in l.iter() {
        let Some(i) = e.get_i64() else {
            break;
        };
        r.push(i);
    }
    Some(r)
}

fn store_comparison_hits(is: &[i64]) {
    loop {
        if store_comparison_hits_if_unchanged(is) {
            break;
        }
    }
}

fn store_comparison_hits_if_unchanged(is: &[i64]) -> bool {
    let l = asr::settings::List::new();
    for i in is {
        l.push(*i);
    }
    let m = asr::settings::Map::load();
    let old = m.clone();
    m.insert("comparison_hits", l);
    m.store_if_unchanged(&old)
}

fn delta_string(i: i64) -> String {
    if i.is_positive() {
        format!("{}{}", PLUS, i)
    } else if i.is_negative() {
        format!("{}{}", MINUS, -i)
    } else {
        format!("{}", i)
    }
}
