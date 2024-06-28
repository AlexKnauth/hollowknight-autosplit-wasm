
use asr::Process;

use crate::hollow_knight_memory::GameManagerFinder;
use crate::timer::{Resettable, Timer};

pub trait GameTime: Resettable {
    fn update_variables(&mut self, timer: &Timer, process: &Process, game_manager_finder: &GameManagerFinder);
    fn update_game_time(&mut self, timer: &Timer, process: &Process, game_manager_finder: &GameManagerFinder);
}

pub struct GameTimePlusVars {
    main: Box<dyn GameTime>,
    vars: Vec<Box<dyn GameTime>>,
}

impl GameTimePlusVars {
    pub fn new(main: Box<dyn GameTime>) -> GameTimePlusVars {
        GameTimePlusVars {
            main,
            vars: Vec::new(),
        }
    }

    pub fn with_var(mut self, var: Box<dyn GameTime>) -> GameTimePlusVars {
        self.vars.push(var);
        self
    }
}

impl Resettable for GameTimePlusVars {
    fn ended(&mut self) {
        self.main.ended();
        for v in self.vars.iter_mut() {
            v.ended();
        }
    }
    fn reset(&mut self) {
        self.main.reset();
        for v in self.vars.iter_mut() {
            v.reset();
        }
    }
}

impl GameTime for GameTimePlusVars {
    fn update_variables(&mut self, timer: &Timer, process: &Process, game_manager_finder: &GameManagerFinder) {
        self.main.update_variables(timer, process, game_manager_finder);
        for v in self.vars.iter_mut() {
            v.update_variables(timer, process, game_manager_finder);
        }
    }

    fn update_game_time(&mut self, timer: &Timer, process: &Process, game_manager_finder: &GameManagerFinder) {
        self.main.update_game_time(timer, process, game_manager_finder);
        for v in self.vars.iter_mut() {
            v.update_variables(timer, process, game_manager_finder);
        }
    }
}
