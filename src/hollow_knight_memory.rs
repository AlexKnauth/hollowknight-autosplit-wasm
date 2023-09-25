
use std::cmp::min;
use std::mem;
use std::collections::BTreeMap;
use asr::future::retry;
use asr::watcher::Pair;
use asr::{Process, Address, Address64};
use asr::game_engine::unity::{SceneManager, get_scene_name};
use asr::game_engine::unity::mono;
use asr::string::{ArrayCString, ArrayWString};

#[cfg(debug_assertions)]
use std::string::String;
#[cfg(debug_assertions)]
use serde::{Deserialize, Serialize};

// --------------------------------------------------------

const HOLLOW_KNIGHT_NAMES: [&str; 2] = [
    "hollow_knight.exe", // Windows
    "Hollow Knight", // Mac
];

pub const SCENE_PATH_SIZE: usize = 64;

const INIT_MAX_DIRTYNESS: usize = 0x10;

const STRING_LEN_OFFSET: u64 = 0x10;
const STRING_CONTENTS_OFFSET: u64 = 0x14;

const PRE_MENU_INTRO: &str = "Pre_Menu_Intro";
pub const MENU_TITLE: &str = "Menu_Title";
pub const QUIT_TO_MENU: &str = "Quit_To_Menu";
pub const OPENING_SEQUENCE: &str = "Opening_Sequence";
pub const GG_ENTRANCE_CUTSCENE: &str = "GG_Entrance_Cutscene";

const NON_PLAY_SCENES: [&str; 15] = [
    PRE_MENU_INTRO,
    MENU_TITLE,
    QUIT_TO_MENU,
    OPENING_SEQUENCE,
    GG_ENTRANCE_CUTSCENE,
    "Cinematic_Ending_A",
    "Cinematic_Ending_B",
    "Cinematic_Ending_C",
    "Cinematic_Ending_D",
    "Cinematic_Ending_E",
    "End_Credits",
    "Cinematic_MrMushroom",
    "End_Game_Completion",
    "PermaDeath",
    "PermaDeath_Unlock",
];

const BAD_SCENE_NAMES: [&str; 11] = [
    "Untagged",
    "left1",
    "oncomplete",
    "Attack Range",
    "onstart",
    "position",
    "looptype",
    "integer1",
    "gameObject",
    "eventTarget",
    "material",
];

// --------------------------------------------------------

const SCENE_NAME_OFFSET: u64 = 0x18;
const NEXT_SCENE_NAME_OFFSET: u64 = 0x20;

const UI_MANAGER_VANILLA_OFFSET: u64 = 0xa0;
const GAME_STATE_VANILLA_OFFSET: u64 = 0x174;
const GAME_STATE_MODDING_API_OFFSET: u64 = 0x184;
const GAME_STATE_PLAYING: i32 = 4;

const PLAYER_DATA_OFFSET: u64 = 0xc8;

#[derive(Debug, PartialEq)]
struct GameManagerOffsets {
    instance: u64,
    scene_name: u64,
    next_scene_name: u64,
    player_data: u64,
}

const GAME_MANAGER_OFFSETS: GameManagerOffsets = GameManagerOffsets {
    instance: 0x8,
    scene_name: SCENE_NAME_OFFSET,
    next_scene_name: NEXT_SCENE_NAME_OFFSET,
    player_data: PLAYER_DATA_OFFSET,
};

impl GameManagerOffsets {
    async fn wait_new(process: &Process, module: &mono::Module, class: &mono::Class) -> GameManagerOffsets {
        let offsets = GameManagerOffsets {
            instance: class.wait_get_field(process, module, "_instance").await as u64,
            scene_name: class.wait_get_field(process, module, "sceneName").await as u64,
            next_scene_name: class.wait_get_field(process, module, "nextSceneName").await as u64,
            player_data: class.wait_get_field(process, module, "playerData").await as u64,
        };
        if offsets != GAME_MANAGER_OFFSETS {
            asr::print_message("GameManagerOffsets mismatch");
        }
        offsets
    }
    fn clean(&mut self, process: &Process, module: &mono::Module, class: &mono::Class) -> bool {
        let mut dirty = false;
        if let Some(instance) = class.get_field(process, module, "_instance") {
            self.instance = instance as u64;
        } else {
            dirty = true;
        }
        if let Some(scene_name) = class.get_field(process, module, "sceneName") {
            self.scene_name = scene_name as u64;
        } else {
            dirty = true;
        }
        if let Some(next_scene_name) = class.get_field(process, module, "nextSceneName") {
            self.next_scene_name = next_scene_name as u64;
        } else {
            dirty = true;
        }
        if let Some(player_data) = class.get_field(process, module, "playerData") {
            self.player_data = player_data as u64;
        } else {
            dirty = true;
        }
        if self != &GAME_MANAGER_OFFSETS {
            asr::print_message("GameManagerOffsets mismatch");
            dirty = true;
        }
        !dirty
    }
}

