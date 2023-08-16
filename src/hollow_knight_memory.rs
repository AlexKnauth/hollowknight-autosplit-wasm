
use std::cmp::min;
use std::mem;
use std::collections::BTreeMap;
use asr::future::{next_tick, retry};
use asr::signature::Signature;
use asr::watcher::Pair;
use asr::{Process, Address, Address64};
use asr::game_engine::unity::{SceneManager, get_scene_name};
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
const UNITY_PLAYER_HAS_ACTIVE_SCENE_OFFSETS: [u64; 14] = [
    0x01A1AC30, // Windows
    0x01A862E8, // Mac?
    0x01A982E8, // Mac?
    0x01AA12E8, // Mac?
    0x01AA22E8, // Mac?
    0x01AA32E8, // Mac?
    0x01AA52E8, // Mac?
    0x01AAF2E8, // Mac?
    0x01AB02E8, // Mac?
    0x01BB32E8, // Mac?
    0x01BB42E8, // Mac?
    0x01BBD2E8, // Mac?
    0x01BBE2E8, // Mac?
    0x01BD82E8, // Mac?
];

const ITER_PER_TICK: u64 = 16384;

const ASSETS_SCENES: &str = "Assets/Scenes/";
const ASSETS_SCENES_LEN: usize = ASSETS_SCENES.len();

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

const BAD_SCENE_NAMES: [&str; 10] = [
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
];

const UNITY_PLAYER_HAS_GAME_MANAGER_OFFSETS: [u64; 10] = [
    0x019D7CF0, // Windows
    0x01ADDA80, // Mac?
    0x01AE6A80, // Mac?
    0x01AE7A80, // Mac?
    0x01AEAA80, // Mac?
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

// Base number of masks, without any charms, bindings, lifeblood, or damage taken
const MAX_HEALTH_BASE_OFFSET: u64 = 0x198;
const MAX_HEALTH_BASE_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    MAX_HEALTH_BASE_OFFSET
];

// Heart pieces represents one of:
//  - number of heart pieces including the ones assembled into masks: 0-3 4-7 8-11 12-15 16
//  - number of heart pieces excluding the ones assembled into masks: 0-3 0-3 0-3  0-3   0
//  - number of heart pieces excluding masks except the final mask:   0-3 0-3 0-3  0-3   4
// and I'm not sure which one
const HEART_PIECES_OFFSET: u64 = 0x1a8;
const HEART_PIECES_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    HEART_PIECES_OFFSET
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

// Dashmaster
const GOT_CHARM_31_OFFSET: u64 = 0x5c9;
const GOT_CHARM_31_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    GOT_CHARM_31_OFFSET
];

const GRUBS_COLLECTED_OFFSET: u64 = 0xb94;
const GRUBS_COLLECTED_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    GRUBS_COLLECTED_OFFSET
];

// Gruz Mother
const KILLED_BIG_FLY_OFFSET: u64 = 0x6c1;
const KILLED_BIG_FLY_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    KILLED_BIG_FLY_OFFSET
];

const SLY_RESCUED_OFFSET: u64 = 0x389;
const SLY_RESCUED_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    SLY_RESCUED_OFFSET
];

const KILLED_GORGEOUS_HUSK_OFFSET: u64 = 0x879;
const KILLED_GORGEOUS_HUSK_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    KILLED_GORGEOUS_HUSK_OFFSET
];

// Lemm
const MET_RELIC_DEALER_SHOP_OFFSET: u64 = 0x34a;
const MET_RELIC_DEALER_SHOP_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    MET_RELIC_DEALER_SHOP_OFFSET
];

const WATCHER_CHANDELIER_OFFSET: u64 = 0xc8d;
const WATCHER_CHANDELIER_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    WATCHER_CHANDELIER_OFFSET
];

const KILLED_BLACK_KNIGHT_OFFSET: u64 = 0x7f9;
const KILLED_BLACK_KNIGHT_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    KILLED_BLACK_KNIGHT_OFFSET
];

const KILLED_MEGA_JELLYFISH_OFFSET: u64 = 0x7a1;
const KILLED_MEGA_JELLYFISH_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    KILLED_MEGA_JELLYFISH_OFFSET
];

const SPIDER_CAPTURE_OFFSET: u64 = 0xca0;
const SPIDER_CAPTURE_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    SPIDER_CAPTURE_OFFSET
];

