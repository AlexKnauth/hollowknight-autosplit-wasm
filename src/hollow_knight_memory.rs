
use core::cell::OnceCell;
use std::cmp::min;
use std::mem;
use std::collections::BTreeMap;
use asr::future::retry;
use asr::watcher::Pair;
use asr::{Process, Address64};
use asr::game_engine::unity::mono::{self, Pointer};
use asr::string::ArrayWString;

#[cfg(debug_assertions)]
use std::string::String;

// --------------------------------------------------------

const HOLLOW_KNIGHT_NAMES: [&str; 2] = [
    "hollow_knight.exe", // Windows
    "Hollow Knight", // Mac
];

pub const SCENE_PATH_SIZE: usize = 64;

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

pub const GAME_STATE_INACTIVE: i32 = 0;
pub const GAME_STATE_MAIN_MENU: i32 = 1;
pub const GAME_STATE_LOADING: i32 = 2;
pub const GAME_STATE_ENTERING_LEVEL: i32 = 3;
pub const GAME_STATE_PLAYING: i32 = 4;
pub const GAME_STATE_EXITING_LEVEL: i32 = 6;

pub const UI_STATE_PLAYING: i32 = 6;
pub const UI_STATE_PAUSED: i32 = 7;

pub const HERO_TRANSITION_STATE_WAITING_TO_ENTER_LEVEL: i32 = 2;

struct GameManagerPointers {
    scene_name: Pointer<2>,
    next_scene_name: Pointer<2>,
    game_state: Pointer<2>,
    ui_state_vanilla: Pointer<3>,
    ui_state_modded: Pointer<3>,
    camera_teleporting: Pointer<3>,
    hazard_respawning: Pointer<4>,
    accepting_input: Pointer<3>,
    hero_transition_state: Pointer<3>,
    tile_map_dirty: Pointer<2>,
}

impl GameManagerPointers {
    fn new() -> GameManagerPointers {
        GameManagerPointers {
            scene_name: Pointer::new("GameManager", 0, &["_instance", "sceneName"]),
            next_scene_name: Pointer::new("GameManager", 0, &["_instance", "nextSceneName"]),
            game_state: Pointer::new("GameManager", 0, &["_instance", "gameState"]),
            ui_state_vanilla: Pointer::new("GameManager", 0, &["_instance", "<ui>k__BackingField", "uiState"]),
            ui_state_modded: Pointer::new("GameManager", 0, &["_instance", "_uiInstance", "uiState"]),
            camera_teleporting: Pointer::new("GameManager", 0, &["_instance", "<cameraCtrl>k__BackingField", "teleporting"]),
            hazard_respawning: Pointer::new("GameManager", 0, &["_instance", "<hero_ctrl>k__BackingField", "cState", "hazardRespawning"]),
            accepting_input: Pointer::new("GameManager", 0, &["_instance", "<inputHandler>k__BackingField", "acceptingInput"]),
            hero_transition_state: Pointer::new("GameManager", 0, &["_instance", "<hero_ctrl>k__BackingField", "transitionState"]),
            tile_map_dirty: Pointer::new("GameManager", 0, &["_instance", "tilemapDirty"]),
        }
    }
}

// --------------------------------------------------------

struct PlayerDataPointers {
    fireball_level: Pointer<3>,
    has_dash: Pointer<3>,
    has_shadow_dash: Pointer<3>,
    has_wall_jump: Pointer<3>,
    has_double_jump: Pointer<3>,
    has_super_dash: Pointer<3>,
    has_acid_armor: Pointer<3>,
    has_dream_nail: Pointer<3>,
    has_dream_gate: Pointer<3>,
    dream_nail_upgraded: Pointer<3>,
    // Base number of masks, without any charms, bindings, lifeblood, or damage taken
    max_health_base: Pointer<3>,
    // Heart pieces represents one of:
    //  - number of heart pieces including the ones assembled into masks: 0-3 4-7 8-11 12-15 16
    //  - number of heart pieces excluding the ones assembled into masks: 0-3 0-3 0-3  0-3   0
    //  - number of heart pieces excluding masks except the final mask:   0-3 0-3 0-3  0-3   4
    // and I'm not sure which one
    heart_pieces: Pointer<3>,
    has_lantern: Pointer<3>,
    simple_keys: Pointer<3>,
    has_sly_key: Pointer<3>,
    has_white_key: Pointer<3>,
    #[cfg(debug_assertions)]
    geo: Pointer<3>,
    // Dashmaster
    got_charm_31: Pointer<3>,
    grubs_collected: Pointer<3>,
    // Gruz Mother
    killed_big_fly: Pointer<3>,
    sly_rescued: Pointer<3>,
    killed_gorgeous_husk: Pointer<3>,
    // Lemm
    met_relic_dealer_shop: Pointer<3>,
    watcher_chandelier: Pointer<3>,
    killed_black_knight: Pointer<3>,
    killed_mega_jellyfish: Pointer<3>,
    spider_capture: Pointer<3>,
    unchained_hollow_knight: Pointer<3>,
}