// --------------------------------------------------------

const FIREBALL_LEVEL_OFFSET: u64 = 0x260;

const HAS_DASH_OFFSET: u64 = 0x284;

const HAS_SHADOW_DASH_OFFSET: u64 = 0x287;

const HAS_WALL_JUMP_OFFSET: u64 = 0x285;

const HAS_DOUBLE_JUMP_OFFSET: u64 = 0x289;

const HAS_SUPER_DASH_OFFSET: u64 = 0x286;

const HAS_ACID_ARMOR_OFFSET: u64 = 0x288;

const HAS_DREAM_NAIL_OFFSET: u64 = 0x271;

const HAS_DREAM_GATE_OFFSET: u64 = 0x272;

const DREAM_NAIL_UPGRADED_OFFSET: u64 = 0x273;

// Base number of masks, without any charms, bindings, lifeblood, or damage taken
const MAX_HEALTH_BASE_OFFSET: u64 = 0x198;

// Heart pieces represents one of:
//  - number of heart pieces including the ones assembled into masks: 0-3 4-7 8-11 12-15 16
//  - number of heart pieces excluding the ones assembled into masks: 0-3 0-3 0-3  0-3   0
//  - number of heart pieces excluding masks except the final mask:   0-3 0-3 0-3  0-3   4
// and I'm not sure which one
const HEART_PIECES_OFFSET: u64 = 0x1a8;

const HAS_LANTERN_OFFSET: u64 = 0x28a;

const SIMPLE_KEYS_OFFSET: u64 = 0x2d8;

const HAS_SLY_KEY_OFFSET: u64 = 0x28e;

const HAS_WHITE_KEY_OFFSET: u64 = 0x290;

#[cfg(debug_assertions)]
const GEO_OFFSET: u64 = 0x1c4;

// Dashmaster
const GOT_CHARM_31_OFFSET: u64 = 0x5c9;

const GRUBS_COLLECTED_OFFSET: u64 = 0xb94;

// Gruz Mother
const KILLED_BIG_FLY_OFFSET: u64 = 0x6c1;

const SLY_RESCUED_OFFSET: u64 = 0x389;

const KILLED_GORGEOUS_HUSK_OFFSET: u64 = 0x879;

// Lemm
const MET_RELIC_DEALER_SHOP_OFFSET: u64 = 0x34a;

const WATCHER_CHANDELIER_OFFSET: u64 = 0xc8d;

const KILLED_BLACK_KNIGHT_OFFSET: u64 = 0x7f9;

const KILLED_MEGA_JELLYFISH_OFFSET: u64 = 0x7a1;

const SPIDER_CAPTURE_OFFSET: u64 = 0xca0;

const UNCHAINED_HOLLOW_KNIGHT_OFFSET: u64 = 0xcc9;

// --------------------------------------------------------

#[cfg(debug_assertions)]
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct SceneInfo {
    name: String,
    path: String
}

#[cfg(debug_assertions)]
pub type SceneTable = BTreeMap<i32, SceneInfo>;

// --------------------------------------------------------

pub fn get_current_scene_name(process: &Process, scene_manager: &SceneManager) -> Result<String, asr::Error> {
    scene_manager.get_current_scene_path::<SCENE_PATH_SIZE>(&process).map(scene_path_to_name_string)
}

async fn wait_get_current_scene_path<const N: usize>(process: &Process, scene_manager: &SceneManager) -> ArrayCString<N> {
    retry(|| scene_manager.get_current_scene_path(&process)).await
}

pub async fn wait_get_current_scene_name(process: &Process, scene_manager: &SceneManager) -> String {
    scene_path_to_name_string(wait_get_current_scene_path::<SCENE_PATH_SIZE>(&process, scene_manager).await)
}

