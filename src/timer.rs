
use asr::timer::TimerState;
use serde::{Deserialize, Serialize};

pub fn is_timer_state_between_runs(s: TimerState) -> bool {
    s == TimerState::NotRunning || s == TimerState::Ended
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum SplitterAction {
    #[default]
    Pass,
    Split,
    Skip,
    Reset,
    ManualSplit,
}

impl SplitterAction {
    pub fn or_else<F: FnOnce() -> SplitterAction>(self, f: F) -> SplitterAction {
        match self {
            SplitterAction::Pass => f(),
            a => a,
        }
    }
}

pub fn should_split(b: bool) -> SplitterAction {
    if b {
        SplitterAction::Split
    } else {
        SplitterAction::Pass
    }
}

pub fn should_skip(b: bool) -> SplitterAction {
    if b {
        SplitterAction::Skip
    } else {
        SplitterAction::Pass
    }
}

pub fn should_split_skip(mb: Option<bool>) -> SplitterAction {
    match mb  {
        Some(true) => SplitterAction::Split,
        Some(false) => SplitterAction::Skip,
        None => SplitterAction::Pass,
    }
}

pub trait Resettable {
    fn reset(&mut self);
}

/// Keep track of autosplit index here because asr::timer won't.
pub struct Timer {
    /// The timer state.
    state: TimerState,
    /// The last observed asr::timer::state.
    /// Just in case asr::timer::state is a tad out-of-date.
    last_state: TimerState,
    /// Index into the list of autosplits including start and end.
    /// Not a segment index, since start is not a segment.
    /// 0: either NotRunning, or Ended with auto-reset safe
    /// [1,n): Running
    /// n: Ended without knowing auto-reset safe
    i: usize,
    /// Number of autosplits including both start and end.
    /// One more than the number of segments.
    n: usize,
    /// The set of timer states where it is safe to use auto-reset.
    auto_reset: &'static [TimerState],
}

impl Resettable for Timer {
    fn reset(&mut self) {
        asr::timer::reset();
        self.state = TimerState::NotRunning;
        self.i = 0;
    }
}

impl Timer {
    pub fn new(n: usize, auto_reset: &'static [TimerState]) -> Timer {
        let asr_state = asr::timer::state();
        Timer {
            state: asr_state,
            last_state: asr_state,
            i: 0,
            n,
            auto_reset,
        }
    }

    pub fn renew(&mut self, n: usize, auto_reset: &'static [TimerState]) {
        self.n = n;
        self.auto_reset = auto_reset;
    }

    pub fn i(&self) -> usize {
        self.i
    }

    pub fn is_auto_reset_safe(&self) -> bool {
        self.auto_reset.contains(&self.state)
    }

    pub fn is_timer_state_between_runs(&self) -> bool {
        is_timer_state_between_runs(self.state)
    }

    pub fn update<R: Resettable>(&mut self, r: &mut R) {
        let asr_state = asr::timer::state();
        if asr_state == self.state || asr_state == self.last_state {
            self.last_state = asr_state;
            return;
        }
        match asr_state {
            // detect manual resets
            TimerState::NotRunning => {
                self.i = 0;
                r.reset();
                asr::print_message("Detected a manual reset.");
            }
            // detect manual starts
            TimerState::Running if is_timer_state_between_runs(self.state) => {
                self.i = 1;
                r.reset();
                asr::print_message("Detected a manual start.");
            }
            // detect manual end-splits
            TimerState::Ended => {
                if self.is_auto_reset_safe() {
                    // do NOT actually reset
                    // 0: either NotRunning, or Ended with auto-reset safe
                    self.i = 0;
                } else {
                    self.i = self.n;
                }
                asr::print_message("Detected a manual end-split.");
            }
            _ => ()
        }
        self.state = asr_state;
        self.last_state = asr_state;
    }

    pub fn action<R: Resettable>(&mut self, a: SplitterAction, r: &mut R) {
        match a {
            SplitterAction::Pass => (),
            SplitterAction::Reset => {
                self.reset();
                r.reset();
            }
            SplitterAction::Skip => {
                asr::timer::skip_split();
                self.i += 1;
            }
            SplitterAction::Split => {
                if self.i == 0 {
                    asr::timer::reset();
                    asr::timer::start();
                    r.reset();
                    self.state = TimerState::Running;
                } else {
                    asr::timer::split();
                }
                self.i += 1;
            }
            SplitterAction::ManualSplit => {
                if 0 < self.i && self.i + 1 < self.n {
                    self.i += 1;
                }
            }
        }
        if self.n <= self.i {
            self.state = TimerState::Ended;
            if self.is_auto_reset_safe() {
                // do NOT actually reset
                // 0: either NotRunning, or Ended with auto-reset safe
                self.i = 0;
            }
        }
    }
}