impl PlayerDataPointers {
    fn new() -> PlayerDataPointers {
        PlayerDataPointers {
            fireball_level: Pointer::new("GameManager", 0, &["_instance", "playerData", "fireballLevel"]),
            has_dash: Pointer::new("GameManager", 0, &["_instance", "playerData", "hasDash"]),
            has_shadow_dash: Pointer::new("GameManager", 0, &["_instance", "playerData", "hasShadowDash"]),
            has_wall_jump: Pointer::new("GameManager", 0, &["_instance", "playerData", "hasWalljump"]),
            has_double_jump: Pointer::new("GameManager", 0, &["_instance", "playerData", "hasDoubleJump"]),
            has_super_dash: Pointer::new("GameManager", 0, &["_instance", "playerData", "hasSuperDash"]),
            has_acid_armor: Pointer::new("GameManager", 0, &["_instance", "playerData", "hasAcidArmour"]),
            has_dream_nail: Pointer::new("GameManager", 0, &["_instance", "playerData", "hasDreamNail"]),
            has_dream_gate: Pointer::new("GameManager", 0, &["_instance", "playerData", "hasDreamGate"]),
            dream_nail_upgraded: Pointer::new("GameManager", 0, &["_instance", "playerData", "dreamNailUpgraded"]),
            max_health_base: Pointer::new("GameManager", 0, &["_instance", "playerData", "maxHealthBase"]),
            heart_pieces: Pointer::new("GameManager", 0, &["_instance", "playerData", "heartPieces"]),
            has_lantern: Pointer::new("GameManager", 0, &["_instance", "playerData", "hasLantern"]),
            simple_keys: Pointer::new("GameManager", 0, &["_instance", "playerData", "simpleKeys"]),
            has_sly_key: Pointer::new("GameManager", 0, &["_instance", "playerData", "hasSlykey"]),
            has_white_key: Pointer::new("GameManager", 0, &["_instance", "playerData", "hasWhiteKey"]),
            #[cfg(debug_assertions)]
            geo: Pointer::new("GameManager", 0, &["_instance", "playerData", "geo"]),
            got_charm_31: Pointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_31"]),
            grubs_collected: Pointer::new("GameManager", 0, &["_instance", "playerData", "grubsCollected"]),
            killed_big_fly: Pointer::new("GameManager", 0, &["_instance", "playerData", "killedBigFly"]),
            sly_rescued: Pointer::new("GameManager", 0, &["_instance", "playerData", "slyRescued"]),
            killed_gorgeous_husk: Pointer::new("GameManager", 0, &["_instance", "playerData", "killedGorgeousHusk"]),
            met_relic_dealer_shop: Pointer::new("GameManager", 0, &["_instance", "playerData", "metRelicDealerShop"]),
            watcher_chandelier: Pointer::new("GameManager", 0, &["_instance", "playerData", "watcherChandelier"]),
            killed_black_knight: Pointer::new("GameManager", 0, &["_instance", "playerData", "killedBlackKnight"]),
            killed_mega_jellyfish: Pointer::new("GameManager", 0, &["_instance", "playerData", "killedMegaJellyfish"]),
            spider_capture: Pointer::new("GameManager", 0, &["_instance", "playerData", "spiderCapture"]),
            unchained_hollow_knight: Pointer::new("GameManager", 0, &["_instance", "playerData", "unchainedHollowKnight"]),
        }
    }
}

// --------------------------------------------------------
// --------------------------------------------------------

pub struct GameManagerFinder {
    module: mono::Module,
    image: mono::Image,
    pointers: GameManagerPointers,
    player_data_pointers: PlayerDataPointers,
    ui_state_offset: OnceCell<u32>,
}

impl GameManagerFinder {
    pub async fn wait_attach(process: &Process) -> GameManagerFinder {
        let module = mono::Module::wait_attach_auto_detect(process).await;
        let image = module.wait_get_default_image(process).await;
        let pointers = GameManagerPointers::new();
        let player_data_pointers = PlayerDataPointers::new();
        let ui_state_offset = OnceCell::new();
        GameManagerFinder { module, image, pointers, player_data_pointers, ui_state_offset }
    }