pub struct GameManagerFinder {
    module: mono::Module,
    image: mono::Image,
    class: mono::Class,
    static_table: Address,
    offsets: GameManagerOffsets,
    max_dirtyness: usize,
    dirtyness: usize,
}

impl GameManagerFinder {
    pub async fn wait_attach(process: &Process) -> GameManagerFinder {
        let module = mono::Module::wait_attach_auto_detect(process).await;
        let image = module.wait_get_default_image(process).await;
        let class = image.wait_get_class(process, &module, "GameManager").await;
        let static_table = class.wait_get_static_table(process, &module).await;
        let offsets = GameManagerOffsets::wait_new(process, &module, &class).await;
        let max_dirtyness = INIT_MAX_DIRTYNESS;
        let dirtyness = 0;
        GameManagerFinder { module, image, class, static_table, offsets, max_dirtyness, dirtyness }
    }

    pub fn is_dirty(&self) -> bool {
        self.max_dirtyness < self.dirtyness
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        if dirty {
            self.dirtyness += 1;
        } else {
            if 0 < self.dirtyness {
                asr::print_message(&format!("GameManagerFinder dirtyness: {}", self.dirtyness))
            }
            self.dirtyness = 0;
            self.max_dirtyness = INIT_MAX_DIRTYNESS;
        }
    }

    pub async fn attempt_clean(&mut self, process: &Process) -> Option<()> {
        if !self.is_dirty() { return Some(()); }
        if let (Some(static_table), true) = (self.class.get_static_table(process, &self.module), self.offsets.clean(process, &self.module, &self.class)) {
            self.static_table = static_table;
        } else {
            if let Some(class) = self.image.get_class(process, &self.module, "GameManager") {
                self.class = class;
            } else {
                if let Some(image) = self.module.get_default_image(process) {
                    self.image = image;
                } else {
                    self.module = mono::Module::wait_attach_auto_detect(process).await;
                    self.image = self.module.wait_get_default_image(process).await;
                }
                self.class = self.image.wait_get_class(process, &self.module, "GameManager").await;
            }
            self.static_table = self.class.wait_get_static_table(process, &self.module).await;
            self.offsets = GameManagerOffsets::wait_new(process, &self.module, &self.class).await;
        }
        self.dirtyness = 0;
        self.max_dirtyness *= 2;
        Some(())
    }