const UNCHAINED_HOLLOW_KNIGHT_OFFSET: u64 = 0xcc9;
const UNCHAINED_HOLLOW_KNIGHT_PATH: &[u64] = &[
    // from game_manager
    PLAYER_DATA_OFFSET,
    UNCHAINED_HOLLOW_KNIGHT_OFFSET
];

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
    async fn attempt_scan_unity_player(process: &Process, unity_player: (Address, u64)) -> Option<UnityPlayerHasActiveScene> {
        asr::print_message("Scanning for active_scene roots...");
        let a = attempt_scan_roots(process, unity_player, attach_active_scene_root, &()).await?;
        Some(UnityPlayerHasActiveScene(a))
    }
    async fn attach_scan(process: &Process) -> Option<UnityPlayerHasActiveScene> {
        let unity_player = get_unity_player_range(process)?;
        if let Some(uphas) = UnityPlayerHasActiveScene::attach_unity_player(process, unity_player) {
            Some(uphas)
        } else {
            next_tick().await;
            UnityPlayerHasActiveScene::attempt_scan_unity_player(process, unity_player).await
        }
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
    async fn attach(process: &Process) -> Option<SceneFinder> {
        if let Some(scene_manager) = SceneManager::attach(process) {
            return Some(SceneFinder::SceneManager(scene_manager));
        }
        if let Some(uphas) = UnityPlayerHasActiveScene::attach_scan(process).await {
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
        loop {
            if let Some(scene_finder) = SceneFinder::attach(&process).await {
                asr::print_message("Attached SceneFinder.");
                return scene_finder;
            }
            next_tick().await;
        }
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
        loop {
            if let Some(g) = GameManagerFinder::attach_scan(process, scene_finder).await {
                asr::print_message("Attached GameManagerFinder.");
                // asr::print_message(&format!("GameManagerFinder found uphgm__ Ok({:?})", g.unity_player_has_game_manager));
                // asr::print_message(&format!("GameManagerFinder found uphgm_0 {:?}", process.read_pointer_path64::<Address64>(g.unity_player_has_game_manager, &[UPHGM_OFFSET_0])));
                // asr::print_message(&format!("GameManagerFinder found uphgm_1 {:?}", process.read_pointer_path64::<Address64>(g.unity_player_has_game_manager, &[UPHGM_OFFSET_0, UPHGM_OFFSET_1])));
                // asr::print_message(&format!("GameManagerFinder found uphgm_2 {:?}", process.read_pointer_path64::<Address64>(g.unity_player_has_game_manager, &[UPHGM_OFFSET_0, UPHGM_OFFSET_1, UPHGM_OFFSET_2])));
                // asr::print_message(&format!("GameManagerFinder found uphgm_3 {:?}", process.read_pointer_path64::<Address64>(g.unity_player_has_game_manager, &[UPHGM_OFFSET_0, UPHGM_OFFSET_1, UPHGM_OFFSET_2, UPHGM_OFFSET_3])));
                asr::print_message(&format!("GameManagerFinder found ___gm__ Ok({:?})", g.game_manager));
                asr::print_message("Scanning for signatures...");
                if let Some(a) = signature_scan_all(&process) {
                    asr::print_message(&format!("Sig found a_10? {:?}", process.read_pointer_path64::<Address64>(a, &[10])));
                    asr::print_message(&format!("Sig found a_10_0? {:?}", process.read_pointer_path64::<Address64>(a, &[10, 0])));
                } else {
                    asr::print_message("Sig not found");
                }
                return g;
            }
            next_tick().await;
        }
    }
    async fn attach_scan(process: &Process, scene_finder: &SceneFinder) -> Option<GameManagerFinder> {
        let scene_name = scene_finder.get_current_scene_name(&process).ok()?;
        if scene_name == PRE_MENU_INTRO { return None; }
        let unity_player = get_unity_player_range(process)?;
        if let Some(g) = GameManagerFinder::attach_unity_player(process, unity_player, &scene_name) {
            Some(g)
        } else {
            next_tick().await;
            GameManagerFinder::attempt_scan_unity_player(process, unity_player, &scene_name).await
        }
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
    async fn attempt_scan_unity_player(process: &Process, unity_player: (Address, u64), scene_name: &str) -> Option<GameManagerFinder> {
        asr::print_message(&format!("Scanning for game_manager_scene_name {}...", scene_name));
        next_tick().await;
        let unity_player_has_game_manager = attempt_scan_roots(process, unity_player, attach_game_manager_scene_name, scene_name).await?;
        let game_manager: Address64 = process.read_pointer_path64(unity_player_has_game_manager, GAME_MANAGER_PATH).ok()?;
        Some(GameManagerFinder{unity_player_has_game_manager, game_manager, dirty: false})
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub fn set_dirty(&mut self) {
        self.dirty = true;
    }

    pub async fn attempt_clean(&mut self, process: &Process, scene_finder: &SceneFinder) -> Option<()> {
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
        next_tick().await;
        let unity_player_has_game_manager = attempt_scan_roots(process, unity_player, attach_game_manager_scene_name, &scene_name).await?;
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

    pub fn get_game_state(&self, process: &Process) -> Option<i32> {
        let ui_manager_vanilla: Address64 = process.read_pointer_path64(self.game_manager, &[UI_MANAGER_VANILLA_OFFSET]).ok()?;
        let game_state_offset = if ui_manager_vanilla.is_null() { GAME_STATE_MODDING_API_OFFSET } else { GAME_STATE_VANILLA_OFFSET };
        process.read_pointer_path64(self.game_manager, &[game_state_offset]).ok()
    }

    fn is_game_state_playing(&self, process: &Process) -> bool {
        self.get_game_state(process) == Some(GAME_STATE_PLAYING)
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

    pub fn max_health_base(&self, process: &Process) -> Option<i32> {
        process.read_pointer_path64(self.game_manager, MAX_HEALTH_BASE_PATH).ok()
    }

    pub fn heart_pieces(&self, process: &Process) -> Option<i32> {
        process.read_pointer_path64(self.game_manager, HEART_PIECES_PATH).ok()
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

    // Dashmaster
    pub fn got_charm_31(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, GOT_CHARM_31_PATH).ok()
    }

    pub fn grubs_collected(&self, process: &Process) -> Option<i32> {
        process.read_pointer_path64(self.game_manager, GRUBS_COLLECTED_PATH).ok()
    }

    // Gruz Mother
    pub fn killed_big_fly(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, KILLED_BIG_FLY_PATH).ok()
    }

    pub fn sly_rescued(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, SLY_RESCUED_PATH).ok()
    }

    pub fn killed_gorgeous_husk(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, KILLED_GORGEOUS_HUSK_PATH).ok()
    }

    // Lemm
    pub fn met_relic_dealer_shop(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, MET_RELIC_DEALER_SHOP_PATH).ok()
    }

    pub fn watcher_chandelier(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, WATCHER_CHANDELIER_PATH).ok()
    }

    pub fn killed_black_knight(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, KILLED_BLACK_KNIGHT_PATH).ok()
    }

    pub fn killed_mega_jellyfish(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, KILLED_MEGA_JELLYFISH_PATH).ok()
    }

    pub fn spider_capture(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, SPIDER_CAPTURE_PATH).ok()
    }

    pub fn unchained_hollow_knight(&self, process: &Process) -> Option<bool> {
        process.read_pointer_path64(self.game_manager, UNCHAINED_HOLLOW_KNIGHT_PATH).ok()
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

async fn attempt_scan_roots<F, V>(process: &Process, unity_player: (Address, u64), attach: F, v: &V) -> Option<Address>
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
        if 0 == i % ITER_PER_TICK {
            next_tick().await;
        }
    }
    None
}