    pub fn get_scene_name(&self, process: &Process) -> Option<String> {
        let s = self.pointers.scene_name.read(process, &self.module, &self.image).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, s)
    }

    pub fn get_next_scene_name(&self, process: &Process) -> Option<String> {
        let s = self.pointers.next_scene_name.read(process, &self.module, &self.image).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, s)
    }

    pub fn get_game_state(&self, process: &Process) -> Option<i32> {
        self.pointers.game_state.read(process, &self.module, &self.image).ok()
    }

    fn is_game_state_playing(&self, process: &Process) -> bool {
        self.get_game_state(process) == Some(GAME_STATE_PLAYING)
    }

    pub fn get_ui_state(&self, process: &Process) -> Option<i32> {
        // save the uiState offset so it doesn't have to find it in the UIManager class every time
        let ui_state_offset = if let Some(ui_state_offset) = self.ui_state_offset.get() {
            ui_state_offset
        } else {
            let ui_manager_class = self.image.get_class(process, &self.module, "UIManager")?;
            let ui_state_offset = ui_manager_class.get_field(process, &self.module, "uiState")?;
            self.ui_state_offset.get_or_init(|| ui_state_offset)
        };
        let ui = if let Ok(ui) = self.pointers.ui_state_vanilla.read(process, &self.module, &self.image) {
            ui
        } else if let Ok(ui) =  self.pointers.ui_state_modded.read(process, &self.module, &self.image) {
            ui
        } else {
            return None;
        };
        if ui_state_offset != &0x124 && ui >= 2 {
            Some(ui + 2)
        } else {
            Some(ui)
        }
    }

    pub fn camera_teleporting(&self, process: &Process) -> Option<bool> {
        self.pointers.camera_teleporting.read(process, &self.module, &self.image).ok()
    }

    pub fn hazard_respawning(&self, process: &Process) -> Option<bool> {
        self.pointers.hazard_respawning.read(process, &self.module, &self.image).ok()
    }

    pub fn accepting_input(&self, process: &Process) -> Option<bool> {
        self.pointers.accepting_input.read(process, &self.module, &self.image).ok()
    }

    pub fn hero_transition_state(&self, process: &Process) -> Option<i32> {
        self.pointers.hero_transition_state.read(process, &self.module, &self.image).ok()
    }

    pub fn tile_map_dirty(&self, process: &Process) -> Option<bool> {
        self.pointers.tile_map_dirty.read(process, &self.module, &self.image).ok()
    }

    pub fn uses_scene_transition_routine(&self) -> Option<bool> {
        /*
         * 1.3.1.5 and above swap from using LoadSceneAdditive to a SceneTransitionRoutine triggered
         * by BeginSceneTransitionRoutine, which doesn't set tilemapDirty back to false when you enter dnail
         * However, the early control glitch can only be performed on early patches so we can avoid this check entirely
         */
        // On current patch, return true
        // TODO: on other patches, something something lastVersion?.Minor >= 3
        Some(true)
    }

    pub fn get_fireball_level(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.fireball_level.read(process, &self.module, &self.image).ok()
    }

    pub fn has_dash(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_dash.read(process, &self.module, &self.image).ok()
    }

    pub fn has_shadow_dash(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_shadow_dash.read(process, &self.module, &self.image).ok()
    }

    pub fn has_wall_jump(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_wall_jump.read(process, &self.module, &self.image).ok()
    }

    pub fn has_double_jump(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_double_jump.read(process, &self.module, &self.image).ok()
    }

    pub fn has_super_dash(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_super_dash.read(process, &self.module, &self.image).ok()
    }

    pub fn has_acid_armour(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_acid_armor.read(process, &self.module, &self.image).ok()
    }

    pub fn has_dream_nail(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_dream_nail.read(process, &self.module, &self.image).ok()
    }

    pub fn has_dream_gate(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_dream_gate.read(process, &self.module, &self.image).ok()
    }
    
    pub fn dream_nail_upgraded(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.dream_nail_upgraded.read(process, &self.module, &self.image).ok()
    }

    pub fn max_health_base(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.max_health_base.read(process, &self.module, &self.image).ok()
    }

    pub fn heart_pieces(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.heart_pieces.read(process, &self.module, &self.image).ok()
    }

    pub fn has_lantern(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_lantern.read(process, &self.module, &self.image).ok()
    }

    pub fn get_simple_keys(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.simple_keys.read(process, &self.module, &self.image).ok()
    }

    pub fn has_sly_key(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_sly_key.read(process, &self.module, &self.image).ok()
    }

    pub fn has_white_key(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_white_key.read(process, &self.module, &self.image).ok()
    }

    #[cfg(debug_assertions)]
    pub fn get_geo(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.geo.read(process, &self.module, &self.image).ok()
    }

    // Dashmaster
    pub fn got_charm_31(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_31.read(process, &self.module, &self.image).ok()
    }

    pub fn grubs_collected(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.grubs_collected.read(process, &self.module, &self.image).ok()
    }

    // Gruz Mother
    pub fn killed_big_fly(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_big_fly.read(process, &self.module, &self.image).ok()
    }

    pub fn sly_rescued(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.sly_rescued.read(process, &self.module, &self.image).ok()
    }

    pub fn killed_gorgeous_husk(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_gorgeous_husk.read(process, &self.module, &self.image).ok()
    }

    // Lemm
    pub fn met_relic_dealer_shop(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.met_relic_dealer_shop.read(process, &self.module, &self.image).ok()
    }

    pub fn watcher_chandelier(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.watcher_chandelier.read(process, &self.module, &self.image).ok()
    }

    pub fn killed_black_knight(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_black_knight.read(process, &self.module, &self.image).ok()
    }

    pub fn killed_mega_jellyfish(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_mega_jellyfish.read(process, &self.module, &self.image).ok()
    }

    pub fn spider_capture(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.spider_capture.read(process, &self.module, &self.image).ok()
    }

    pub fn unchained_hollow_knight(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.unchained_hollow_knight.read(process, &self.module, &self.image).ok()
    }
}

