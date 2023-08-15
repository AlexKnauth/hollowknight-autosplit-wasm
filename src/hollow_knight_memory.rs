
use std::cmp::min;
use std::mem;
use std::collections::BTreeMap;
use asr::future::{next_tick, retry};
use asr::watcher::Pair;
use asr::{Process, Address, Address64};
use asr::game_engine::unity::{SceneManager, get_scene_name};
use asr::string::{ArrayCString, ArrayWString};

// --------------------------------------------------------

const HOLLOW_KNIGHT_NAMES: [&str; 2] = [
    "hollow_knight.exe", // Windows
    "Hollow Knight", // Mac
];

const UNITY_PLAYER_NAMES: [&str; 2] = [
    "UnityPlayer.dll", // Windows
    "UnityPlayer.dylib", // Mac
];

pub const SCENE_PATH_SIZE: usize = 64;

const STRING_LEN_OFFSET: u64 = 0x10;
const STRING_CONTENTS_OFFSET: u64 = 0x14;

const SCENE_ASSET_PATH_OFFSET: u64 = 0x10;
#[cfg(debug_assertions)]
const SCENE_BUILD_INDEX_OFFSET: u64 = 0x98;
const ACTIVE_SCENE_OFFSET: u64 = 0x48;
const ACTIVE_SCENE_CONTENTS_PATH: &[u64] = &[0, ACTIVE_SCENE_OFFSET, SCENE_ASSET_PATH_OFFSET, 0];
const UNITY_PLAYER_HAS_ACTIVE_SCENE_OFFSETS: [u64; 12] = [
    0x01A1AC30, // Windows
    0x01A862E8, // Mac?
    0x01A982E8, // Mac?
    0x01AA22E8, // Mac?
    0x01AA32E8, // Mac?
    0x01AAF2E8, // Mac?
    0x01AB02E8, // Mac?
    0x01BB32E8, // Mac?
    0x01BB42E8, // Mac?
    0x01BBD2E8, // Mac?
    0x01BBE2E8, // Mac?
    0x01BD82E8, // Mac?
];

const ASSETS_SCENES: &str = "Assets/Scenes/";
const ASSETS_SCENES_LEN: usize = ASSETS_SCENES.len();

const PRE_MENU_INTRO: &str = "Pre_Menu_Intro";
pub const MENU_TITLE: &str = "Menu_Title";
pub const QUIT_TO_MENU: &str = "Quit_To_Menu";

const BAD_SCENE_NAMES: [&str; 9] = [
    "Untagged",
    "left1",
    "oncomplete",
    "Attack Range",
    "onstart",
    "position",
    "looptype",
    "integer1",
    "gameObject",
];

const UNITY_PLAYER_HAS_GAME_MANAGER_OFFSETS: [u64; 8] = [
    0x019D7CF0, // Windows
    0x01ADDA80, // Mac?
    0x01AE7A80, // Mac?
    0x01BF8A80, // Mac?
    0x01BF9A80, // Mac?
    0x01C02A80, // Mac?
    0x01C03A80, // Mac?
    0x01C1DA80, // Mac?
];

const UPHGM_OFFSET_0: u64 = 0;
const UPHGM_OFFSET_1: u64 = 0x10;
const UPHGM_OFFSET_2: u64 = 0x80;
const UPHGM_OFFSET_3: u64 = 0x28;
const UPHGM_OFFSET_4: u64 = 0x38;
const GAME_MANAGER_PATH: &[u64] = &[
    UPHGM_OFFSET_0,
    UPHGM_OFFSET_1,
    UPHGM_OFFSET_2,
    UPHGM_OFFSET_3,
    UPHGM_OFFSET_4
];

const SCENE_NAME_OFFSET: u64 = 0x18;
const NEXT_SCENE_NAME_OFFSET: u64 = 0x20;
const SCENE_NAME_PATH: &[u64] = &[
    // from game_manager
    SCENE_NAME_OFFSET
];
const NEXT_SCENE_NAME_PATH: &[u64] = &[
    // from game_manager
    NEXT_SCENE_NAME_OFFSET
];

const UI_MANAGER_VANILLA_OFFSET: u64 = 0xa0;
const GAME_STATE_VANILLA_OFFSET: u64 = 0x174;
const GAME_STATE_MODDING_API_OFFSET: u64 = 0x184;
const GAME_STATE_PLAYING: i32 = 4;