/*
            gameManager = new ProgramPointer(
                new FindPointerSignature(PointerVersion.Normal64, AutoDeref.Single, "41FFD3E96300000048B8????????????????488B10488BCE488D6424009049BB", 10),
                new FindPointerSignature(PointerVersion.Normal64, AutoDeref.Single32, "488BCE49BB????????????????41FFD3E9??000000488B1425", 25),
                new FindPointerSignature(PointerVersion.Normal, AutoDeref.Single, "83C41083EC0C57E8????????83C410EB3D8B05", 19),
                new FindPointerSignature(PointerVersion.API, AutoDeref.Single, "83C41083EC0C57393FE8????????83C410EB3F8B05", 21)) { UpdatedPointer = UpdatedPointer };
*/

const SIG_1: Signature<32> = Signature::new("41FFD3E96300000048B8????????????????488B10488BCE488D6424009049BB");
const SIG_2: Signature<25> = Signature::new("488BCE49BB????????????????41FFD3E9??000000488B1425");
const SIG_3: Signature<19> = Signature::new("83C41083EC0C57E8????????83C410EB3D8B05");
const SIG_4: Signature<21> = Signature::new("83C41083EC0C57393FE8????????83C410EB3F8B05");

fn signature_scan_range(process: &Process, range: (Address, u64)) -> Option<Address> {
    if let Some(a) = SIG_1.scan_process_range(process, range) {
        asr::print_message(&format!("SIG_1 found {}", a));
        Some(a)
    } else if let Some(a) = SIG_2.scan_process_range(process, range) {
        asr::print_message(&format!("SIG_2 found {}", a));
        Some(a)
    } else if let Some(a) =  SIG_3.scan_process_range(process, range) {
        asr::print_message(&format!("SIG_3 found {}", a));
        Some(a)
    } else if let Some(a) = SIG_4.scan_process_range(process, range) {
        asr::print_message(&format!("SIG_4 found {}", a));
        Some(a)
    } else {
        None
    }
}

fn signature_scan_all(process: &Process) -> Option<Address> {
    for r in process.memory_ranges() {
        if let Ok(range) = r.range() {
            if let Some(a) = signature_scan_range(process, range) {
                return Some(a);
            }
        }
    }
    None
}

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
pub fn update_scene_table(process: &Process, scene_finder: &SceneFinder, scene_table: &mut SceneTable) {
    let si = scene_finder.get_current_scene_index(&process).unwrap_or(-1);
    let sp: ArrayCString<SCENE_PATH_SIZE> = scene_finder.get_current_scene_path(&process).unwrap_or_default();
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