pub struct SceneStore {
    prev_scene_name: String,
    curr_scene_name: String,
    next_scene_name: String,
    new_data_curr: bool,
    new_data_next: bool
}

impl SceneStore {
    pub fn new() -> SceneStore {
        SceneStore {
            prev_scene_name: "".to_string(),
            curr_scene_name: "".to_string(),
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
    pub fn new_curr_scene_name1(&mut self, msn: Option<String>) -> bool {
        match msn {
            None => false,
            Some(bad) if BAD_SCENE_NAMES.contains(&bad.as_str()) => {
                true
            }
            Some(sn) => {
                self.new_curr_scene_name(Some(sn));
                false
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
    map_i32: BTreeMap<&'static str, i32>,
    map_bool: BTreeMap<&'static str, bool>,
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
                self.map_i32.insert("fireball_level", l);
                l
            }
            _ => {
                *self.map_i32.get("fireball_level").unwrap_or(&0)
            }
        }
    }

    pub fn has_dash(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        match game_manager_finder.has_dash(process) {
            Some(k) if k || game_manager_finder.is_game_state_playing(process) => {
                self.map_bool.insert("has_dash", k);
                k
            }
            _ => {
                *self.map_bool.get("has_dash").unwrap_or(&false)
            }
        }
    }

    pub fn has_wall_jump(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        match game_manager_finder.has_wall_jump(process) {
            Some(w) if w || game_manager_finder.is_game_state_playing(process) => {
                self.map_bool.insert("has_wall_jump", w);
                w
            }
            _ => {
                *self.map_bool.get("has_wall_jump").unwrap_or(&false)
            }
        }
    }

    pub fn has_double_jump(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        match game_manager_finder.has_double_jump(process) {
            Some(d) if d || game_manager_finder.is_game_state_playing(process) => {
                self.map_bool.insert("has_double_jump", d);
                d
            }
            _ => {
                *self.map_bool.get("has_double_jump").unwrap_or(&false)
            }
        }
    }

    pub fn has_acid_armour(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        match game_manager_finder.has_acid_armour(process) {
            Some(a) if a || game_manager_finder.is_game_state_playing(process) => {
                self.map_bool.insert("has_acid_armor", a);
                a
            }
            _ => {
                *self.map_bool.get("has_acid_armor").unwrap_or(&false)
            }
        }
    }

    pub fn incremented_simple_keys(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        let store_simple_keys = self.map_i32.get("simple_keys").cloned();
        let player_data_simple_keys = game_manager_finder.get_simple_keys(process);
        if let Some(simple_keys) = player_data_simple_keys {
            if simple_keys != 0 || game_manager_finder.is_game_state_playing(process) {
                self.map_i32.insert("simple_keys", simple_keys);
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
                self.map_bool.insert("killed_gorgeous_husk", k);
                k
            }
            _ => {
                *self.map_bool.get("killed_gorgeous_husk").unwrap_or(&false)
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