const PLAYER_DATA_OFFSET: u64 = 0xc8;

const FIREBALL_LEVEL_OFFSET: u64 = 0x260;
const FIREBALL_LEVEL_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    FIREBALL_LEVEL_OFFSET
];

const HAS_DASH_OFFSET: u64 = 0x284;
const HAS_DASH_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    HAS_DASH_OFFSET
];

const HAS_SHADOW_DASH_OFFSET: u64 = 0x287;
const HAS_SHADOW_DASH_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    HAS_SHADOW_DASH_OFFSET
];

const HAS_WALL_JUMP_OFFSET: u64 = 0x285;
const HAS_WALL_JUMP_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    HAS_WALL_JUMP_OFFSET
];

const HAS_DOUBLE_JUMP_OFFSET: u64 = 0x289;
const HAS_DOUBLE_JUMP_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    HAS_DOUBLE_JUMP_OFFSET
];

const HAS_SUPER_DASH_OFFSET: u64 = 0x286;
const HAS_SUPER_DASH_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    HAS_SUPER_DASH_OFFSET
];

const HAS_ACID_ARMOR_OFFSET: u64 = 0x288;
const HAS_ACID_ARMOR_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    HAS_ACID_ARMOR_OFFSET
];

const HAS_DREAM_NAIL_OFFSET: u64 = 0x271;
const HAS_DREAM_NAIL_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    HAS_DREAM_NAIL_OFFSET
];

const HAS_DREAM_GATE_OFFSET: u64 = 0x272;
const HAS_DREAM_GATE_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    HAS_DREAM_GATE_OFFSET
];

const DREAM_NAIL_UPGRADED_OFFSET: u64 = 0x273;
const DREAM_NAIL_UPGRADED_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    DREAM_NAIL_UPGRADED_OFFSET
];

const HAS_LANTERN_OFFSET: u64 = 0x28a;
const HAS_LANTERN_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    HAS_LANTERN_OFFSET
];

const SIMPLE_KEYS_OFFSET: u64 = 0x2d8;
const SIMPLE_KEYS_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    SIMPLE_KEYS_OFFSET
];

const HAS_SLY_KEY_OFFSET: u64 = 0x28e;
const HAS_SLY_KEY_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    HAS_SLY_KEY_OFFSET
];

const HAS_WHITE_KEY_OFFSET: u64 = 0x290;
const HAS_WHITE_KEY_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    HAS_WHITE_KEY_OFFSET
];

#[cfg(debug_assertions)]
const GEO_OFFSET: u64 = 0x1c4;
#[cfg(debug_assertions)]
const GEO_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    GEO_OFFSET
];

// --------------------------------------------------------

pub struct UnityPlayerHasActiveScene(Address);

impl UnityPlayerHasActiveScene {
    fn attach_unity_player(process: &Process, unity_player: (Address, u64)) -> Option<UnityPlayerHasActiveScene> {
        let (addr, _) = unity_player;
        for offset in UNITY_PLAYER_HAS_ACTIVE_SCENE_OFFSETS.iter() {
            if let Some(a) = attach_active_scene_root(process, addr.add(*offset), &()) {
                return Some(UnityPlayerHasActiveScene(a));
            }
        }
        None
    }
    fn attempt_scan_unity_player(process: &Process, unity_player: (Address, u64)) -> Option<UnityPlayerHasActiveScene> {
        asr::print_message("Scanning for active_scene roots...");
        let a = attempt_scan_roots(process, unity_player, attach_active_scene_root, &())?;
        Some(UnityPlayerHasActiveScene(a))
    }
    fn attach_scan(process: &Process) -> Option<UnityPlayerHasActiveScene> {
        let unity_player = get_unity_player_range(process)?;
        UnityPlayerHasActiveScene::attach_unity_player(process, unity_player).or_else(|| {
            UnityPlayerHasActiveScene::attempt_scan_unity_player(process, unity_player)
        })
    }

    #[cfg(debug_assertions)]
    fn get_current_scene_index(&self, process: &Process) -> Result<i32, asr::Error> {
        process.read_pointer_path64(self.0, &[0, ACTIVE_SCENE_OFFSET, SCENE_BUILD_INDEX_OFFSET])
    }

