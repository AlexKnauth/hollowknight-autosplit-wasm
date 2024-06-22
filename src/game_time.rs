
use asr::Process;

use crate::hollow_knight_memory::GameManagerFinder;
use crate::timer::{Resettable, Timer};

pub trait GameTime: Resettable {
    fn update_variables(&mut self, timer: &Timer, process: &Process, game_manager_finder: &GameManagerFinder);
    fn update_game_time(&mut self, timer: &Timer, process: &Process, game_manager_finder: &GameManagerFinder);
}