    pub fn get_scene_name(&self, process: &Process) -> Option<String> {
        let s = process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.scene_name]).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, s)
    }

    pub fn get_next_scene_name(&self, process: &Process) -> Option<String> {
        let s = process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.next_scene_name]).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, s)
    }

    pub fn get_game_state(&self, process: &Process) -> Option<i32> {
        let ui_manager_vanilla: Address64 = process.read_pointer_path64(self.static_table, &[self.offsets.instance, UI_MANAGER_VANILLA_OFFSET]).ok()?;
        let game_state_offset = if ui_manager_vanilla.is_null() { GAME_STATE_MODDING_API_OFFSET } else { GAME_STATE_VANILLA_OFFSET };
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, game_state_offset]).ok()
    }

    fn is_game_state_playing(&self, process: &Process) -> bool {
        self.get_game_state(process) == Some(GAME_STATE_PLAYING)
    }

    pub fn get_fireball_level(&self, process: &Process) -> Option<i32> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, FIREBALL_LEVEL_OFFSET]).ok()
    }

    pub fn has_dash(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, HAS_DASH_OFFSET]).ok()
    }

    pub fn has_shadow_dash(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, HAS_SHADOW_DASH_OFFSET]).ok()
    }

    pub fn has_wall_jump(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, HAS_WALL_JUMP_OFFSET]).ok()
    }

    pub fn has_double_jump(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, HAS_DOUBLE_JUMP_OFFSET]).ok()
    }

    pub fn has_super_dash(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, HAS_SUPER_DASH_OFFSET]).ok()
    }

    pub fn has_acid_armour(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, HAS_ACID_ARMOR_OFFSET]).ok()
    }

    pub fn has_dream_nail(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, HAS_DREAM_NAIL_OFFSET]).ok()
    }

    pub fn has_dream_gate(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, HAS_DREAM_GATE_OFFSET]).ok()
    }
    
    pub fn dream_nail_upgraded(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, DREAM_NAIL_UPGRADED_OFFSET]).ok()
    }

    pub fn max_health_base(&self, process: &Process) -> Option<i32> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, MAX_HEALTH_BASE_OFFSET]).ok()
    }

    pub fn heart_pieces(&self, process: &Process) -> Option<i32> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, HEART_PIECES_OFFSET]).ok()
    }

    pub fn has_lantern(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, HAS_LANTERN_OFFSET]).ok()
    }

    pub fn get_simple_keys(&self, process: &Process) -> Option<i32> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, SIMPLE_KEYS_OFFSET]).ok()
    }

    pub fn has_sly_key(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, HAS_SLY_KEY_OFFSET]).ok()
    }

    pub fn has_white_key(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, HAS_WHITE_KEY_OFFSET]).ok()
    }

    #[cfg(debug_assertions)]
    pub fn get_geo(&self, process: &Process) -> Option<i32> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, GEO_OFFSET]).ok()
    }

    // Dashmaster
    pub fn got_charm_31(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, GOT_CHARM_31_OFFSET]).ok()
    }

    pub fn grubs_collected(&self, process: &Process) -> Option<i32> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, GRUBS_COLLECTED_OFFSET]).ok()
    }

    // Gruz Mother
    pub fn killed_big_fly(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, KILLED_BIG_FLY_OFFSET]).ok()
    }

    pub fn sly_rescued(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, SLY_RESCUED_OFFSET]).ok()
    }

    pub fn killed_gorgeous_husk(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, KILLED_GORGEOUS_HUSK_OFFSET]).ok()
    }

    // Lemm
    pub fn met_relic_dealer_shop(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, MET_RELIC_DEALER_SHOP_OFFSET]).ok()
    }

    pub fn watcher_chandelier(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, WATCHER_CHANDELIER_OFFSET]).ok()
    }

    pub fn killed_black_knight(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, KILLED_BLACK_KNIGHT_OFFSET]).ok()
    }

    pub fn killed_mega_jellyfish(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, KILLED_MEGA_JELLYFISH_OFFSET]).ok()
    }

    pub fn spider_capture(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, SPIDER_CAPTURE_OFFSET]).ok()
    }

    pub fn unchained_hollow_knight(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.static_table, &[self.offsets.instance, self.offsets.player_data, UNCHAINED_HOLLOW_KNIGHT_OFFSET]).ok()
    }
}

pub struct SceneStore {
    old_scene_name: String,
    prev_scene_name: String,
    curr_scene_name: String,
    next_scene_name: String,
    new_data_curr: bool,
    new_data_next: bool
}

impl SceneStore {
    pub fn new(init_scene_name: String) -> SceneStore {
        #[cfg(debug_assertions)]
        asr::print_message(&format!("init_scene_name: {}", init_scene_name));
        SceneStore {
            old_scene_name: "".to_string(),
            prev_scene_name: "".to_string(),
            curr_scene_name: init_scene_name,
            next_scene_name: "".to_string(),
            new_data_curr: false,
            new_data_next: false
        }
    }

    #[cfg(debug_assertions)]
    pub fn curr_scene_name(&self) -> &str {
        &self.curr_scene_name
    }