    fn get_current_scene_path<const N: usize>(&self, process: &Process) -> Result<ArrayCString<N>, asr::Error> {
        process.read_pointer_path64(self.0, &[0, ACTIVE_SCENE_OFFSET, SCENE_ASSET_PATH_OFFSET, 0])
    }
}

pub enum SceneFinder {
    SceneManager(SceneManager),
    UnityPlayerHasActiveScene(UnityPlayerHasActiveScene)
}

impl SceneFinder {
    fn attach(process: &Process) -> Option<SceneFinder> {
        if let Some(scene_manager) = SceneManager::attach(process) {
            return Some(SceneFinder::SceneManager(scene_manager));
        }
        if let Some(uphas) = UnityPlayerHasActiveScene::attach_scan(process) {
            return Some(SceneFinder::UnityPlayerHasActiveScene(uphas))
        }
        None
    }
    pub async fn wait_attach(process: &Process) -> SceneFinder {
        asr::print_message("Trying to attach SceneManager...");
        next_tick().await;
        for i in 0 .. 10 {
            if let Some(scene_manager) = SceneManager::attach(&process) {
                asr::print_message(&format!("Attached SceneManager ({}).", i));
                return SceneFinder::SceneManager(scene_manager)
            }
            next_tick().await;
        }
        
        asr::print_message("Trying to attach SceneFinder...");
        next_tick().await;
        let scene_finder = retry(|| SceneFinder::attach(&process)).await;
        asr::print_message("Attached SceneFinder.");
        scene_finder
    }

    #[cfg(debug_assertions)]
    pub fn get_current_scene_index(&self, process: &Process) -> Result<i32, asr::Error> {
        match self {
            SceneFinder::SceneManager(scene_manager) => {
                scene_manager.get_current_scene_index(process)
            }
            SceneFinder::UnityPlayerHasActiveScene(uphas) => {
                uphas.get_current_scene_index(process)
            }
        }
    }

    pub fn get_current_scene_path<const N: usize>(&self, process: &Process) -> Result<ArrayCString<N>, asr::Error> {
        match self {
            SceneFinder::SceneManager(scene_manager) => {
                scene_manager.get_current_scene_path(process)
            }
            SceneFinder::UnityPlayerHasActiveScene(uphas) => {
                uphas.get_current_scene_path(process)
            }
        }
    }

    pub fn get_current_scene_name(&self, process: &Process) -> Result<String, asr::Error> {
        self.get_current_scene_path::<SCENE_PATH_SIZE>(&process).map(scene_path_to_name_string)
    }

    async fn wait_get_current_scene_path<const N: usize>(&self, process: &Process) -> ArrayCString<N> {
        retry(|| self.get_current_scene_path(&process)).await
    }

    pub async fn wait_get_current_scene_name(&self, process: &Process) -> String {
        scene_path_to_name_string(self.wait_get_current_scene_path::<SCENE_PATH_SIZE>(&process).await)
    }
}

pub struct GameManagerFinder {
    unity_player_has_game_manager: Address,
    game_manager: Address64,
    dirty: bool
}