    pub fn new_curr_scene_name(&mut self, mcsn: Option<String>) {
        match mcsn {
            Some(csn) if csn != self.curr_scene_name => {
                self.prev_scene_name = mem::replace(&mut self.curr_scene_name, csn);
                #[cfg(debug_assertions)]
                asr::print_message(&format!("curr_scene_name: {}", self.curr_scene_name));
                self.new_data_curr = self.curr_scene_name != self.next_scene_name;
            }
            _ => ()
        }
    }
    pub fn new_curr_scene_name2(&mut self, ma: Option<String>, mb: Option<String>) -> (bool, bool) {
        match (ma, mb) {
            (None, None) => (false, false),
            (Some(ab), None) | (None, Some(ab)) => {
                self.old_scene_name = ab.clone();
                self.new_curr_scene_name(Some(ab));
                (false, false)
            }
            (Some(a), Some(b)) if a == b => {
                self.old_scene_name = b;
                self.new_curr_scene_name(Some(a));
                (false, false)
            }
            (Some(good), Some(bad)) if BAD_SCENE_NAMES.contains(&bad.as_str()) && !BAD_SCENE_NAMES.contains(&good.as_str()) => {
                self.old_scene_name = bad;
                self.new_curr_scene_name(Some(good));
                (false, true)
            }
            (Some(bad), Some(good)) if BAD_SCENE_NAMES.contains(&bad.as_str()) && !BAD_SCENE_NAMES.contains(&good.as_str()) => {
                self.old_scene_name = bad;
                self.new_curr_scene_name(Some(good));
                (true, false)
            }
            (Some(a), Some(b)) => {
                // A is at least as up-to-date as B if: B == old || (B == curr && A != curr && A != old)
                if b == self.old_scene_name || (b == self.curr_scene_name && a != self.curr_scene_name && a != self.old_scene_name) {
                    self.old_scene_name = b;
                    self.new_curr_scene_name(Some(a));
                    (false, self.old_scene_name != self.prev_scene_name)
                } else if a == self.old_scene_name || (a == self.curr_scene_name && b != self.curr_scene_name && b != self.old_scene_name) {
                    self.old_scene_name = a;
                    self.new_curr_scene_name(Some(b));
                    (self.old_scene_name != self.prev_scene_name, false)
                } else {
                    asr::print_message(&format!("scene name mismatch: {} vs {}", a, b));
                    (a != self.prev_scene_name, b != self.prev_scene_name)
                }
            }
        }
    }
    pub fn new_next_scene_name(&mut self, mnsn: Option<String>) {
        match mnsn {
            Some(nsn) if nsn != self.next_scene_name => {
                self.next_scene_name = nsn;
                #[cfg(debug_assertions)]
                asr::print_message(&format!("next_scene_name: {}", self.next_scene_name));
                self.new_data_next = !self.next_scene_name.is_empty();
            }
            _ => ()
        }
    }
    pub fn new_next_scene_name1(&mut self, mnsn: Option<String>) -> bool {
        match mnsn {
            None => false,
            Some(bad) if BAD_SCENE_NAMES.contains(&bad.as_str()) => {
                true
            }
            Some(nsn) => {
                self.new_next_scene_name(Some(nsn));
                false
            }
        }
    }

    pub fn transition_pair(&mut self) -> Option<Pair<&str>> {
        if self.new_data_next {
            self.new_data_curr = false;
            self.new_data_next = false;
            Some(Pair{old: &self.curr_scene_name, current: &self.next_scene_name})
        } else if self.new_data_curr {
            self.new_data_curr = false;
            Some(Pair{old: &self.prev_scene_name, current: &self.curr_scene_name})
        } else {
            None
        }
    }
}

pub struct PlayerDataStore {
    map_i32: BTreeMap<u64, i32>,
    map_bool: BTreeMap<u64, bool>,
}

impl PlayerDataStore {
    pub fn new() -> PlayerDataStore {
        PlayerDataStore { 
            map_i32: BTreeMap::new(),
            map_bool: BTreeMap::new(),
        }
    }
    pub fn reset(&mut self) {
        self.map_i32.clear();
        self.map_bool.clear();
    }

    pub fn get_fireball_level(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> i32 {
        match game_manager_finder.get_fireball_level(process) {
            Some(l) if l != 0 || game_manager_finder.is_game_state_playing(process) => {
                self.map_i32.insert(FIREBALL_LEVEL_OFFSET, l);
                l
            }
            _ => {
                *self.map_i32.get(&FIREBALL_LEVEL_OFFSET).unwrap_or(&0)
            }
        }
    }

    pub fn has_dash(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        match game_manager_finder.has_dash(process) {
            Some(k) if k || game_manager_finder.is_game_state_playing(process) => {
                self.map_bool.insert(HAS_DASH_OFFSET, k);
                k
            }
            _ => {
                *self.map_bool.get(&HAS_DASH_OFFSET).unwrap_or(&false)
            }
        }
    }

    pub fn has_wall_jump(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        match game_manager_finder.has_wall_jump(process) {
            Some(w) if w || game_manager_finder.is_game_state_playing(process) => {
                self.map_bool.insert(HAS_WALL_JUMP_OFFSET, w);
                w
            }
            _ => {
                *self.map_bool.get(&HAS_WALL_JUMP_OFFSET).unwrap_or(&false)
            }
        }
    }

    pub fn has_double_jump(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        match game_manager_finder.has_double_jump(process) {
            Some(d) if d || game_manager_finder.is_game_state_playing(process) => {
                self.map_bool.insert(HAS_DOUBLE_JUMP_OFFSET, d);
                d
            }
            _ => {
                *self.map_bool.get(&HAS_DOUBLE_JUMP_OFFSET).unwrap_or(&false)
            }
        }
    }

    pub fn has_acid_armour(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        match game_manager_finder.has_acid_armour(process) {
            Some(a) if a || game_manager_finder.is_game_state_playing(process) => {
                self.map_bool.insert(HAS_ACID_ARMOR_OFFSET, a);
                a
            }
            _ => {
                *self.map_bool.get(&HAS_ACID_ARMOR_OFFSET).unwrap_or(&false)
            }
        }
    }

    pub fn incremented_simple_keys(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        let store_simple_keys = self.map_i32.get(&SIMPLE_KEYS_OFFSET).cloned();
        let player_data_simple_keys = game_manager_finder.get_simple_keys(process);
        if let Some(simple_keys) = player_data_simple_keys {
            if simple_keys != 0 || game_manager_finder.is_game_state_playing(process) {
                self.map_i32.insert(SIMPLE_KEYS_OFFSET, simple_keys);
            }
        }
        match (store_simple_keys, player_data_simple_keys) {
            (Some(prev_simple_keys), Some(simple_keys)) => {
                simple_keys == prev_simple_keys + 1
            }
            _ => false
        }
    }

    pub fn killed_gorgeous_husk(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        match game_manager_finder.killed_gorgeous_husk(process) {
            Some(k) if k || game_manager_finder.is_game_state_playing(process) => {
                self.map_bool.insert(KILLED_GORGEOUS_HUSK_OFFSET, k);
                k
            }
            _ => {
                *self.map_bool.get(&KILLED_GORGEOUS_HUSK_OFFSET).unwrap_or(&false)
            }
        }
    }
}

// --------------------------------------------------------

pub async fn wait_attach_hollow_knight() -> Process {
    retry(|| {
        HOLLOW_KNIGHT_NAMES.into_iter().find_map(Process::attach)
    }).await
}

pub fn scene_path_to_name_string<const N: usize>(scene_path: ArrayCString<N>) -> String {
    String::from_utf8(get_scene_name(&scene_path).to_vec()).unwrap()
}

fn read_string_object<const N: usize>(process: &Process, a: Address64) -> Option<String> {
    let n: u32 = process.read_pointer_path64(a, &[STRING_LEN_OFFSET]).ok()?;
    if !(n < 2048) { return None; }
    let w: ArrayWString<N> = process.read_pointer_path64(a, &[STRING_CONTENTS_OFFSET]).ok()?;
    if !(w.len() == min(n as usize, N)) { return None; }
    String::from_utf16(&w.to_vec()).ok()
}

// --------------------------------------------------------
// --------------------------------------------------------

pub fn is_menu(s: &str) -> bool {
    s == MENU_TITLE || s == QUIT_TO_MENU
}

pub fn is_play_scene(s: &str) -> bool {
    !NON_PLAY_SCENES.contains(&s) && !BAD_SCENE_NAMES.contains(&s)
}

// --------------------------------------------------------

// Logging in debug_assertions mode

#[cfg(debug_assertions)]
fn log_scene_table(scene_table: &SceneTable) {
    // Log scene_table as json
    if let Ok(j) = serde_json::to_string_pretty(&scene_table) {
        asr::print_message(&format!("begin scene_table.json\n{}", j));
    }
}

#[cfg(debug_assertions)]
pub fn update_scene_table(process: &Process, scene_manager: &SceneManager, scene_table: &mut SceneTable) {
    let si = scene_manager.get_current_scene_index(&process).unwrap_or(-1);
    let sp: ArrayCString<SCENE_PATH_SIZE> = scene_manager.get_current_scene_path(&process).unwrap_or_default();
    let sn = scene_path_to_name_string(sp);
    let sv = SceneInfo{name: sn.clone(), path: String::from_utf8(sp.to_vec()).unwrap()};
    if let Some(tv) = scene_table.get(&si) {
        assert_eq!(&sv, tv);
    } else if si == -1 {
        assert_eq!(sv, SceneInfo{name: "".to_string(), path: "".to_string()});
    } else {
        scene_table.insert(si, sv);
        log_scene_table(scene_table);
    }
}