impl GameManagerFinder {
    pub async fn wait_attach(process: &Process, scene_finder: &SceneFinder) -> GameManagerFinder {
        asr::print_message("Trying to attach GameManagerFinder...");
        let g = retry(|| GameManagerFinder::attach_scan(process, scene_finder)).await;
        asr::print_message("Attached GameManagerFinder.");
        g
    }
    fn attach_scan(process: &Process, scene_finder: &SceneFinder) -> Option<GameManagerFinder> {
        let scene_name = scene_finder.get_current_scene_name(&process).ok()?;
        if scene_name == PRE_MENU_INTRO { return None; }
        let unity_player = get_unity_player_range(process)?;
        GameManagerFinder::attach_unity_player(process, unity_player, &scene_name).or_else(|| {
            GameManagerFinder::attempt_scan_unity_player(process, unity_player, &scene_name)
        })
    }
    fn attach_unity_player(process: &Process, unity_player: (Address, u64), scene_name: &str) -> Option<GameManagerFinder> {
        let (addr, _) = unity_player;
        for offset in UNITY_PLAYER_HAS_GAME_MANAGER_OFFSETS.iter() {
            if let Some(unity_player_has_game_manager) = attach_game_manager_scene_name(process, addr.add(*offset), scene_name) {
                let game_manager: Address64 = process.read_pointer_path64(unity_player_has_game_manager, GAME_MANAGER_PATH).ok()?;
                return Some(GameManagerFinder{unity_player_has_game_manager, game_manager, dirty: false});
            }
        }
        None
    }
    fn attempt_scan_unity_player(process: &Process, unity_player: (Address, u64), scene_name: &str) -> Option<GameManagerFinder> {
        asr::print_message(&format!("Scanning for game_manager_scene_name {}...", scene_name));
        let unity_player_has_game_manager = attempt_scan_roots(process, unity_player, attach_game_manager_scene_name, scene_name)?;
        let game_manager: Address64 = process.read_pointer_path64(unity_player_has_game_manager, GAME_MANAGER_PATH).ok()?;
        Some(GameManagerFinder{unity_player_has_game_manager, game_manager, dirty: false})
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub fn set_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn attempt_clean(&mut self, process: &Process, scene_finder: &SceneFinder) -> Option<()> {
        if !self.is_dirty() { return Some(()); }
        let scene_name = scene_finder.get_current_scene_name(&process).ok()?;
        if self.get_scene_name(process).is_some_and(|s| s == scene_name) {
            self.dirty = false;
            return Some(());
        }
        if scene_name == PRE_MENU_INTRO { return None; }
        let unity_player = get_unity_player_range(process)?;
        if let Some(unity_player_has_game_manager) = attach_game_manager_scene_name(process, self.unity_player_has_game_manager, &scene_name) {
            let game_manager: Address64 = process.read_pointer_path64(unity_player_has_game_manager, GAME_MANAGER_PATH).ok()?;
            self.game_manager = game_manager;
            self.dirty = false;
            return Some(());
        }
        let (addr, _) = unity_player;
        for offset in UNITY_PLAYER_HAS_GAME_MANAGER_OFFSETS.iter() {
            if let Some(unity_player_has_game_manager) = attach_game_manager_scene_name(process, addr.add(*offset), &scene_name) {
                let game_manager: Address64 = process.read_pointer_path64(unity_player_has_game_manager, GAME_MANAGER_PATH).ok()?;
                self.unity_player_has_game_manager = unity_player_has_game_manager;
                self.game_manager = game_manager;
                self.dirty = false;
                return Some(());
            }
        }
        asr::print_message(&format!("Scanning for game_manager_scene_name {}...", scene_name));
        let unity_player_has_game_manager = attempt_scan_roots(process, unity_player, attach_game_manager_scene_name, &scene_name)?;
        let game_manager: Address64 = process.read_pointer_path64(unity_player_has_game_manager, GAME_MANAGER_PATH).ok()?;
        self.unity_player_has_game_manager = unity_player_has_game_manager;
        self.game_manager = game_manager;
        self.dirty = false;
        Some(())
    }

    pub fn get_scene_name(&self, process: &Process) -> Option<String> {
        let s = process.read_pointer_path64(self.game_manager, SCENE_NAME_PATH).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, s)
    }

    pub fn get_next_scene_name(&self, process: &Process) -> Option<String> {
        let s = process.read_pointer_path64(self.game_manager, NEXT_SCENE_NAME_PATH).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, s)
    }

    fn get_game_state(&self, process: &Process) -> Option<i32> {
        let ui_manager_vanilla: Address64 = process.read_pointer_path64(self.game_manager, &[UI_MANAGER_VANILLA_OFFSET]).ok()?;
        let game_state_offset = if ui_manager_vanilla.is_null() { GAME_STATE_MODDING_API_OFFSET } else { GAME_STATE_VANILLA_OFFSET };
        process.read_pointer_path64(self.game_manager, &[game_state_offset]).ok()
    }

    pub fn get_fireball_level(&self, process: &Process) -> Option<i32> {
        process.read_pointer_path64(self.game_manager, FIREBALL_LEVEL_PATH).ok()
    }

    pub fn has_dash(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, HAS_DASH_PATH).ok()
    }

    pub fn has_shadow_dash(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, HAS_SHADOW_DASH_PATH).ok()
    }

    pub fn has_wall_jump(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, HAS_WALL_JUMP_PATH).ok()
    }

    pub fn has_double_jump(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, HAS_DOUBLE_JUMP_PATH).ok()
    }

    pub fn has_super_dash(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, HAS_SUPER_DASH_PATH).ok()
    }

    pub fn has_acid_armour(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, HAS_ACID_ARMOR_PATH).ok()
    }

    pub fn has_dream_nail(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, HAS_DREAM_NAIL_PATH).ok()
    }

    pub fn has_dream_gate(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, HAS_DREAM_GATE_PATH).ok()
    }
    
    pub fn dream_nail_upgraded(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, DREAM_NAIL_UPGRADED_PATH).ok()
    }

    pub fn has_lantern(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, HAS_LANTERN_PATH).ok()
    }

    pub fn get_simple_keys(&self, process: &Process) -> Option<i32> {
        process.read_pointer_path64(self.game_manager, SIMPLE_KEYS_PATH).ok()
    }

    pub fn has_sly_key(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, HAS_SLY_KEY_PATH).ok()
    }

    pub fn has_white_key(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, HAS_WHITE_KEY_PATH).ok()
    }

    #[cfg(debug_assertions)]
    pub fn get_geo(&self, process: &Process) -> Option<i32> {
        process.read_pointer_path64(self.game_manager, GEO_PATH).ok()
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
    map_i32: BTreeMap<u64, i32>
}

impl PlayerDataStore {
    pub fn new() -> PlayerDataStore {
        PlayerDataStore { map_i32: BTreeMap::new() }
    }
    pub fn reset(&mut self) {
        self.map_i32.clear();
    }
    
    pub fn incremented_simple_keys(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        let store_simple_keys = self.map_i32.get(&SIMPLE_KEYS_OFFSET).cloned();
        let player_data_simple_keys = game_manager_finder.get_simple_keys(process);
        if let Some(simple_keys) = player_data_simple_keys {
            if simple_keys != 0 || game_manager_finder.get_game_state(process) == Some(GAME_STATE_PLAYING) {
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

fn get_unity_player_range(process: &Process) -> Option<(Address, u64)> {
    UNITY_PLAYER_NAMES.into_iter().find_map(|name| {
        process.get_module_range(name).ok()
    })
}

fn read_string_object<const N: usize>(process: &Process, a: Address64) -> Option<String> {
    let n: u32 = process.read_pointer_path64(a, &[STRING_LEN_OFFSET]).ok()?;
    if !(n < 2048) { return None; }
    let w: ArrayWString<N> = process.read_pointer_path64(a, &[STRING_CONTENTS_OFFSET]).ok()?;
    if !(w.len() == min(n as usize, N)) { return None; }
    String::from_utf16(&w.to_vec()).ok()
}

// --------------------------------------------------------

// Scanning for values in memory

fn attach_active_scene_root(process: &Process, a: Address, _v: &()) -> Option<Address> {
    let s1: ArrayCString<ASSETS_SCENES_LEN> = process.read_pointer_path64(a, ACTIVE_SCENE_CONTENTS_PATH).ok()?;
    let s2: String = String::from_utf8(s1.to_vec()).ok()?;
    if s2 == ASSETS_SCENES {
        Some(a)
    } else {
        None
    }
}

fn attach_game_manager_scene_name(process: &Process, a: Address, scene_name: &str) -> Option<Address> {
    let gm: Address64 = process.read_pointer_path64(a, GAME_MANAGER_PATH).ok()?;
    let s0: Address64 = process.read_pointer_path64(gm, SCENE_NAME_PATH).ok()?;
    let s2: String = read_string_object::<SCENE_PATH_SIZE>(process, s0)?;
    if s2 == scene_name {
        Some(a)
    } else {
        None
    }
}

fn attempt_scan_roots<F, V>(process: &Process, unity_player: (Address, u64), attach: F, v: &V) -> Option<Address>
where
    F: Fn(&Process, Address, &V) -> Option<Address>,
    V: ?Sized,
{
    let (addr, len) = unity_player;
    for i in 0 .. (len / 8) {
        let a = addr.add(i * 8);
        if let Some(a) = attach(process, a, v) {
            let offset = a.value() - addr.value();
            asr::print_message(&format!("Found UnityPlayer + 0x{:X}", offset));
            return Some(a);
        }
    }
    None
}
