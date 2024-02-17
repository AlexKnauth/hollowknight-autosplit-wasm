
use core::cell::OnceCell;
use std::cmp::min;
use std::mem;
use std::collections::BTreeMap;
use asr::file_format::{elf, pe};
use asr::future::{next_tick, retry};
use asr::watcher::Pair;
use asr::{Address, PointerSize, Process};
use asr::game_engine::unity::mono::{self, UnityPointer};
use asr::string::ArrayWString;
use ugly_widget::store::StoreGui;

#[cfg(debug_assertions)]
use std::string::String;

use crate::file;

// --------------------------------------------------------

static HOLLOW_KNIGHT_NAMES: [&str; 4] = [
    "hollow_knight.exe", // Windows
    "hollow_knight.x86_64", // Linux
    "Hollow Knight", // Mac
    "hollow_knight", // Mac
];

pub const SCENE_PATH_SIZE: usize = 64;

struct StringListOffsets {
    pointer_size: PointerSize,
    string_len: u64,
    string_contents: u64,
    list_array: u64,
    array_len: u64,
    array_contents: u64,
}

impl StringListOffsets {
    fn new(pointer_size: PointerSize) -> StringListOffsets {
        match pointer_size {
            PointerSize::Bit64 => StringListOffsets {
                pointer_size,
                string_len: 0x10,
                string_contents: 0x14,
                list_array: 0x10,
                array_len: 0x18,
                array_contents: 0x20,
            },
            PointerSize::Bit32 => StringListOffsets {
                pointer_size,
                string_len: 0x8,
                string_contents: 0xc,
                list_array: 0x8,
                array_len: 0xc,
                array_contents: 0x10,
            },
            PointerSize::Bit16 => panic!("16-bit is not supported"),
        }
    }
}

const PRE_MENU_INTRO: &str = "Pre_Menu_Intro";
pub const MENU_TITLE: &str = "Menu_Title";
pub const QUIT_TO_MENU: &str = "Quit_To_Menu";
pub const PERMA_DEATH: &str = "PermaDeath";
pub const INTRO_CUTSCENE: &str = "Intro_Cutscene";
pub const OPENING_SEQUENCE: &str = "Opening_Sequence";
pub const GG_ENTRANCE_CUTSCENE: &str = "GG_Entrance_Cutscene";
pub static OPENING_SCENES: [&str; 2] = [
    INTRO_CUTSCENE,
    OPENING_SEQUENCE,
];

static NON_PLAY_SCENES: [&str; 16] = [
    PRE_MENU_INTRO,
    MENU_TITLE,
    QUIT_TO_MENU,
    INTRO_CUTSCENE,
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
    PERMA_DEATH,
    "PermaDeath_Unlock",
];

static DEBUG_SAVE_STATE_SCENE_NAMES: [&str; 2] = [
    "Room_Mender_House",
    "Room_Sly_Storeroom",
];

static BAD_SCENE_NAMES: [&str; 11] = [
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

pub static FUNGAL_WASTES_ENTRY_SCENES: &[&str] = &[
    "Fungus2_06", // Room outside Leg Eater
    "Fungus2_03", // From Queens' Station
    "Fungus2_23", // Bretta from Waterways
    "Fungus2_20", // Spore Shroom room, from QG (this one's unlikely to come up)
];

pub static CRYSTAL_PEAK_ENTRY_SCENES: &[&str] = &[
    "Mines_02",
    "Mines_10",
];

pub static WATERWAYS_ENTRY_SCENES: &[&str] = &[
    "Waterways_01", // Simple Key manhole entrance
    // Note: Waterways_06 does not show the Area Text
    "Waterways_07", // Where the Spike-tunnel and KE-Tram-CDash entrances meet
];

pub static FOG_CANYON_ENTRY_SCENES: &[&str] = &[
    "Fungus3_01", // West Fog Canyon entrance from Greenpath
    "Fungus3_02", // West Fog Canyon entrance from Queen's Station or QGA
    "Fungus3_24", // West Fog Canyon entrance from Queen's Gardens via Overgrown Mound
    "Fungus3_26", // East Fog Canyon, where the Crossroads acid and Leg Eater acid entrances meet
];

pub static QUEENS_GARDENS_ENTRY_SCENES: &[&str] = &[
    "Fungus3_34",
    "Deepnest_43",
];

pub static DEEPNEST_ENTRY_SCENES: &[&str] = &[
    "Fungus2_25", // Room after Mantis Lords
    "Deepnest_42", // Room outside Mask Maker
    "Abyss_03b", // Deepnest Tram
    "Deepnest_01b", // Near Spore Shroom
];

pub static GODHOME_LORE_SCENES: &[&str] = &[
    "GG_Engine", // includes GG_Engine_Prime when using starts_with_any
    "GG_Unn",
    "GG_Wyrm",
];

// --------------------------------------------------------

const VERSION_VEC_MAJOR: usize = 0;
const VERSION_VEC_MINOR: usize = 1;
// const VERSION_VEC_BUILD: usize = 2;
// const VERSION_VEC_REVISION: usize = 3;

pub const GAME_STATE_INACTIVE: i32 = 0;
pub const GAME_STATE_MAIN_MENU: i32 = 1;
pub const GAME_STATE_LOADING: i32 = 2;
pub const GAME_STATE_ENTERING_LEVEL: i32 = 3;
pub const GAME_STATE_PLAYING: i32 = 4;
// pub const GAME_STATE_PAUSED: i32 = 5;
pub const GAME_STATE_EXITING_LEVEL: i32 = 6;
pub const GAME_STATE_CUTSCENE: i32 = 7;

pub static NON_MENU_GAME_STATES: [i32; 2] = [
    GAME_STATE_PLAYING,
    GAME_STATE_CUTSCENE,
];
pub static NON_CONTINUOUS_GAME_STATES: [i32; 4] = [
    GAME_STATE_MAIN_MENU,
    GAME_STATE_LOADING,
    GAME_STATE_ENTERING_LEVEL,
    GAME_STATE_EXITING_LEVEL,
];

pub const UI_STATE_PLAYING: i32 = 6;
pub const UI_STATE_PAUSED: i32 = 7;

pub const HERO_TRANSITION_STATE_WAITING_TO_ENTER_LEVEL: i32 = 2;

struct GameManagerPointers {
    version_number: UnityPointer<4>,
    scene_name: UnityPointer<2>,
    next_scene_name: UnityPointer<2>,
    game_state: UnityPointer<2>,
    ui_state_vanilla: UnityPointer<3>,
    ui_state_modded: UnityPointer<3>,
    camera_teleporting: UnityPointer<3>,
    accepting_input: UnityPointer<3>,
    tile_map_dirty: UnityPointer<2>,
    hero_dead: UnityPointer<4>,
    hazard_death: UnityPointer<4>,
    hazard_respawning: UnityPointer<4>,
    hero_recoiling: UnityPointer<4>,
    hero_recoil_frozen: UnityPointer<4>,
    hero_transition_state: UnityPointer<3>,
    focusing: UnityPointer<4>,
}

impl GameManagerPointers {
    fn new() -> GameManagerPointers {
        GameManagerPointers {
            version_number: UnityPointer::new("GameManager", 0, &["_instance", "<inputHandler>k__BackingField", "debugInfo", "versionNumber"]),
            scene_name: UnityPointer::new("GameManager", 0, &["_instance", "sceneName"]),
            next_scene_name: UnityPointer::new("GameManager", 0, &["_instance", "nextSceneName"]),
            game_state: UnityPointer::new("GameManager", 0, &["_instance", "gameState"]),
            ui_state_vanilla: UnityPointer::new("GameManager", 0, &["_instance", "<ui>k__BackingField", "uiState"]),
            ui_state_modded: UnityPointer::new("GameManager", 0, &["_instance", "_uiInstance", "uiState"]),
            camera_teleporting: UnityPointer::new("GameManager", 0, &["_instance", "<cameraCtrl>k__BackingField", "teleporting"]),
            accepting_input: UnityPointer::new("GameManager", 0, &["_instance", "<inputHandler>k__BackingField", "acceptingInput"]),
            tile_map_dirty: UnityPointer::new("GameManager", 0, &["_instance", "tilemapDirty"]),
            hero_dead: UnityPointer::new("GameManager", 0, &["_instance", "<hero_ctrl>k__BackingField", "cState", "dead"]),
            hazard_death: UnityPointer::new("GameManager", 0, &["_instance", "<hero_ctrl>k__BackingField", "cState", "hazardDeath"]),
            hazard_respawning: UnityPointer::new("GameManager", 0, &["_instance", "<hero_ctrl>k__BackingField", "cState", "hazardRespawning"]),
            hero_recoiling: UnityPointer::new("GameManager", 0, &["_instance", "<hero_ctrl>k__BackingField", "cState", "recoiling"]),
            hero_recoil_frozen: UnityPointer::new("GameManager", 0, &["_instance", "<hero_ctrl>k__BackingField", "cState", "recoilFrozen"]),
            hero_transition_state: UnityPointer::new("GameManager", 0, &["_instance", "<hero_ctrl>k__BackingField", "transitionState"]),
            focusing: UnityPointer::new("GameManager", 0, &["_instance", "<hero_ctrl>k__BackingField", "cState", "focusing"]),
        }
    }
}

// --------------------------------------------------------

struct PlayerDataPointers {
    version: UnityPointer<4>,
    disable_pause: UnityPointer<3>,
    health: UnityPointer<3>,
    max_health: UnityPointer<3>,
    mpcharge: UnityPointer<3>,
    fireball_level: UnityPointer<3>,
    quake_level: UnityPointer<3>,
    scream_level: UnityPointer<3>,
    has_dash: UnityPointer<3>,
    has_shadow_dash: UnityPointer<3>,
    has_wall_jump: UnityPointer<3>,
    has_double_jump: UnityPointer<3>,
    has_super_dash: UnityPointer<3>,
    has_acid_armour: UnityPointer<3>,
    /// hasCyclone: actually means Cyclone Slash, from Mato
    has_cyclone: UnityPointer<3>,
    /// hasDashSlash: secretly means Great Slash, from Sheo
    has_dash_slash: UnityPointer<3>,
    /// hasUpwardSlash: secretly means Dash Slash, from Oro
    has_upward_slash: UnityPointer<3>,
    has_dream_nail: UnityPointer<3>,
    has_dream_gate: UnityPointer<3>,
    dream_nail_upgraded: UnityPointer<3>,
    // Base number of masks, without any charms, bindings, lifeblood, or damage taken
    max_health_base: UnityPointer<3>,
    // Heart pieces represents one of:
    //  - number of heart pieces including the ones assembled into masks: 0-3 4-7 8-11 12-15 16
    //  - number of heart pieces excluding the ones assembled into masks: 0-3 0-3 0-3  0-3   0
    //  - number of heart pieces excluding masks except the final mask:   0-3 0-3 0-3  0-3   4
    // and I'm not sure which one
    heart_pieces: UnityPointer<3>,
    soul_limited: UnityPointer<3>,
    /// Magic Power Reserve Max: amount of soul that can be held by soul vessels, 33 each
    mp_reserve_max: UnityPointer<3>,
    vessel_fragments: UnityPointer<3>,
    at_bench: UnityPointer<3>,
    // Dreamers
    mask_broken_lurien: UnityPointer<3>,
    mask_broken_monomon: UnityPointer<3>,
    mask_broken_hegemol: UnityPointer<3>,
    guardians_defeated: UnityPointer<3>,
    // Old Dreamer Timings, mark deprecated or whatever
    lurien_defeated: UnityPointer<3>,
    monomon_defeated: UnityPointer<3>,
    hegemol_defeated: UnityPointer<3>,
    mr_mushroom_state: UnityPointer<3>,
    // Keys
    has_city_key: UnityPointer<3>,
    has_lantern: UnityPointer<3>,
    simple_keys: UnityPointer<3>,
    has_sly_key: UnityPointer<3>,
    has_white_key: UnityPointer<3>,
    has_love_key: UnityPointer<3>,
    got_lurker_key: UnityPointer<3>,
    sly_simple_key: UnityPointer<3>,
    has_kings_brand: UnityPointer<3>,
    has_tram_pass: UnityPointer<3>,
    cornifer_at_home: UnityPointer<3>,
    #[cfg(debug_assertions)]
    geo: UnityPointer<3>,
    // Nail and Pale Ore
    nail_smith_upgrades: UnityPointer<3>,
    ore: UnityPointer<3>,
    // Stags
    stag_position: UnityPointer<3>,
    opened_crossroads: UnityPointer<3>,
    opened_greenpath: UnityPointer<3>,
    opened_fungal_wastes: UnityPointer<3>,
    opened_ruins1: UnityPointer<3>,
    opened_ruins2: UnityPointer<3>,
    opened_resting_grounds: UnityPointer<3>,
    opened_hidden_station: UnityPointer<3>,
    opened_deepnest: UnityPointer<3>,
    opened_royal_gardens: UnityPointer<3>,
    opened_stag_nest: UnityPointer<3>,
    travelling: UnityPointer<3>,
    // Relics
    trinket1: UnityPointer<3>,
    trinket2: UnityPointer<3>,
    trinket3: UnityPointer<3>,
    trinket4: UnityPointer<3>,
    sold_trinket1: UnityPointer<3>,
    sold_trinket2: UnityPointer<3>,
    sold_trinket3: UnityPointer<3>,
    sold_trinket4: UnityPointer<3>,
    rancid_eggs: UnityPointer<3>,
    jinn_eggs_sold: UnityPointer<3>,
    ghost_coins: UnityPointer<3>,
    // Charm Notches
    notch_shroom_ogres: UnityPointer<3>,
    salubra_notch1: UnityPointer<3>,
    salubra_notch2: UnityPointer<3>,
    salubra_notch3: UnityPointer<3>,
    salubra_notch4: UnityPointer<3>,
    notch_fog_canyon: UnityPointer<3>,
    got_grimm_notch: UnityPointer<3>,
    charm_slots: UnityPointer<3>,
    can_overcharm: UnityPointer<3>,
    // Charms
    got_charm_1: UnityPointer<3>,
    got_charm_2: UnityPointer<3>,
    got_charm_3: UnityPointer<3>,
    got_charm_4: UnityPointer<3>,
    got_charm_5: UnityPointer<3>,
    equipped_charm_5: UnityPointer<3>,
    got_charm_6: UnityPointer<3>,
    got_charm_7: UnityPointer<3>,
    equipped_charm_7: UnityPointer<3>,
    got_charm_8: UnityPointer<3>,
    got_charm_9: UnityPointer<3>,
    got_charm_10: UnityPointer<3>,
    got_charm_11: UnityPointer<3>,
    got_charm_12: UnityPointer<3>,
    got_charm_13: UnityPointer<3>,
    got_charm_14: UnityPointer<3>,
    got_charm_15: UnityPointer<3>,
    got_charm_16: UnityPointer<3>,
    got_charm_17: UnityPointer<3>,
    equipped_charm_17: UnityPointer<3>,
    got_charm_18: UnityPointer<3>,
    got_charm_19: UnityPointer<3>,
    got_charm_20: UnityPointer<3>,
    got_charm_21: UnityPointer<3>,
    got_charm_22: UnityPointer<3>,
    got_charm_26: UnityPointer<3>,
    got_charm_27: UnityPointer<3>,
    got_charm_28: UnityPointer<3>,
    equipped_charm_28: UnityPointer<3>,
    got_charm_29: UnityPointer<3>,
    got_charm_30: UnityPointer<3>,
    // Dashmaster
    got_charm_31: UnityPointer<3>,
    got_charm_32: UnityPointer<3>,
    got_charm_33: UnityPointer<3>,
    got_charm_34: UnityPointer<3>,
    got_charm_35: UnityPointer<3>,
    got_charm_37: UnityPointer<3>,
    got_charm_38: UnityPointer<3>,
    got_charm_39: UnityPointer<3>,
    // Fragile / Unbreakable Charms
    got_charm_23: UnityPointer<3>,
    got_charm_24: UnityPointer<3>,
    got_charm_25: UnityPointer<3>,
    broken_charm_23: UnityPointer<3>,
    broken_charm_24: UnityPointer<3>,
    broken_charm_25: UnityPointer<3>,
    fragile_greed_unbreakable: UnityPointer<3>,
    fragile_health_unbreakable: UnityPointer<3>,
    fragile_strength_unbreakable: UnityPointer<3>,
    // Grimmchild / Carefree Melody
    got_charm_40: UnityPointer<3>,
    equipped_charm_40: UnityPointer<3>,
    grimm_child_level: UnityPointer<3>,
    flames_collected: UnityPointer<3>,
    got_brumms_flame: UnityPointer<3>,
    // Kingsoul / VoidHeart
    charm_cost_36: UnityPointer<3>,
    got_queen_fragment: UnityPointer<3>,
    got_king_fragment: UnityPointer<3>,
    royal_charm_state: UnityPointer<3>,
    got_shade_charm: UnityPointer<3>,
    grubs_collected: UnityPointer<3>,
    scenes_grub_rescued: UnityPointer<3>,
    kills_grub_mimic: UnityPointer<3>,
    dream_orbs: UnityPointer<3>,
    scenes_encountered_dream_plant_c: UnityPointer<3>,
    dream_gate_scene: UnityPointer<3>,
    dream_gate_x: UnityPointer<3>,
    dream_gate_y: UnityPointer<3>,
    map_dirtmouth: UnityPointer<3>,
    map_crossroads: UnityPointer<3>,
    map_greenpath: UnityPointer<3>,
    map_fog_canyon: UnityPointer<3>,
    map_royal_gardens: UnityPointer<3>,
    map_fungal_wastes: UnityPointer<3>,
    map_city: UnityPointer<3>,
    map_waterways: UnityPointer<3>,
    map_mines: UnityPointer<3>,
    map_deepnest: UnityPointer<3>,
    map_cliffs: UnityPointer<3>,
    map_outskirts: UnityPointer<3>,
    map_resting_grounds: UnityPointer<3>,
    map_abyss: UnityPointer<3>,
    visited_dirtmouth: UnityPointer<3>,
    sly_shell_frag1: UnityPointer<3>,
    sly_shell_frag4: UnityPointer<3>,
    sly_vessel_frag1: UnityPointer<3>,
    sly_vessel_frag2: UnityPointer<3>,
    elderbug_gave_flower: UnityPointer<3>,
    killed_grimm: UnityPointer<3>,
    killed_nightmare_grimm: UnityPointer<3>,
    killed_grey_prince: UnityPointer<3>,
    grey_prince_orbs_collected: UnityPointer<3>,
    grey_prince_defeats: UnityPointer<3>,
    visited_crossroads: UnityPointer<3>,
    crossroads_infected: UnityPointer<3>,
    killed_mender_bug: UnityPointer<3>,
    killed_mawlek: UnityPointer<3>,
    // Gruz Mother
    killed_big_fly: UnityPointer<3>,
    sly_rescued: UnityPointer<3>,
    killed_false_knight: UnityPointer<3>,
    false_knight_dream_defeated: UnityPointer<3>,
    false_knight_orbs_collected: UnityPointer<3>,
    salubra_blessing: UnityPointer<3>,
    unchained_hollow_knight: UnityPointer<3>,
    killed_hollow_knight: UnityPointer<3>,
    killed_final_boss: UnityPointer<3>,
    visited_greenpath: UnityPointer<3>,
    killed_moss_knight: UnityPointer<3>,
    zote_rescued_buzzer: UnityPointer<3>,
    killed_hornet: UnityPointer<3>,
    /// killedLazyFlyer: Aluba
    killed_lazy_flyer: UnityPointer<3>,
    killed_hunter_mark: UnityPointer<3>,
    killed_ghost_no_eyes: UnityPointer<3>,
    no_eyes_defeated: UnityPointer<3>,
    mega_moss_charger_defeated: UnityPointer<3>,
    nailsmith_convo_art: UnityPointer<3>,
    visited_fungus: UnityPointer<3>,
    kills_mushroom_brawler: UnityPointer<3>,
    killed_ghost_hu: UnityPointer<3>,
    elder_hu_defeated: UnityPointer<3>,
    bretta_rescued: UnityPointer<3>,
    defeated_mantis_lords: UnityPointer<3>,
    // Gorb
    killed_ghost_aladar: UnityPointer<3>,
    aladar_slug_defeated: UnityPointer<3>,
    nightmare_lantern_lit: UnityPointer<3>,
    destroyed_nightmare_lantern: UnityPointer<3>,
    visited_resting_grounds: UnityPointer<3>,
    killed_ghost_xero: UnityPointer<3>,
    xero_defeated: UnityPointer<3>,
    glade_door_opened: UnityPointer<3>,
    moth_departed: UnityPointer<3>,
    /// Met Grey Mourner
    met_xun: UnityPointer<3>,
    has_xun_flower: UnityPointer<3>,
    xun_reward_given: UnityPointer<3>,
    opened_city_gate: UnityPointer<3>,
    visited_ruins: UnityPointer<3>,
    killed_gorgeous_husk: UnityPointer<3>,
    // Lemm
    met_relic_dealer_shop: UnityPointer<3>,
    toll_bench_city: UnityPointer<3>,
    /// Killed Soul Twister
    killed_mage: UnityPointer<3>,
    /// Killed Soul Warrior
    killed_mage_knight: UnityPointer<3>,
    // Soul Master
    mage_lord_encountered: UnityPointer<3>,
    mage_lord_encountered_2: UnityPointer<3>,
    killed_mage_lord: UnityPointer<3>,
    mage_lord_dream_defeated: UnityPointer<3>,
    mage_lord_orbs_collected: UnityPointer<3>,
    kills_great_shield_zombie: UnityPointer<3>,
    watcher_chandelier: UnityPointer<3>,
    killed_black_knight: UnityPointer<3>,
    collector_defeated: UnityPointer<3>,
    nailsmith_killed: UnityPointer<3>,
    nailsmith_spared: UnityPointer<3>,
    visited_mines: UnityPointer<3>,
    kills_zombie_miner: UnityPointer<3>,
    // Crystal Guardian
    defeated_mega_beam_miner: UnityPointer<3>,
    kills_mega_beam_miner: UnityPointer<3>,
    mine_lift_opened: UnityPointer<3>,
    opened_waterways_manhole: UnityPointer<3>,
    visited_waterways: UnityPointer<3>,
    killed_dung_defender: UnityPointer<3>,
    killed_white_defender: UnityPointer<3>,
    white_defender_orbs_collected: UnityPointer<3>,
    white_defender_defeats: UnityPointer<3>,
    met_emilitia: UnityPointer<3>,
    given_emilitia_flower: UnityPointer<3>,
    killed_fluke_mother: UnityPointer<3>,
    /// Visited Ancient Basin
    visited_abyss: UnityPointer<3>,
    saved_cloth: UnityPointer<3>,
    toll_bench_abyss: UnityPointer<3>,
    // Broken Vessel
    killed_infected_knight: UnityPointer<3>,
    infected_knight_dream_defeated: UnityPointer<3>,
    infected_knight_orbs_collected: UnityPointer<3>,
    abyss_gate_opened: UnityPointer<3>,
    abyss_lighthouse: UnityPointer<3>,
    visited_white_palace: UnityPointer<3>,
    white_palace_orb_1: UnityPointer<3>,
    white_palace_orb_2: UnityPointer<3>,
    white_palace_orb_3: UnityPointer<3>,
    /// New data on hunter's journal entry Seal of Binding / Path of Pain
    new_data_binding_seal: UnityPointer<3>,
    white_palace_secret_room_visited: UnityPointer<3>,
    /// Visited Kingdom's Edge
    visited_outskirts: UnityPointer<3>,
    visited_hive: UnityPointer<3>,
    killed_hive_knight: UnityPointer<3>,
    killed_giant_hopper: UnityPointer<3>,
    given_oro_flower: UnityPointer<3>,
    hornet_outskirts_defeated: UnityPointer<3>,
    killed_ghost_markoth: UnityPointer<3>,
    markoth_defeated: UnityPointer<3>,
    little_fool_met: UnityPointer<3>,
    colosseum_bronze_opened: UnityPointer<3>,
    seen_colosseum_title: UnityPointer<3>,
    kills_col_shield: UnityPointer<3>,
    kills_col_roller: UnityPointer<3>,
    kills_col_miner: UnityPointer<3>,
    kills_spitter: UnityPointer<3>,
    kills_super_spitter: UnityPointer<3>,
    kills_buzzer: UnityPointer<3>,
    kills_big_buzzer: UnityPointer<3>,
    kills_bursting_bouncer: UnityPointer<3>,
    kills_big_fly: UnityPointer<3>,
    killed_zote: UnityPointer<3>,
    colosseum_bronze_completed: UnityPointer<3>,
    colosseum_silver_opened: UnityPointer<3>,
    kills_col_worm: UnityPointer<3>,
    kills_col_flying_sentry: UnityPointer<3>,
    kills_col_mosquito: UnityPointer<3>,
    kills_ceiling_dropper: UnityPointer<3>,
    kills_giant_hopper: UnityPointer<3>,
    kills_blobble: UnityPointer<3>,
    kills_oblobble: UnityPointer<3>,
    colosseum_silver_completed: UnityPointer<3>,
    colosseum_gold_opened: UnityPointer<3>,
    kills_angry_buzzer: UnityPointer<3>,
    kills_col_hopper: UnityPointer<3>,
    kills_heavy_mantis: UnityPointer<3>,
    kills_mantis_heavy_flyer: UnityPointer<3>,
    kills_mage_knight: UnityPointer<3>,
    kills_electric_mage: UnityPointer<3>,
    kills_mage: UnityPointer<3>,
    kills_lesser_mawlek: UnityPointer<3>,
    kills_mawlek: UnityPointer<3>,
    // God Tamer
    killed_lobster_lancer: UnityPointer<3>,
    kills_lobster_lancer: UnityPointer<3>,
    colosseum_gold_completed: UnityPointer<3>,
    visited_fog_canyon: UnityPointer<3>,
    // Uumuu
    encountered_mega_jelly: UnityPointer<3>,
    killed_mega_jellyfish: UnityPointer<3>,
    visited_royal_gardens: UnityPointer<3>,
    toll_bench_queens_gardens: UnityPointer<3>,
    xun_flower_given: UnityPointer<3>,
    killed_ghost_marmu: UnityPointer<3>,
    mum_caterpillar_defeated: UnityPointer<3>,
    killed_traitor_lord: UnityPointer<3>,
    given_white_lady_flower: UnityPointer<3>,
    visited_deepnest: UnityPointer<3>,
    visited_deepnest_spa: UnityPointer<3>,
    zote_rescued_deepnest: UnityPointer<3>,
    opened_tram_lower: UnityPointer<3>,
    // Nosk
    killed_mimic_spider: UnityPointer<3>,
    killed_ghost_galien: UnityPointer<3>,
    galien_defeated: UnityPointer<3>,
    spider_capture: UnityPointer<3>,
    has_godfinder: UnityPointer<3>,
    given_godseeker_flower: UnityPointer<3>,
    visited_godhome: UnityPointer<3>,
    zote_statue_wall_broken: UnityPointer<3>,
    ordeal_achieved: UnityPointer<3>,
    // Oro & Mato
    killed_nail_bros: UnityPointer<3>,
    killed_paintmaster: UnityPointer<3>,
    killed_nailsage: UnityPointer<3>,
    // Pure Vessel
    killed_hollow_knight_prime: UnityPointer<3>,
}

impl PlayerDataPointers {
    fn new() -> PlayerDataPointers {
        PlayerDataPointers {
            version: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "version"]),
            disable_pause: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "disablePause"]),
            health: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "health"]),
            max_health: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "maxHealth"]),
            mpcharge: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "MPCharge"]),
            fireball_level: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "fireballLevel"]),
            quake_level: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "quakeLevel"]),
            scream_level: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "screamLevel"]),
            has_dash: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasDash"]),
            has_shadow_dash: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasShadowDash"]),
            has_wall_jump: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasWalljump"]),
            has_double_jump: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasDoubleJump"]),
            has_super_dash: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasSuperDash"]),
            has_acid_armour: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasAcidArmour"]),
            has_cyclone: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasCyclone"]),
            has_dash_slash: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasDashSlash"]),
            has_upward_slash: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasUpwardSlash"]),
            has_dream_nail: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasDreamNail"]),
            has_dream_gate: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasDreamGate"]),
            dream_nail_upgraded: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "dreamNailUpgraded"]),
            max_health_base: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "maxHealthBase"]),
            heart_pieces: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "heartPieces"]),
            soul_limited: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "soulLimited"]),
            mp_reserve_max: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "MPReserveMax"]),
            vessel_fragments: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "vesselFragments"]),
            at_bench: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "atBench"]),
            // Dreamers
            mask_broken_lurien: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "maskBrokenLurien"]),
            mask_broken_monomon: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "maskBrokenMonomon"]),
            mask_broken_hegemol: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "maskBrokenHegemol"]),
            guardians_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "guardiansDefeated"]),
            lurien_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "lurienDefeated"]),
            monomon_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "monomonDefeated"]),
            hegemol_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hegemolDefeated"]),
            mr_mushroom_state: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mrMushroomState"]),
            // Keys
            has_city_key: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasCityKey"]),
            has_lantern: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasLantern"]),
            simple_keys: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "simpleKeys"]),
            has_sly_key: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasSlykey"]),
            has_white_key: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasWhiteKey"]),
            has_love_key: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasLoveKey"]),
            got_lurker_key: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotLurkerKey"]),
            sly_simple_key: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "slySimpleKey"]),
            has_kings_brand: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasKingsBrand"]),
            has_tram_pass: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasTramPass"]),
            cornifer_at_home: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "corniferAtHome"]),
            #[cfg(debug_assertions)]
            geo: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "geo"]),
            // Nail and Pale Ore
            nail_smith_upgrades: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "nailSmithUpgrades"]),
            ore: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "ore"]),
            // Stags
            stag_position: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "stagPosition"]),
            opened_crossroads: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "openedCrossroads"]),
            opened_greenpath: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "openedGreenpath"]),
            opened_fungal_wastes: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "openedFungalWastes"]),
            opened_ruins1: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "openedRuins1"]),
            opened_ruins2: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "openedRuins2"]),
            opened_resting_grounds: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "openedRestingGrounds"]),
            opened_hidden_station: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "openedHiddenStation"]),
            opened_deepnest: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "openedDeepnest"]),
            opened_royal_gardens: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "openedRoyalGardens"]),
            opened_stag_nest: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "openedStagNest"]),
            travelling: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "travelling"]),
            // Relics
            trinket1: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "trinket1"]),
            trinket2: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "trinket2"]),
            trinket3: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "trinket3"]),
            trinket4: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "trinket4"]),
            sold_trinket1: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "soldTrinket1"]),
            sold_trinket2: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "soldTrinket2"]),
            sold_trinket3: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "soldTrinket3"]),
            sold_trinket4: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "soldTrinket4"]),
            rancid_eggs: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "rancidEggs"]),
            jinn_eggs_sold: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "jinnEggsSold"]),
            ghost_coins: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "ghostCoins"]),
            // Charm Notches
            notch_shroom_ogres: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "notchShroomOgres"]),
            salubra_notch1: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "salubraNotch1"]),
            salubra_notch2: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "salubraNotch2"]),
            salubra_notch3: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "salubraNotch3"]),
            salubra_notch4: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "salubraNotch4"]),
            notch_fog_canyon: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "notchFogCanyon"]),
            got_grimm_notch: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotGrimmNotch"]),
            charm_slots: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "charmSlots"]),
            can_overcharm: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "canOvercharm"]),
            // Charms
            got_charm_1: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_1"]),
            got_charm_2: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_2"]),
            got_charm_3: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_3"]),
            got_charm_4: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_4"]),
            got_charm_5: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_5"]),
            equipped_charm_5: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "equippedCharm_5"]),
            got_charm_6: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_6"]),
            got_charm_7: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_7"]),
            equipped_charm_7: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "equippedCharm_7"]),
            got_charm_8: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_8"]),
            got_charm_9: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_9"]),
            got_charm_10: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_10"]),
            got_charm_11: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_11"]),
            got_charm_12: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_12"]),
            got_charm_13: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_13"]),
            got_charm_14: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_14"]),
            got_charm_15: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_15"]),
            got_charm_16: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_16"]),
            got_charm_17: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_17"]),
            equipped_charm_17: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "equippedCharm_17"]),
            got_charm_18: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_18"]),
            got_charm_19: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_19"]),
            got_charm_20: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_20"]),
            got_charm_21: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_21"]),
            got_charm_22: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_22"]),
            got_charm_26: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_26"]),
            got_charm_27: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_27"]),
            got_charm_28: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_28"]),
            equipped_charm_28: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "equippedCharm_28"]),
            got_charm_29: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_29"]),
            got_charm_30: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_30"]),
            got_charm_31: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_31"]),
            got_charm_32: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_32"]),
            got_charm_33: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_33"]),
            got_charm_34: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_34"]),
            got_charm_35: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_35"]),
            got_charm_37: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_37"]),
            got_charm_38: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_38"]),
            got_charm_39: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_39"]),
            // Fragile / Unbreakable Charms
            got_charm_23: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_23"]),
            got_charm_24: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_24"]),
            got_charm_25: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_25"]),
            broken_charm_23: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "brokenCharm_23"]),
            broken_charm_24: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "brokenCharm_24"]),
            broken_charm_25: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "brokenCharm_23"]),
            fragile_greed_unbreakable: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "fragileGreed_unbreakable"]),
            fragile_health_unbreakable: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "fragileHealth_unbreakable"]),
            fragile_strength_unbreakable: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "fragileStrength_unbreakable"]),
            // Grimmchild / Carefree Melody
            got_charm_40: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_40"]),
            equipped_charm_40: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "equippedCharm_40"]),
            grimm_child_level: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "grimmChildLevel"]),
            flames_collected: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "flamesCollected"]),
            got_brumms_flame: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotBrummsFlame"]),
            // Kingsoul / VoidHeart
            charm_cost_36: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "charmCost_36"]),
            got_queen_fragment: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotQueenFragment"]),
            got_king_fragment: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotKingFragment"]),
            royal_charm_state: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "royalCharmState"]),
            got_shade_charm: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotShadeCharm"]),
            grubs_collected: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "grubsCollected"]),
            scenes_grub_rescued: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "scenesGrubRescued"]),
            kills_grub_mimic: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsGrubMimic"]),
            dream_orbs: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "dreamOrbs"]),
            scenes_encountered_dream_plant_c: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "scenesEncounteredDreamPlantC"]),
            dream_gate_scene: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "dreamGateScene"]),
            dream_gate_x: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "dreamGateX"]),
            dream_gate_y: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "dreamGateY"]),
            map_dirtmouth: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mapDirtmouth"]),
            map_crossroads: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mapCrossroads"]),
            map_greenpath: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mapGreenpath"]),
            map_fog_canyon: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mapFogCanyon"]),
            map_royal_gardens: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mapRoyalGardens"]),
            map_fungal_wastes: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mapFungalWastes"]),
            map_city: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mapCity"]),
            map_waterways: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mapWaterways"]),
            map_mines: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mapMines"]),
            map_deepnest: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mapDeepnest"]),
            map_cliffs: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mapCliffs"]),
            map_outskirts: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mapOutskirts"]),
            map_resting_grounds: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mapRestingGrounds"]),
            map_abyss: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mapAbyss"]),
            visited_dirtmouth: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedDirtmouth"]),
            sly_shell_frag1: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "slyShellFrag1"]),
            sly_shell_frag4: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "slyShellFrag4"]),
            sly_vessel_frag1: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "slyVesselFrag1"]),
            sly_vessel_frag2: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "slyVesselFrag2"]),
            elderbug_gave_flower: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "elderbugGaveFlower"]),
            killed_grimm: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGrimm"]),
            killed_nightmare_grimm: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedNightmareGrimm"]),
            killed_grey_prince: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGreyPrince"]),
            grey_prince_orbs_collected: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "greyPrinceOrbsCollected"]),
            grey_prince_defeats: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "greyPrinceDefeats"]),
            visited_crossroads: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedCrossroads"]),
            crossroads_infected: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "crossroadsInfected"]),
            killed_mender_bug: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedMenderBug"]),
            killed_mawlek: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedMawlek"]),
            killed_big_fly: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedBigFly"]),
            sly_rescued: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "slyRescued"]),
            killed_false_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedFalseKnight"]),
            false_knight_dream_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "falseKnightDreamDefeated"]),
            false_knight_orbs_collected: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "falseKnightOrbsCollected"]),
            salubra_blessing: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "salubraBlessing"]),
            unchained_hollow_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "unchainedHollowKnight"]),
            killed_hollow_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedHollowKnight"]),
            killed_final_boss: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedFinalBoss"]),
            visited_greenpath: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedGreenpath"]),
            killed_moss_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedMossKnight"]),
            zote_rescued_buzzer: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "zoteRescuedBuzzer"]),
            killed_hornet: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedHornet"]),
            killed_lazy_flyer: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedLazyFlyer"]),
            killed_hunter_mark: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedHunterMark"]),
            killed_ghost_no_eyes: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGhostNoEyes"]),
            no_eyes_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "noEyesDefeated"]),
            mega_moss_charger_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "megaMossChargerDefeated"]),
            nailsmith_convo_art: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "nailsmithConvoArt"]),
            visited_fungus: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedFungus"]),
            kills_mushroom_brawler: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsMushroomBrawler"]),
            killed_ghost_hu: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGhostHu"]),
            elder_hu_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "elderHuDefeated"]),
            bretta_rescued: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "brettaRescued"]),
            defeated_mantis_lords: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedMantisLords"]),
            // Gorb
            killed_ghost_aladar: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGhostAladar"]),
            aladar_slug_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "aladarSlugDefeated"]),
            nightmare_lantern_lit: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "nightmareLanternLit"]),
            destroyed_nightmare_lantern: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "destroyedNightmareLantern"]),
            visited_resting_grounds: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedRestingGrounds"]),
            killed_ghost_xero: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGhostXero"]),
            xero_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "xeroDefeated"]),
            glade_door_opened: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gladeDoorOpened"]),
            moth_departed: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mothDeparted"]),
            met_xun: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "metXun"]),
            has_xun_flower: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasXunFlower"]),
            xun_reward_given: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "xunRewardGiven"]),
            opened_city_gate: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "openedCityGate"]),
            visited_ruins: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedRuins"]),
            killed_gorgeous_husk: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGorgeousHusk"]),
            met_relic_dealer_shop: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "metRelicDealerShop"]),
            toll_bench_city: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "tollBenchCity"]),
            killed_mage: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedMage"]),
            killed_mage_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedMageKnight"]),
            mage_lord_encountered: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mageLordEncountered"]),
            mage_lord_encountered_2: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mageLordEncountered_2"]),
            killed_mage_lord: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedMageLord"]),
            mage_lord_dream_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mageLordDreamDefeated"]),
            mage_lord_orbs_collected: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mageLordOrbsCollected"]),
            kills_great_shield_zombie: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsGreatShieldZombie"]),
            watcher_chandelier: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "watcherChandelier"]),
            killed_black_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedBlackKnight"]),
            collector_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "collectorDefeated"]),
            nailsmith_killed: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "nailsmithKilled"]),
            nailsmith_spared: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "nailsmithSpared"]),
            visited_mines: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedMines"]),
            kills_zombie_miner: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsZombieMiner"]),
            defeated_mega_beam_miner: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedMegaBeamMiner"]),
            kills_mega_beam_miner: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsMegaBeamMiner"]),
            mine_lift_opened: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mineLiftOpened"]),
            opened_waterways_manhole: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "openedWaterwaysManhole"]),
            visited_waterways: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedWaterways"]),
            killed_dung_defender: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedDungDefender"]),
            killed_white_defender: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedWhiteDefender"]),
            white_defender_orbs_collected: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "whiteDefenderOrbsCollected"]),
            white_defender_defeats: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "whiteDefenderDefeats"]),
            met_emilitia: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "metEmilitia"]),
            given_emilitia_flower: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "givenEmilitiaFlower"]),
            killed_fluke_mother: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedFlukeMother"]),
            visited_abyss: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedAbyss"]),
            saved_cloth: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "savedCloth"]),
            toll_bench_abyss: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "tollBenchAbyss"]),
            killed_infected_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedInfectedKnight"]),
            infected_knight_dream_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "infectedKnightDreamDefeated"]),
            infected_knight_orbs_collected: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "infectedKnightOrbsCollected"]),
            abyss_gate_opened: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "abyssGateOpened"]),
            abyss_lighthouse: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "abyssLighthouse"]),
            visited_white_palace: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedWhitePalace"]),
            white_palace_orb_1: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "whitePalaceOrb_1"]),
            white_palace_orb_2: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "whitePalaceOrb_2"]),
            white_palace_orb_3: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "whitePalaceOrb_3"]),
            new_data_binding_seal: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "newDataBindingSeal"]),
            white_palace_secret_room_visited: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "whitePalaceSecretRoomVisited"]),
            visited_outskirts: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedOutskirts"]),
            visited_hive: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedHive"]),
            killed_hive_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedHiveKnight"]),
            killed_giant_hopper: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGiantHopper"]),
            given_oro_flower: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "givenOroFlower"]),
            hornet_outskirts_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hornetOutskirtsDefeated"]),
            killed_ghost_markoth: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGhostMarkoth"]),
            markoth_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "markothDefeated"]),
            little_fool_met: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "littleFoolMet"]),
            colosseum_bronze_opened: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "colosseumBronzeOpened"]),
            seen_colosseum_title: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "seenColosseumTitle"]),
            kills_col_shield: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsColShield"]),
            kills_col_roller: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsColRoller"]),
            kills_col_miner: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsColMiner"]),
            kills_spitter: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsSpitter"]),
            kills_super_spitter: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsSuperSpitter"]),
            kills_buzzer: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsBuzzer"]),
            kills_big_buzzer: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsBigBuzzer"]),
            kills_bursting_bouncer: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsBurstingBouncer"]),
            kills_big_fly: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsBigFly"]),
            killed_zote: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedZote"]),
            colosseum_bronze_completed: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "colosseumBronzeCompleted"]),
            colosseum_silver_opened: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "colosseumSilverOpened"]),
            kills_col_worm: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsColWorm"]),
            kills_col_flying_sentry: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsColFlyingSentry"]),
            kills_col_mosquito: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsColMosquito"]),
            kills_ceiling_dropper: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsCeilingDropper"]),
            kills_giant_hopper: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsGiantHopper"]),
            kills_blobble: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsBlobble"]),
            kills_oblobble: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsOblobble"]),
            colosseum_silver_completed: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "colosseumSilverCompleted"]),
            colosseum_gold_opened: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "colosseumGoldOpened"]),
            kills_angry_buzzer: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsAngryBuzzer"]),
            kills_col_hopper: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsColHopper"]),
            kills_heavy_mantis: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsHeavyMantis"]),
            kills_mantis_heavy_flyer: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsMantisHeavyFlyer"]),
            kills_mage_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsMageKnight"]),
            kills_electric_mage: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsElectricMage"]),
            kills_mage: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsMage"]),
            kills_lesser_mawlek: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsLesserMawlek"]),
            kills_mawlek: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsMawlek"]),
            killed_lobster_lancer: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedLobsterLancer"]),
            kills_lobster_lancer: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsLobsterLancer"]),
            colosseum_gold_completed: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "colosseumGoldCompleted"]),
            visited_fog_canyon: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedFogCanyon"]),
            encountered_mega_jelly: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "encounteredMegaJelly"]),
            killed_mega_jellyfish: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedMegaJellyfish"]),
            visited_royal_gardens: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedRoyalGardens"]),
            toll_bench_queens_gardens: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "tollBenchQueensGardens"]),
            xun_flower_given: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "xunFlowerGiven"]),
            killed_ghost_marmu: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGhostMarmu"]),
            mum_caterpillar_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mumCaterpillarDefeated"]),
            killed_traitor_lord: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedTraitorLord"]),
            given_white_lady_flower: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "givenWhiteLadyFlower"]),
            visited_deepnest: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedDeepnest"]),
            visited_deepnest_spa: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedDeepnestSpa"]),
            zote_rescued_deepnest: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "zoteRescuedDeepnest"]),
            opened_tram_lower: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "openedTramLower"]),
            killed_mimic_spider: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedMimicSpider"]),
            killed_ghost_galien: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGhostGalien"]),
            galien_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "galienDefeated"]),
            spider_capture: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "spiderCapture"]),
            has_godfinder: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasGodfinder"]),
            given_godseeker_flower: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "givenGodseekerFlower"]),
            visited_godhome: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "visitedGodhome"]),
            zote_statue_wall_broken: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "zoteStatueWallBroken"]),
            ordeal_achieved: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "ordealAchieved"]),
            killed_nail_bros: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedNailBros"]),
            killed_paintmaster: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedPaintmaster"]),
            killed_nailsage: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedNailsage"]),
            killed_hollow_knight_prime: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedHollowKnightPrime"]),
        }
    }
}

/// boss door state pointers to BossSequenceDoorCompletion
struct CompletionPointers {
    boss_door_state_tier1: UnityPointer<3>,
    boss_door_state_tier2: UnityPointer<3>,
    boss_door_state_tier3: UnityPointer<3>,
    boss_door_state_tier4: UnityPointer<3>,
    boss_door_state_tier5: UnityPointer<3>,
}

impl CompletionPointers {
    fn new() -> CompletionPointers {
        CompletionPointers {
            boss_door_state_tier1: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "bossDoorStateTier1"]),
            boss_door_state_tier2: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "bossDoorStateTier2"]),
            boss_door_state_tier3: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "bossDoorStateTier3"]),
            boss_door_state_tier4: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "bossDoorStateTier4"]),
            boss_door_state_tier5: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "bossDoorStateTier5"]),
        }
    }
}

#[derive(bytemuck::CheckedBitPattern, Clone, Copy)] // bytemuck::Zeroable
#[repr(C)]
pub struct BossSequenceDoorCompletion {
    can_unlock: bool, // canUnlock
    unlocked: bool,
    pub completed: bool,
    all_bindings: bool, // allBindings
    no_hits: bool, // noHits
    bound_nail: bool, // boundNail
    bound_shell: bool, // boundShell
    bound_charms: bool, // boundCharms
    bound_soul: bool, // boundSoul
}

// --------------------------------------------------------
// --------------------------------------------------------

pub struct GameManagerFinder {
    string_list_offests: Box<StringListOffsets>,
    module: Box<mono::Module>,
    image: mono::Image,
    pointers: Box<GameManagerPointers>,
    player_data_pointers: Box<PlayerDataPointers>,
    completion_pointers: Box<CompletionPointers>,
    ui_state_offset: OnceCell<u32>,
}

impl GameManagerFinder {
    fn new(pointer_size: PointerSize, module: mono::Module, image: mono::Image) -> GameManagerFinder {
        GameManagerFinder {
            string_list_offests: Box::new(StringListOffsets::new(pointer_size)),
            module: Box::new(module),
            image,
            pointers: Box::new(GameManagerPointers::new()),
            player_data_pointers: Box::new(PlayerDataPointers::new()),
            completion_pointers: Box::new(CompletionPointers::new()),
            ui_state_offset: OnceCell::new(),
        }
    }

    pub async fn wait_attach(process: &Process) -> GameManagerFinder {
        let pointer_size = process_pointer_size(process).unwrap_or(PointerSize::Bit64);
        asr::print_message(&format!("GameManagerFinder wait_attach: pointer_size = {:?}", pointer_size));
        asr::print_message("GameManagerFinder wait_attach: Module wait_attach_auto_detect...");
        next_tick().await;
        let mut found_module = false;
        let mut needed_retry = false;
        loop {
            let module = mono::Module::wait_attach_auto_detect(process).await;
            if !found_module {
                found_module = true;
                asr::print_message("GameManagerFinder wait_attach: module get_default_image...");
                next_tick().await;
            }
            for _ in 0..0x10 {
                if let Some(image) = module.get_default_image(process) {
                    asr::print_message("GameManagerFinder wait_attach: got module and image");
                    next_tick().await;
                    return GameManagerFinder::new(pointer_size, module, image);
                }
                next_tick().await;
            }
            if !needed_retry {
                needed_retry = true;
                asr::print_message("GameManagerFinder wait_attach: retry...");
                next_tick().await;
            }
        }
    }

    fn deref_pointer<const PN: usize>(&self, process: &Process, pointer: &UnityPointer<PN>) -> Result<Address, asr::Error> {
        let a = pointer.deref_offsets(process, &self.module, &self.image)?;
        process.read_pointer(a, self.string_list_offests.pointer_size)
    }

    pub fn get_scene_name(&self, process: &Process) -> Option<String> {
        let s = self.deref_pointer(process, &self.pointers.scene_name).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, &self.string_list_offests, s)
    }

    pub fn get_next_scene_name(&self, process: &Process) -> Option<String> {
        let s = self.deref_pointer(process, &self.pointers.next_scene_name).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, &self.string_list_offests, s)
    }

    pub fn get_game_state(&self, process: &Process) -> Option<i32> {
        self.pointers.game_state.deref(process, &self.module, &self.image).ok()
    }

    fn is_game_state_non_menu(&self, process: &Process) -> bool {
        self.get_game_state(process).is_some_and(|gs| NON_MENU_GAME_STATES.contains(&gs))
    }

    fn is_game_state_non_continuous(&self, process: &Process) -> bool {
        self.get_game_state(process).is_some_and(|gs| NON_CONTINUOUS_GAME_STATES.contains(&gs))
    }

    pub fn get_ui_state(&self, process: &Process) -> Option<i32> {
        // save the uiState offset so it doesn't have to find it in the UIManager class every time
        let ui_state_offset = if let Some(ui_state_offset) = self.ui_state_offset.get() {
            ui_state_offset
        } else {
            let ui_manager_class = self.image.get_class(process, &self.module, "UIManager")?;
            let ui_state_offset = ui_manager_class.get_field_offset(process, &self.module, "uiState")?;
            self.ui_state_offset.get_or_init(|| ui_state_offset)
        };
        let ui = if let Ok(ui) = self.pointers.ui_state_vanilla.deref(process, &self.module, &self.image) {
            ui
        } else if let Ok(ui) =  self.pointers.ui_state_modded.deref(process, &self.module, &self.image) {
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
        self.pointers.camera_teleporting.deref(process, &self.module, &self.image).ok()
    }

    pub fn hazard_respawning(&self, process: &Process) -> Option<bool> {
        self.pointers.hazard_respawning.deref(process, &self.module, &self.image).ok()
    }

    pub fn accepting_input(&self, process: &Process) -> Option<bool> {
        self.pointers.accepting_input.deref(process, &self.module, &self.image).ok()
    }

    pub fn hero_transition_state(&self, process: &Process) -> Option<i32> {
        self.pointers.hero_transition_state.deref(process, &self.module, &self.image).ok()
    }

    pub fn focusing(&self, process: &Process) -> Option<bool> {
        self.pointers.focusing.deref(process, &self.module, &self.image).ok()
    }

    pub fn tile_map_dirty(&self, process: &Process) -> Option<bool> {
        self.pointers.tile_map_dirty.deref(process, &self.module, &self.image).ok()
    }

    pub fn uses_scene_transition_routine(&self, process: &Process) -> Option<bool> {
        /*
         * 1.3.1.5 and above swap from using LoadSceneAdditive to a SceneTransitionRoutine triggered
         * by BeginSceneTransitionRoutine, which doesn't set tilemapDirty back to false when you enter dnail
         * However, the early control glitch can only be performed on early patches so we can avoid this check entirely
         */
        // On current patch, return true
        Some(*self.get_version_vec(process)?.get(VERSION_VEC_MINOR)? >= 3)
    }

    pub fn hero_dead(&self, process: &Process) -> Option<bool> {
        self.pointers.hero_dead.deref(process, &self.module, &self.image).ok()
    }

    pub fn hazard_death(&self, process: &Process) -> Option<bool> {
        self.pointers.hazard_death.deref(process, &self.module, &self.image).ok()
    }

    fn hero_recoiling(&self, process: &Process) -> Option<bool> {
        self.pointers.hero_recoiling.deref(process, &self.module, &self.image).ok()
    }

    fn hero_recoil_frozen(&self, process: &Process) -> Option<bool> {
        self.pointers.hero_recoil_frozen.deref(process, &self.module, &self.image).ok()
    }

    pub fn hero_recoil(&self, process: &Process) -> Option<bool> {
        let maybe_recoil_frozen = self.hero_recoil_frozen(process);
        if maybe_recoil_frozen.is_some_and(|f| f) {
            return Some(true);
        }
        let maybe_recoiling = self.hero_recoiling(process);
        if maybe_recoiling.is_some_and(|r| r) {
            return Some(true);
        }
        Some(maybe_recoil_frozen? || maybe_recoiling?)
    }

    pub fn get_version_string(&self, process: &Process) -> Option<String> {
        let s: Address = [&self.pointers.version_number, &self.player_data_pointers.version].into_iter().find_map(|ptr| {
            self.deref_pointer(process, ptr).ok()
        })?;
        read_string_object::<SCENE_PATH_SIZE>(process, &self.string_list_offests, s)
    }

    pub fn get_version_vec(&self, process: &Process) -> Option<Vec<i32>> {
        Some(self.get_version_string(process)?.split('.').map(|s| {
            s.parse().unwrap_or(0)
        }).collect())
    }

    pub fn disable_pause(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.disable_pause.deref(process, &self.module, &self.image).ok()
    }

    pub fn get_health(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.health.deref(process, &self.module, &self.image).ok()
    }

    pub fn get_max_health(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.max_health.deref(process, &self.module, &self.image).ok()
    }

    pub fn get_mpcharge(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.mpcharge.deref(process, &self.module, &self.image).ok()
    }

    pub fn get_fireball_level(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.fireball_level.deref(process, &self.module, &self.image).ok()
    }

    pub fn get_quake_level(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.quake_level.deref(process, &self.module, &self.image).ok()
    }

    pub fn get_scream_level(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.scream_level.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_dash(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_dash.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_shadow_dash(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_shadow_dash.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_wall_jump(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_wall_jump.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_double_jump(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_double_jump.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_super_dash(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_super_dash.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_acid_armour(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_acid_armour.deref(process, &self.module, &self.image).ok()
    }

    /// hasCyclone: actually means Cyclone Slash, from Mato
    pub fn has_cyclone(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_cyclone.deref(process, &self.module, &self.image).ok()
    }

    /// hasDashSlash: secretly means Great Slash, from Sheo
    pub fn has_dash_slash(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_dash_slash.deref(process, &self.module, &self.image).ok()
    }

    /// hasUpwardSlash: secretly means Dash Slash, from Oro
    pub fn has_upward_slash(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_upward_slash.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_dream_nail(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_dream_nail.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_dream_gate(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_dream_gate.deref(process, &self.module, &self.image).ok()
    }

    pub fn dream_nail_upgraded(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.dream_nail_upgraded.deref(process, &self.module, &self.image).ok()
    }

    pub fn max_health_base(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.max_health_base.deref(process, &self.module, &self.image).ok()
    }

    pub fn heart_pieces(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.heart_pieces.deref(process, &self.module, &self.image).ok()
    }

    pub fn mp_reserve_max(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.mp_reserve_max.deref(process, &self.module, &self.image).ok()
    }

    pub fn vessel_fragments(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.vessel_fragments.deref(process, &self.module, &self.image).ok()
    }

    pub fn at_bench(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.at_bench.deref(process, &self.module, &self.image).ok()
    }

    // Dreamers

    pub fn mask_broken_lurien(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.mask_broken_lurien.deref(process, &self.module, &self.image).ok()
    }

    pub fn mask_broken_monomon(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.mask_broken_monomon.deref(process, &self.module, &self.image).ok()
    }

    pub fn mask_broken_hegemol(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.mask_broken_hegemol.deref(process, &self.module, &self.image).ok()
    }

    pub fn guardians_defeated(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.guardians_defeated.deref(process, &self.module, &self.image).ok()
    }

    // Old Dreamer Timings, mark deprecated or whatever
    pub fn lurien_defeated(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.lurien_defeated.deref(process, &self.module, &self.image).ok()
    }
    pub fn monomon_defeated(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.monomon_defeated.deref(process, &self.module, &self.image).ok()
    }
    pub fn hegemol_defeated(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.hegemol_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn mr_mushroom_state(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.mr_mushroom_state.deref(process, &self.module, &self.image).ok()
    }

    // Keys

    pub fn has_city_key(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_city_key.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_lantern(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_lantern.deref(process, &self.module, &self.image).ok()
    }

    pub fn simple_keys(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.simple_keys.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_sly_key(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_sly_key.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_white_key(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_white_key.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_love_key(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_love_key.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_lurker_key(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_lurker_key.deref(process, &self.module, &self.image).ok()
    }

    pub fn sly_simple_key(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.sly_simple_key.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_kings_brand(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_kings_brand.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_tram_pass(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_tram_pass.deref(process, &self.module, &self.image).ok()
    }

    #[cfg(debug_assertions)]
    pub fn get_geo(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.geo.deref(process, &self.module, &self.image).ok()
    }

    // Nail and Pale Ore

    pub fn nail_smith_upgrades(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.nail_smith_upgrades.deref(process, &self.module, &self.image).ok()
    }
    pub fn ore(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.ore.deref(process, &self.module, &self.image).ok()
    }
    pub fn ore_gross(&self, process: &Process) -> Option<i32> {
        let upgrades = self.nail_smith_upgrades(process)?;
        let ore_from_upgrades = (upgrades * (upgrades - 1)) / 2;
        Some(ore_from_upgrades + self.ore(process)?)
    }

    // Stags

    pub fn opened_crossroads(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.opened_crossroads.deref(process, &self.module, &self.image).ok()
    }

    pub fn opened_greenpath(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.opened_greenpath.deref(process, &self.module, &self.image).ok()
    }

    pub fn opened_fungal_wastes(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.opened_fungal_wastes.deref(process, &self.module, &self.image).ok()
    }

    pub fn opened_ruins1(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.opened_ruins1.deref(process, &self.module, &self.image).ok()
    }

    pub fn opened_ruins2(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.opened_ruins2.deref(process, &self.module, &self.image).ok()
    }

    pub fn opened_resting_grounds(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.opened_resting_grounds.deref(process, &self.module, &self.image).ok()
    }

    pub fn opened_hidden_station(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.opened_hidden_station.deref(process, &self.module, &self.image).ok()
    }

    pub fn opened_deepnest(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.opened_deepnest.deref(process, &self.module, &self.image).ok()
    }

    pub fn opened_royal_gardens(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.opened_royal_gardens.deref(process, &self.module, &self.image).ok()
    }

    pub fn opened_stag_nest(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.opened_stag_nest.deref(process, &self.module, &self.image).ok()
    }

    pub fn travelling(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.travelling.deref(process, &self.module, &self.image).ok()
    }

    // Relics
    #[allow(unused)]
    pub fn trinket1(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.trinket1.deref(process, &self.module, &self.image).ok()
    }
    pub fn trinket2(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.trinket2.deref(process, &self.module, &self.image).ok()
    }
    #[allow(unused)]
    pub fn trinket3(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.trinket3.deref(process, &self.module, &self.image).ok()
    }
    pub fn trinket4(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.trinket4.deref(process, &self.module, &self.image).ok()
    }

    pub fn sold_trinket1(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.sold_trinket1.deref(process, &self.module, &self.image).ok()
    }
    pub fn sold_trinket2(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.sold_trinket2.deref(process, &self.module, &self.image).ok()
    }
    pub fn sold_trinket3(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.sold_trinket3.deref(process, &self.module, &self.image).ok()
    }
    pub fn sold_trinket4(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.sold_trinket4.deref(process, &self.module, &self.image).ok()
    }
    pub fn sold_trinkets_geo(&self, p: &Process) -> Option<i32> {
        Some(200 * self.sold_trinket1(p)?
             + 450 * self.sold_trinket2(p)?
             + 800 * self.sold_trinket3(p)?
             + 1200 * self.sold_trinket4(p)?)
    }

    pub fn rancid_eggs(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.rancid_eggs.deref(process, &self.module, &self.image).ok()
    }

    pub fn jinn_eggs_sold(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.jinn_eggs_sold.deref(process, &self.module, &self.image).ok()
    }

    // Charm Notches
    pub fn notch_shroom_ogres(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.notch_shroom_ogres.deref(process, &self.module, &self.image).ok()
    }
    pub fn salubra_notch1(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.salubra_notch1.deref(process, &self.module, &self.image).ok()
    }
    pub fn salubra_notch2(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.salubra_notch2.deref(process, &self.module, &self.image).ok()
    }
    pub fn salubra_notch3(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.salubra_notch3.deref(process, &self.module, &self.image).ok()
    }
    pub fn salubra_notch4(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.salubra_notch4.deref(process, &self.module, &self.image).ok()
    }
    pub fn notch_fog_canyon(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.notch_fog_canyon.deref(process, &self.module, &self.image).ok()
    }
    pub fn got_grimm_notch(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_grimm_notch.deref(process, &self.module, &self.image).ok()
    }

    pub fn can_overcharm(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.can_overcharm.deref(process, &self.module, &self.image).ok()
    }

    // Charms

    pub fn got_charm_1(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_1.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_2(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_2.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_3(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_3.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_4(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_4.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_5(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_5.deref(process, &self.module, &self.image).ok()
    }

    pub fn equipped_charm_5(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.equipped_charm_5.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_6(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_6.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_7(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_7.deref(process, &self.module, &self.image).ok()
    }

    pub fn equipped_charm_7(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.equipped_charm_7.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_8(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_8.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_9(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_9.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_10(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_10.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_11(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_11.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_12(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_12.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_13(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_13.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_14(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_14.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_15(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_15.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_16(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_16.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_17(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_17.deref(process, &self.module, &self.image).ok()
    }

    pub fn equipped_charm_17(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.equipped_charm_17.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_18(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_18.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_19(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_19.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_20(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_20.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_21(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_21.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_22(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_22.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_26(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_26.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_27(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_27.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_28(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_28.deref(process, &self.module, &self.image).ok()
    }

    pub fn equipped_charm_28(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.equipped_charm_28.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_29(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_29.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_30(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_30.deref(process, &self.module, &self.image).ok()
    }

    // Dashmaster
    pub fn got_charm_31(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_31.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_32(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_32.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_33(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_33.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_34(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_34.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_35(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_35.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_37(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_37.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_38(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_38.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_39(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_39.deref(process, &self.module, &self.image).ok()
    }

    // Fragile / Unbreakable Charms

    pub fn got_charm_23(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_23.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_24(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_24.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_25(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_25.deref(process, &self.module, &self.image).ok()
    }

    pub fn broken_charm_23(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.broken_charm_23.deref(process, &self.module, &self.image).ok()
    }

    pub fn broken_charm_24(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.broken_charm_24.deref(process, &self.module, &self.image).ok()
    }

    pub fn broken_charm_25(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.broken_charm_25.deref(process, &self.module, &self.image).ok()
    }

    pub fn fragile_greed_unbreakable(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.fragile_greed_unbreakable.deref(process, &self.module, &self.image).ok()
    }

    pub fn fragile_health_unbreakable(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.fragile_health_unbreakable.deref(process, &self.module, &self.image).ok()
    }

    pub fn fragile_strength_unbreakable(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.fragile_strength_unbreakable.deref(process, &self.module, &self.image).ok()
    }

    // Grimmchild / Carefree Melody

    pub fn got_charm_40(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_40.deref(process, &self.module, &self.image).ok()
    }

    pub fn equipped_charm_40(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.equipped_charm_40.deref(process, &self.module, &self.image).ok()
    }

    pub fn grimm_child_level(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.grimm_child_level.deref(process, &self.module, &self.image).ok()
    }

    pub fn flames_collected(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.flames_collected.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_brumms_flame(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_brumms_flame.deref(process, &self.module, &self.image).ok()
    }

    // Kingsoul / VoidHeart

    pub fn charm_cost_36(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.charm_cost_36.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_queen_fragment(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_queen_fragment.deref(process, &self.module, &self.image).ok()
    }
    pub fn got_king_fragment(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_king_fragment.deref(process, &self.module, &self.image).ok()
    }

    pub fn royal_charm_state(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.royal_charm_state.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_shade_charm(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_shade_charm.deref(process, &self.module, &self.image).ok()
    }

    pub fn grubs_collected(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.grubs_collected.deref(process, &self.module, &self.image).ok()
    }

    pub fn scenes_grub_rescued(&self, process: &Process) -> Option<Vec<String>> {
        let l = self.deref_pointer(process, &self.player_data_pointers.scenes_grub_rescued).ok()?;
        read_string_list_object::<SCENE_PATH_SIZE>(process, &self.string_list_offests, l)
    }

    pub fn grub_waterways_isma(&self, process: &Process) -> Option<bool> {
        Some(self.scenes_grub_rescued(process)?.contains(&"Waterways_13".to_string()))
    }

    pub fn kills_grub_mimic(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.kills_grub_mimic.deref(process, &self.module, &self.image).ok()
    }

    pub fn dream_orbs(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.dream_orbs.deref(process, &self.module, &self.image).ok()
    }

    pub fn scenes_encountered_dream_plant_c(&self, process: &Process) -> Option<Vec<String>> {
        let l = self.deref_pointer(process, &self.player_data_pointers.scenes_encountered_dream_plant_c).ok()?;
        read_string_list_object::<SCENE_PATH_SIZE>(process, &self.string_list_offests, l)
    }

    pub fn dream_gate_scene(&self, process: &Process) -> Option<String> {
        let s = self.deref_pointer(process, &self.player_data_pointers.dream_gate_scene).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, &self.string_list_offests, s)
    }
    pub fn dream_gate_x(&self, process: &Process) -> Option<f32> {
        self.player_data_pointers.dream_gate_x.deref(process, &self.module, &self.image).ok()
    }
    pub fn dream_gate_y(&self, process: &Process) -> Option<f32> {
        self.player_data_pointers.dream_gate_y.deref(process, &self.module, &self.image).ok()
    }

    pub fn map_dirtmouth(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.map_dirtmouth.deref(process, &self.module, &self.image).ok()
    }
    pub fn map_crossroads(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.map_crossroads.deref(process, &self.module, &self.image).ok()
    }
    pub fn map_greenpath(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.map_greenpath.deref(process, &self.module, &self.image).ok()
    }
    pub fn map_fog_canyon(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.map_fog_canyon.deref(process, &self.module, &self.image).ok()
    }
    pub fn map_royal_gardens(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.map_royal_gardens.deref(process, &self.module, &self.image).ok()
    }
    pub fn map_fungal_wastes(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.map_fungal_wastes.deref(process, &self.module, &self.image).ok()
    }
    pub fn map_city(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.map_city.deref(process, &self.module, &self.image).ok()
    }
    pub fn map_waterways(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.map_waterways.deref(process, &self.module, &self.image).ok()
    }
    pub fn map_mines(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.map_mines.deref(process, &self.module, &self.image).ok()
    }
    pub fn map_deepnest(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.map_deepnest.deref(process, &self.module, &self.image).ok()
    }
    pub fn map_cliffs(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.map_cliffs.deref(process, &self.module, &self.image).ok()
    }
    pub fn map_outskirts(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.map_outskirts.deref(process, &self.module, &self.image).ok()
    }
    pub fn map_resting_grounds(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.map_resting_grounds.deref(process, &self.module, &self.image).ok()
    }
    pub fn map_abyss(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.map_abyss.deref(process, &self.module, &self.image).ok()
    }

    pub fn visited_dirtmouth(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_dirtmouth.deref(process, &self.module, &self.image).ok()
    }

    pub fn sly_shell_frag1(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.sly_shell_frag1.deref(process, &self.module, &self.image).ok()
    }
    pub fn sly_shell_frag4(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.sly_shell_frag4.deref(process, &self.module, &self.image).ok()
    }
    pub fn sly_vessel_frag1(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.sly_vessel_frag1.deref(process, &self.module, &self.image).ok()
    }
    pub fn sly_vessel_frag2(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.sly_vessel_frag2.deref(process, &self.module, &self.image).ok()
    }
    pub fn sly_shop_finished(&self, p: &Process) -> Option<bool> {
        Some(self.has_lantern(p)?
             && self.got_charm_1(p)?
             && self.got_charm_4(p)?
             && self.got_charm_15(p)?
             && self.got_charm_37(p)?
             && self.sly_shell_frag4(p)?
             && self.sly_vessel_frag2(p)?)
    }

    pub fn elderbug_gave_flower(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.elderbug_gave_flower.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_grimm(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_grimm.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_nightmare_grimm(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_nightmare_grimm.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_grey_prince(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_grey_prince.deref(process, &self.module, &self.image).ok()
    }

    pub fn grey_prince_orbs_collected(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.grey_prince_orbs_collected.deref(process, &self.module, &self.image).ok()
    }

    pub fn visited_crossroads(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_crossroads.deref(process, &self.module, &self.image).ok()
    }
    pub fn crossroads_infected(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.crossroads_infected.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_mender_bug(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_mender_bug.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_mawlek(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_mawlek.deref(process, &self.module, &self.image).ok()
    }

    // Gruz Mother
    pub fn killed_big_fly(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_big_fly.deref(process, &self.module, &self.image).ok()
    }

    pub fn sly_rescued(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.sly_rescued.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_false_knight(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_false_knight.deref(process, &self.module, &self.image).ok()
    }

    pub fn false_knight_dream_defeated(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.false_knight_dream_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn false_knight_orbs_collected(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.false_knight_orbs_collected.deref(process, &self.module, &self.image).ok()
    }

    pub fn salubra_blessing(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.salubra_blessing.deref(process, &self.module, &self.image).ok()
    }

    pub fn unchained_hollow_knight(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.unchained_hollow_knight.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_hollow_knight(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_hollow_knight.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_final_boss(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_final_boss.deref(process, &self.module, &self.image).ok()
    }

    pub fn visited_greenpath(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_greenpath.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_moss_knight(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_moss_knight.deref(process, &self.module, &self.image).ok()
    }

    pub fn zote_rescued_buzzer(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.zote_rescued_buzzer.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_hornet(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_hornet.deref(process, &self.module, &self.image).ok()
    }

    /// killedLazyFlyer: Aluba
    pub fn killed_lazy_flyer(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_lazy_flyer.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_hunter_mark(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_hunter_mark.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_ghost_no_eyes(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_ghost_no_eyes.deref(process, &self.module, &self.image).ok()
    }
    pub fn no_eyes_defeated(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.no_eyes_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn mega_moss_charger_defeated(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.mega_moss_charger_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn nailsmith_convo_art(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.nailsmith_convo_art.deref(process, &self.module, &self.image).ok()
    }

    pub fn visited_fungus(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_fungus.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_ghost_hu(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_ghost_hu.deref(process, &self.module, &self.image).ok()
    }
    pub fn elder_hu_defeated(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.elder_hu_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn bretta_rescued(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.bretta_rescued.deref(process, &self.module, &self.image).ok()
    }

    pub fn defeated_mantis_lords(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.defeated_mantis_lords.deref(process, &self.module, &self.image).ok()
    }

    // Gorb
    pub fn killed_ghost_aladar(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_ghost_aladar.deref(process, &self.module, &self.image).ok()
    }
    pub fn aladar_slug_defeated(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.aladar_slug_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn nightmare_lantern_lit(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.nightmare_lantern_lit.deref(process, &self.module, &self.image).ok()
    }
    pub fn destroyed_nightmare_lantern(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.destroyed_nightmare_lantern.deref(process, &self.module, &self.image).ok()
    }

    pub fn visited_resting_grounds(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_resting_grounds.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_ghost_xero(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_ghost_xero.deref(process, &self.module, &self.image).ok()
    }
    pub fn xero_defeated(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.xero_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn glade_door_opened(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.glade_door_opened.deref(process, &self.module, &self.image).ok()
    }
    pub fn moth_departed(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.moth_departed.deref(process, &self.module, &self.image).ok()
    }

    /// Met Grey Mourner
    pub fn met_xun(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.met_xun.deref(process, &self.module, &self.image).ok()
    }

    /// Has Delicate Flower
    pub fn has_xun_flower(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_xun_flower.deref(process, &self.module, &self.image).ok()
    }

    /// Flower Reward Given
    pub fn xun_reward_given(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.xun_reward_given.deref(process, &self.module, &self.image).ok()
    }

    pub fn opened_city_gate(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.opened_city_gate.deref(process, &self.module, &self.image).ok()
    }

    pub fn visited_ruins(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_ruins.deref(process, &self.module, &self.image).ok()
    }

    // Lemm
    pub fn met_relic_dealer_shop(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.met_relic_dealer_shop.deref(process, &self.module, &self.image).ok()
    }

    pub fn toll_bench_city(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.toll_bench_city.deref(process, &self.module, &self.image).ok()
    }

    /// Killed Soul Twister
    pub fn killed_mage(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_mage.deref(process, &self.module, &self.image).ok()
    }

    /// Killed Soul Warrior
    pub fn killed_mage_knight(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_mage_knight.deref(process, &self.module, &self.image).ok()
    }

    pub fn mage_lord_encountered(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.mage_lord_encountered.deref(process, &self.module, &self.image).ok()
    }

    pub fn mage_lord_encountered_2(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.mage_lord_encountered_2.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_mage_lord(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_mage_lord.deref(process, &self.module, &self.image).ok()
    }

    pub fn mage_lord_dream_defeated(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.mage_lord_dream_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn mage_lord_orbs_collected(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.mage_lord_orbs_collected.deref(process, &self.module, &self.image).ok()
    }

    pub fn kills_great_shield_zombie(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.kills_great_shield_zombie.deref(process, &self.module, &self.image).ok()
    }

    pub fn watcher_chandelier(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.watcher_chandelier.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_black_knight(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_black_knight.deref(process, &self.module, &self.image).ok()
    }

    pub fn collector_defeated(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.collector_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn nailsmith_killed(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.nailsmith_killed.deref(process, &self.module, &self.image).ok()
    }
    pub fn nailsmith_spared(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.nailsmith_spared.deref(process, &self.module, &self.image).ok()
    }

    pub fn visited_mines(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_mines.deref(process, &self.module, &self.image).ok()
    }

    /// Defeated Crystal Guardian
    pub fn defeated_mega_beam_miner(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.defeated_mega_beam_miner.deref(process, &self.module, &self.image).ok()
    }

    /// Kills left to complete Crystal Guardian journal
    pub fn kills_mega_beam_miner(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.kills_mega_beam_miner.deref(process, &self.module, &self.image).ok()
    }

    pub fn mine_lift_opened(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.mine_lift_opened.deref(process, &self.module, &self.image).ok()
    }

    pub fn opened_waterways_manhole(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.opened_waterways_manhole.deref(process, &self.module, &self.image).ok()
    }

    pub fn visited_waterways(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_waterways.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_dung_defender(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_dung_defender.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_white_defender(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_white_defender.deref(process, &self.module, &self.image).ok()
    }

    pub fn white_defender_orbs_collected(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.white_defender_orbs_collected.deref(process, &self.module, &self.image).ok()
    }

    pub fn met_emilitia(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.met_emilitia.deref(process, &self.module, &self.image).ok()
    }

    pub fn given_emilitia_flower(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.given_emilitia_flower.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_fluke_mother(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_fluke_mother.deref(process, &self.module, &self.image).ok()
    }

    pub fn visited_abyss(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_abyss.deref(process, &self.module, &self.image).ok()
    }

    pub fn saved_cloth(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.saved_cloth.deref(process, &self.module, &self.image).ok()
    }

    pub fn toll_bench_abyss(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.toll_bench_abyss.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_infected_knight(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_infected_knight.deref(process, &self.module, &self.image).ok()
    }

    pub fn infected_knight_dream_defeated(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.infected_knight_dream_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn infected_knight_orbs_collected(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.infected_knight_orbs_collected.deref(process, &self.module, &self.image).ok()
    }

    pub fn abyss_gate_opened(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.abyss_gate_opened.deref(process, &self.module, &self.image).ok()
    }

    pub fn abyss_lighthouse(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.abyss_lighthouse.deref(process, &self.module, &self.image).ok()
    }

    pub fn visited_white_palace(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_white_palace.deref(process, &self.module, &self.image).ok()
    }

    pub fn white_palace_orb_1(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.white_palace_orb_1.deref(process, &self.module, &self.image).ok()
    }

    pub fn white_palace_orb_2(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.white_palace_orb_2.deref(process, &self.module, &self.image).ok()
    }

    pub fn white_palace_orb_3(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.white_palace_orb_3.deref(process, &self.module, &self.image).ok()
    }

    /// New data on hunter's journal entry Seal of Binding / Path of Pain
    pub fn new_data_binding_seal(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.new_data_binding_seal.deref(process, &self.module, &self.image).ok()
    }
    
    pub fn white_palace_secret_room_visited(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.white_palace_secret_room_visited.deref(process, &self.module, &self.image).ok()
    }

    /// Visited Kingdom's Edge
    pub fn visited_outskirts(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_outskirts.deref(process, &self.module, &self.image).ok()
    }
    pub fn visited_hive(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_hive.deref(process, &self.module, &self.image).ok()
    }

    fn hive_knight_doesnt_exist(&self, process: &Process) -> Option<bool> {
        let v = self.get_version_vec(process)?;
        Some((*v.get(VERSION_VEC_MAJOR)? <= 1) && (*v.get(VERSION_VEC_MINOR)? <= 2))
    }

    pub fn killed_hive_knight(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_hive_knight.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_giant_hopper(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_giant_hopper.deref(process, &self.module, &self.image).ok()
    }

    pub fn given_oro_flower(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.given_oro_flower.deref(process, &self.module, &self.image).ok()
    }

    pub fn hornet_outskirts_defeated(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.hornet_outskirts_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_ghost_markoth(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_ghost_markoth.deref(process, &self.module, &self.image).ok()
    }
    pub fn markoth_defeated(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.markoth_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn little_fool_met(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.little_fool_met.deref(process, &self.module, &self.image).ok()
    }

    pub fn colosseum_bronze_opened(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.colosseum_bronze_opened.deref(process, &self.module, &self.image).ok()
    }

    pub fn seen_colosseum_title(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.seen_colosseum_title.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_zote(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_zote.deref(process, &self.module, &self.image).ok()
    }

    pub fn colosseum_bronze_completed(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.colosseum_bronze_completed.deref(process, &self.module, &self.image).ok()
    }

    pub fn colosseum_silver_opened(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.colosseum_silver_opened.deref(process, &self.module, &self.image).ok()
    }

    pub fn colosseum_silver_completed(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.colosseum_silver_completed.deref(process, &self.module, &self.image).ok()
    }

    pub fn colosseum_gold_opened(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.colosseum_gold_opened.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_lobster_lancer(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_lobster_lancer.deref(process, &self.module, &self.image).ok()
    }

    pub fn colosseum_gold_completed(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.colosseum_gold_completed.deref(process, &self.module, &self.image).ok()
    }

    pub fn visited_fog_canyon(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_fog_canyon.deref(process, &self.module, &self.image).ok()
    }

    // Uumuu
    pub fn encountered_mega_jelly(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.encountered_mega_jelly.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_mega_jellyfish(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_mega_jellyfish.deref(process, &self.module, &self.image).ok()
    }

    pub fn visited_royal_gardens(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_royal_gardens.deref(process, &self.module, &self.image).ok()
    }

    pub fn toll_bench_queens_gardens(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.toll_bench_queens_gardens.deref(process, &self.module, &self.image).ok()
    }

    pub fn xun_flower_given(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.xun_flower_given.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_ghost_marmu(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_ghost_marmu.deref(process, &self.module, &self.image).ok()
    }
    pub fn mum_caterpillar_defeated(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.mum_caterpillar_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_traitor_lord(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_traitor_lord.deref(process, &self.module, &self.image).ok()
    }

    pub fn given_white_lady_flower(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.given_white_lady_flower.deref(process, &self.module, &self.image).ok()
    }

    pub fn visited_deepnest(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_deepnest.deref(process, &self.module, &self.image).ok()
    }
    pub fn visited_deepnest_spa(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_deepnest_spa.deref(process, &self.module, &self.image).ok()
    }

    pub fn zote_rescued_deepnest(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.zote_rescued_deepnest.deref(process, &self.module, &self.image).ok()
    }

    pub fn opened_tram_lower(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.opened_tram_lower.deref(process, &self.module, &self.image).ok()
    }


    pub fn killed_mimic_spider(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_mimic_spider.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_ghost_galien(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_ghost_galien.deref(process, &self.module, &self.image).ok()
    }
    pub fn galien_defeated(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.galien_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn spider_capture(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.spider_capture.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_godfinder(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_godfinder.deref(process, &self.module, &self.image).ok()
    }

    pub fn given_godseeker_flower(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.given_godseeker_flower.deref(process, &self.module, &self.image).ok()
    }

    pub fn visited_godhome(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.visited_godhome.deref(process, &self.module, &self.image).ok()
    }

    pub fn zote_statue_wall_broken(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.zote_statue_wall_broken.deref(process, &self.module, &self.image).ok()
    }
    pub fn ordeal_achieved(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.ordeal_achieved.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_nail_bros(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_nail_bros.deref(process, &self.module, &self.image).ok()
    }

    pub fn boss_door_state_tier1(&self, process: &Process) -> Option<BossSequenceDoorCompletion> {
        self.completion_pointers.boss_door_state_tier1.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_paintmaster(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_paintmaster.deref(process, &self.module, &self.image).ok()
    }

    pub fn boss_door_state_tier2(&self, process: &Process) -> Option<BossSequenceDoorCompletion> {
        self.completion_pointers.boss_door_state_tier2.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_nailsage(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_nailsage.deref(process, &self.module, &self.image).ok()
    }

    pub fn boss_door_state_tier3(&self, process: &Process) -> Option<BossSequenceDoorCompletion> {
        self.completion_pointers.boss_door_state_tier3.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_hollow_knight_prime(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_hollow_knight_prime.deref(process, &self.module, &self.image).ok()
    }

    pub fn boss_door_state_tier4(&self, process: &Process) -> Option<BossSequenceDoorCompletion> {
        self.completion_pointers.boss_door_state_tier4.deref(process, &self.module, &self.image).ok()
    }

    pub fn boss_door_state_tier5(&self, process: &Process) -> Option<BossSequenceDoorCompletion> {
        self.completion_pointers.boss_door_state_tier5.deref(process, &self.module, &self.image).ok()
    }


}

pub struct SceneStore {
    prev_scene_name: String,
    curr_scene_name: String,
    next_scene_name: String,
    new_data_curr: bool,
    new_data_next: bool,
    last_next: bool,
    pub split_this_transition: bool,
}

impl SceneStore {
    pub fn new() -> SceneStore {
        SceneStore {
            prev_scene_name: "".to_string(),
            curr_scene_name: "".to_string(),
            next_scene_name: "".to_string(),
            new_data_curr: false,
            new_data_next: false,
            last_next: true,
            split_this_transition: false,
        }
    }

    pub fn pair(&self) -> Pair<&str> {
        if self.last_next && self.next_scene_name != self.curr_scene_name {
            Pair { old: &self.curr_scene_name, current: &self.next_scene_name }
        } else {
            Pair { old: &self.prev_scene_name, current: &self.curr_scene_name }
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

    pub fn transition_now(&mut self, prc: &Process, g: &GameManagerFinder) -> bool {
        self.new_curr_scene_name1(g.get_scene_name(prc));
        self.new_next_scene_name1(g.get_next_scene_name(prc));

        if self.new_data_next {
            self.new_data_curr = false;
            self.new_data_next = false;
            self.last_next = true;
            self.split_this_transition = false;
            #[cfg(debug_assertions)]
            asr::print_message(&format!("curr {} -> next {}", &self.curr_scene_name, &self.next_scene_name));
            true
        } else if self.new_data_curr {
            self.new_data_curr = false;
            self.last_next = false;
            self.split_this_transition = false;
            #[cfg(debug_assertions)]
            asr::print_message(&format!("prev {} -> curr {}", &self.prev_scene_name, &self.curr_scene_name));
            true
        } else {
            false
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
    pub fn clean_on_entry(&mut self) {
        self.map_i32.retain(|k, _| !k.ends_with("_on_entry"));
        self.map_bool.retain(|k, _| !k.ends_with("_on_entry"));
    }

    fn get_bool<const N: usize>(&mut self, p: &Process, g: &GameManagerFinder, key: &'static str, pointer: &UnityPointer<N>) -> Option<bool> {
        if !g.is_game_state_non_menu(p) {
            return self.map_bool.get(key).copied();
        };
        let Ok(b) = pointer.deref(p, &g.module, &g.image) else {
            return self.map_bool.get(key).copied();
        };
        self.map_bool.insert(key, b);
        Some(b)
    }

    fn changed_bool<const N: usize>(&mut self, p: &Process, g: &GameManagerFinder, key: &'static str, pointer: &UnityPointer<N>) -> Option<bool> {
        let store_val = self.map_bool.get(key).copied();
        let player_data_val = pointer.deref(p, &g.module, &g.image).ok();
        if let Some(val) = player_data_val {
            if val || g.is_game_state_non_menu(p) {
                self.map_bool.insert(key, val);
            }
        }
        if player_data_val? == store_val? { return None; }
        player_data_val
    }

    fn get_i32<const N: usize>(&mut self, p: &Process, g: &GameManagerFinder, key: &'static str, pointer: &UnityPointer<N>) -> Option<i32> {
        if !g.is_game_state_non_menu(p) {
            return self.map_i32.get(key).copied();
        };
        let Ok(i) = pointer.deref(p, &g.module, &g.image) else {
            return self.map_i32.get(key).copied();
        };
        self.map_i32.insert(key, i);
        Some(i)
    }

    fn changed_i32_delta<const N: usize>(&mut self, p: &Process, g: &GameManagerFinder, key: &'static str, pointer: &UnityPointer<N>) -> Option<i32> {
        let store_val = self.map_i32.get(key).cloned();
        let player_data_val = pointer.deref(p, &g.module, &g.image).ok();
        if let Some(val) = player_data_val {
            if val != 0 || g.is_game_state_non_menu(p) {
                self.map_i32.insert(key, val);
            }
        }
        Some(player_data_val? - store_val?)
    }

    fn incremented_i32<const N: usize>(&mut self, p: &Process, g: &GameManagerFinder, key: &'static str, pointer: &UnityPointer<N>) -> bool {
        self.changed_i32_delta(p, g, key, pointer).is_some_and(|d| d == 1)
    }

    fn increased_i32<const N: usize>(&mut self, p: &Process, g: &GameManagerFinder, key: &'static str, pointer: &UnityPointer<N>) -> bool {
        self.changed_i32_delta(p, g, key, pointer).is_some_and(|d| 0 < d)
    }

    fn decremented_i32<const N: usize>(&mut self, p: &Process, g: &GameManagerFinder, key: &'static str, pointer: &UnityPointer<N>) -> bool {
        self.changed_i32_delta(p, g, key, pointer).is_some_and(|d| d == -1)
    }

    #[cfg(debug_assertions)]
    pub fn get_game_state(&mut self, p: &Process, g: &GameManagerFinder) -> i32 {
        let Ok(i) = g.pointers.game_state.deref(p, &g.module, &g.image) else {
            return self.map_i32.get("game_state").copied().unwrap_or(0);
        };
        #[cfg(debug_assertions)]
        if self.map_i32.get("game_state").is_some_and(|&old| old != i) {
            asr::print_message(&format!("game_state: {}", i));
        }
        self.map_i32.insert("game_state", i);
        i
    }

    pub fn obtained_mask_shard(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.incremented_i32(p, g, "max_health_base", &g.player_data_pointers.max_health_base)
        || (self.incremented_i32(p, g, "heart_pieces", &g.player_data_pointers.heart_pieces)
            && self.map_i32.get("heart_pieces").is_some_and(|&s| s < 4))
    }

    pub fn shade_killed(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.changed_bool(p, g, "soul_limited", &g.player_data_pointers.soul_limited).is_some_and(|l| !l)
    }

    pub fn obtained_vessel_fragment(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.increased_i32(p, g, "mp_reserve_max", &g.player_data_pointers.mp_reserve_max)
        || (self.incremented_i32(p, g, "vessel_fragments", &g.player_data_pointers.vessel_fragments)
            && self.map_i32.get("vessel_fragments").is_some_and(|&f| f < 3))
    }

    pub fn guardians_defeated(&mut self, p: &Process, g: &GameManagerFinder) -> i32 {
        self.get_i32(p, g, "guardians_defeated", &g.player_data_pointers.guardians_defeated).unwrap_or(0)
    }

    pub fn get_fireball_level(&mut self, p: &Process, g: &GameManagerFinder) -> i32 {
        self.get_i32(p, g, "fireball_level", &g.player_data_pointers.fireball_level).unwrap_or(0)
    }

    pub fn get_quake_level(&mut self, p: &Process, g: &GameManagerFinder) -> i32 {
        self.get_i32(p, g, "quake_level", &g.player_data_pointers.quake_level).unwrap_or(0)
    }

    pub fn has_dash(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "has_dash", &g.player_data_pointers.has_dash).unwrap_or(false)
    }

    pub fn has_wall_jump(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "has_wall_jump", &g.player_data_pointers.has_wall_jump).unwrap_or(false)
    }

    pub fn has_double_jump(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "has_double_jump", &g.player_data_pointers.has_double_jump).unwrap_or(false)
    }

    pub fn has_acid_armour(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "has_acid_armour", &g.player_data_pointers.has_acid_armour).unwrap_or(false)
    }

    pub fn has_dream_nail(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "has_dream_nail", &g.player_data_pointers.has_dream_nail).unwrap_or(false)
    }

    pub fn has_dream_gate(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "has_dream_gate", &g.player_data_pointers.has_dream_gate).unwrap_or(false)
    }

    pub fn has_lantern(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "has_lantern", &g.player_data_pointers.has_lantern).unwrap_or(false)
    }

    pub fn sly_shop_finished(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        if !g.is_game_state_non_menu(p) {
            return self.map_bool.get("sly_shop_finished").unwrap_or(&false).clone();
        };
        let Some(b) = g.sly_shop_finished(p) else {
            return self.map_bool.get("sly_shop_finished").unwrap_or(&false).clone();
        };
        self.map_bool.insert("sly_shop_finished", b);
        b
    }

    pub fn cornifer_at_home(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "cornifer_at_home", &g.player_data_pointers.cornifer_at_home).unwrap_or(false)
    }

    pub fn pure_snail(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        match g.focusing(p) {
            None => false,
            Some(false) => {
                if let Some(health) = g.get_health(p) {
                    self.map_i32.insert("health_before_focus", health);
                }
                if let Some(mpcharge) = g.get_mpcharge(p) {
                    self.map_i32.insert("mpcharge_before_focus", mpcharge);
                }
                false
            }
            Some(true) => {
                if !g.equipped_charm_5(p).is_some_and(|e| e) {
                    return false;
                }
                if !g.equipped_charm_7(p).is_some_and(|e| e) {
                    return false;
                }
                if !g.equipped_charm_17(p).is_some_and(|e| e) {
                    return false;
                }
                if !g.equipped_charm_28(p).is_some_and(|e| e) {
                    return false;
                }
                let Some(&health_before_focus) = self.map_i32.get("health_before_focus") else {
                    return false;
                };
                let Some(health) = g.get_health(p) else {
                    return false;
                };
                if health_before_focus < health {
                    return true;
                }
                let Some(max_health) = g.get_max_health(p) else {
                    return false;
                };
                if health != max_health {
                    return false;
                }
                let Some(&mpcharge_before_focus) = self.map_i32.get("mpcharge_before_focus") else {
                    return false;
                };
                let Some(mpcharge) = g.get_mpcharge(p) else {
                    return false;
                };
                mpcharge + 33 <= mpcharge_before_focus
            }
        }
    }

    pub fn got_charm_31(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "got_charm_31", &g.player_data_pointers.got_charm_31).unwrap_or(false)
    }

    pub fn got_shade_charm(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "got_shade_charm", &g.player_data_pointers.got_shade_charm).unwrap_or(false)
    }

    pub fn incremented_grubs_collected(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.incremented_i32(process, game_manager_finder, "grubs_collected", &game_manager_finder.player_data_pointers.grubs_collected)
    }

    pub fn grub_waterways_isma(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        if !g.is_game_state_non_menu(p) {
            return self.map_bool.get("grub_waterways_isma").unwrap_or(&false).clone();
        };
        let Some(b) = g.grub_waterways_isma(p) else {
            return self.map_bool.get("grub_waterways_isma").unwrap_or(&false).clone();
        };
        self.map_bool.insert("grub_waterways_isma", b);
        b
    }

    pub fn incremented_ore(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.incremented_i32(process, game_manager_finder, "ore", &game_manager_finder.player_data_pointers.ore)
    }

    pub fn changed_travelling_true(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.changed_bool(process, game_manager_finder, "travelling", &game_manager_finder.player_data_pointers.travelling).is_some_and(|t| t)
    }

    pub fn changed_stag_position(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.changed_i32_delta(process, game_manager_finder, "stag_position", &game_manager_finder.player_data_pointers.stag_position).is_some_and(|d| d != 0)
    }

    pub fn incremented_simple_keys(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.incremented_i32(process, game_manager_finder, "simple_keys", &game_manager_finder.player_data_pointers.simple_keys)
    }

    pub fn decremented_simple_keys(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.decremented_i32(process, game_manager_finder, "simple_keys", &game_manager_finder.player_data_pointers.simple_keys)
    }

    pub fn incremented_trinket1(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.incremented_i32(process, game_manager_finder, "trinket1", &game_manager_finder.player_data_pointers.trinket1)
    }

    pub fn incremented_trinket2(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.incremented_i32(process, game_manager_finder, "trinket2", &game_manager_finder.player_data_pointers.trinket2)
    }

    pub fn incremented_trinket3(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.incremented_i32(process, game_manager_finder, "trinket3", &game_manager_finder.player_data_pointers.trinket3)
    }

    pub fn incremented_trinket4(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.incremented_i32(process, game_manager_finder, "trinket4", &game_manager_finder.player_data_pointers.trinket4)
    }

    pub fn incremented_rancid_eggs(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.incremented_i32(process, game_manager_finder, "rancid_eggs", &game_manager_finder.player_data_pointers.rancid_eggs)
    }

    pub fn incremented_ghost_coins(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.incremented_i32(process, game_manager_finder, "ghost_coins", &game_manager_finder.player_data_pointers.ghost_coins)
    }

    pub fn incremented_charm_slots(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.incremented_i32(process, game_manager_finder, "charm_slots", &game_manager_finder.player_data_pointers.charm_slots)
    }

    pub fn can_overcharm(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "can_overcharm", &g.player_data_pointers.can_overcharm).unwrap_or(false)
    }

    pub fn incremented_dream_orbs(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.incremented_i32(process, game_manager_finder, "dream_orbs", &game_manager_finder.player_data_pointers.dream_orbs)
    }

    pub fn incremented_grey_prince_defeats(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.incremented_i32(process, game_manager_finder, "grey_prince_defeats", &game_manager_finder.player_data_pointers.grey_prince_defeats)
    }

    pub fn incremented_white_defender_defeats(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.incremented_i32(process, game_manager_finder, "white_defender_defeats", &game_manager_finder.player_data_pointers.white_defender_defeats)
    }

    pub fn zote_rescued_buzzer(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "zote_rescued_buzzer", &g.player_data_pointers.zote_rescued_buzzer).unwrap_or(false)
    }

    pub fn mega_moss_charger_defeated(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "mega_moss_charger_defeated", &g.player_data_pointers.mega_moss_charger_defeated).unwrap_or(false)
    }

    pub fn killed_ghost_hu(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "killed_ghost_hu", &g.player_data_pointers.killed_ghost_hu).unwrap_or(false)
    }

    pub fn killed_gorgeous_husk(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "killed_gorgeous_husk", &g.player_data_pointers.killed_gorgeous_husk).unwrap_or(false)
    }

    pub fn killed_black_knight(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "killed_black_knight", &g.player_data_pointers.killed_black_knight).unwrap_or(false)
    }

    pub fn collector_defeated(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "collector_defeated", &g.player_data_pointers.collector_defeated).unwrap_or(false)
    }

    pub fn killed_infected_knight(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "killed_infected_knight", &g.player_data_pointers.killed_infected_knight).unwrap_or(false)
    }

    pub fn decremented_kills_zombie_miner(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.decremented_i32(process, game_manager_finder, "kills_zombie_miner", &game_manager_finder.player_data_pointers.kills_zombie_miner)
    }

    pub fn colosseum_bronze_completed(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "colosseum_bronze_completed", &g.player_data_pointers.colosseum_bronze_completed).unwrap_or(false)
    }

    pub fn colosseum_silver_completed(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "colosseum_silver_completed", &g.player_data_pointers.colosseum_silver_completed).unwrap_or(false)
    }

    pub fn colosseum_gold_completed(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "colosseum_gold_completed", &g.player_data_pointers.colosseum_gold_completed).unwrap_or(false)
    }

    pub fn increased_royal_charm_state(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.increased_i32(process, game_manager_finder, "royal_charm_state", &game_manager_finder.player_data_pointers.royal_charm_state)
    }

    fn been_dead_for_a_tick<const N: usize>(&mut self, prc: &Process, gmf: &GameManagerFinder, key: &'static str, pointer: &UnityPointer<N>) -> bool {
        if gmf.is_game_state_non_continuous(prc) {
            self.map_bool.remove(key);
            return false;
        }
        match self.map_bool.get(key) {
            None | Some(false) => {
                if let Ok(dead_now) = pointer.deref(prc, &gmf.module, &gmf.image) {
                    self.map_bool.insert(key, dead_now);
                }
                false
            }
            Some(true) => true,
        }
    }

    pub fn traitor_lord_been_dead_for_a_tick(&mut self, prc: &Process, gmf: &GameManagerFinder) -> bool {
        self.been_dead_for_a_tick(prc, gmf, "killed_traitor_lord", &gmf.player_data_pointers.killed_traitor_lord)
    }

    pub fn hive_knight_been_dead_for_a_tick(&mut self, prc: &Process, gmf: &GameManagerFinder) -> bool {
        gmf.hive_knight_doesnt_exist(prc).is_some_and(|d| d)
        || self.been_dead_for_a_tick(prc, gmf, "killed_hive_knight", &gmf.player_data_pointers.killed_hive_knight)
    }

    fn kills_on_entry<const N: usize>(&mut self, prc: &Process, gmf: &GameManagerFinder, key: &'static str, pointer: &UnityPointer<N>) -> Option<i32> {
        if gmf.is_game_state_non_continuous(prc) {
            self.map_i32.remove(key);
            return None;
        }
        match self.map_i32.get(key) {
            None => {
                let kills_now = pointer.deref(prc, &gmf.module, &gmf.image).ok()?;
                self.map_i32.insert(key, kills_now);
                Some(kills_now)
            }
            Some(k) => Some(*k)
        }
    }

    /// Produces Some(true) when d of the enemy have been killed in a row,
    /// produces Some(false) when the journal kills have reached 0 without that,
    /// or produces None when neither has happened yet.
    fn kills_decreased_by<const N: usize>(&mut self, prc: &Process, gmf: &GameManagerFinder, key: &'static str, pointer: &UnityPointer<N>, d: i32) -> Option<bool> {
        match self.kills_on_entry(prc, gmf, key, pointer) {
            None => None,
            Some(kills_on_entry) => {
                let kills_now: i32 = pointer.deref(prc, &gmf.module, &gmf.image).ok()?;
                if kills_now + d <= kills_on_entry {
                    Some(true)
                } else if kills_now == 0 {
                    Some(false)
                } else {
                    None
                }
            }
        }
    }

    pub fn aspid_hunter_arena(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Aspid: {0} +3 {3}
        self.kills_decreased_by(prc, gmf, "kills_spitter_on_entry", &gmf.player_data_pointers.kills_spitter, 3)
    }

    pub fn mushroom_brawler_arena(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Shrumal Ogre: {0} +2 {2}
        self.kills_decreased_by(prc, gmf, "kills_mushroom_brawler_on_entry", &gmf.player_data_pointers.kills_mushroom_brawler, 2)
    }

    pub fn bronze1a(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Shielded Fool: {0} +1 {1}
        self.kills_decreased_by(prc, gmf, "kills_col_shield_on_entry", &gmf.player_data_pointers.kills_col_shield, 1)
    }
    pub fn bronze1b(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Shielded Fool: {1} +2 {3}
        self.kills_decreased_by(prc, gmf, "kills_col_shield_on_entry", &gmf.player_data_pointers.kills_col_shield, 3)
    }
    pub fn bronze1c(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Baldur: {0} +2 {2}
        self.kills_decreased_by(prc, gmf, "kills_col_roller_on_entry", &gmf.player_data_pointers.kills_col_roller, 2)
    }
    pub fn bronze2(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Baldur: {2} +5 {7}
        self.kills_decreased_by(prc, gmf, "kills_col_roller_on_entry", &gmf.player_data_pointers.kills_col_roller, 7)
    }
    pub fn bronze3a(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Sturdy Fool: {0} +1 {1}
        self.kills_decreased_by(prc, gmf, "kills_col_miner_on_entry", &gmf.player_data_pointers.kills_col_miner, 1)
    }
    pub fn bronze3b(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Sturdy Fool: {1} +2 {3}
        self.kills_decreased_by(prc, gmf, "kills_col_miner_on_entry", &gmf.player_data_pointers.kills_col_miner, 3)
    }
    pub fn bronze4(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Aspid: {0} +2 {2}
        self.kills_decreased_by(prc, gmf, "kills_spitter_on_entry", &gmf.player_data_pointers.kills_spitter, 2)
    }
    pub fn bronze5(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Aspid: {2} +2 {4}
        self.kills_decreased_by(prc, gmf, "kills_spitter_on_entry", &gmf.player_data_pointers.kills_spitter, 4)
    }
    pub fn bronze6(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Sturdy Fool: {3} +3 {6}
        self.kills_decreased_by(prc, gmf, "kills_col_miner_on_entry", &gmf.player_data_pointers.kills_col_miner, 6)
    }
    pub fn bronze7(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Aspid: {4} +2 {6}
        // Baldur: {7} +2 {9}
        Some(self.kills_decreased_by(prc, gmf, "kills_spitter_on_entry", &gmf.player_data_pointers.kills_spitter, 6)?
             && self.kills_decreased_by(prc, gmf, "kills_col_roller_on_entry", &gmf.player_data_pointers.kills_col_roller, 9)?)
    }
    pub fn bronze8a(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Vengefly: {0} +4 {4}
        self.kills_decreased_by(prc, gmf, "kills_buzzer_on_entry", &gmf.player_data_pointers.kills_buzzer, 4)
    }
    pub fn bronze8b(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Vengefly King: {0} +1 {1}
        self.kills_decreased_by(prc, gmf, "kills_big_buzzer_on_entry", &gmf.player_data_pointers.kills_big_buzzer, 1)
    }
    pub fn bronze9(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Sturdy Fool: {6} +3 {9}
        // Shielded Fool: {3} +2 {5}
        // Aspid: {6} +2 {8}
        // Baldur: {9} +1 {10}
        Some(self.kills_decreased_by(prc, gmf, "kills_spitter_on_entry", &gmf.player_data_pointers.kills_spitter, 8)?
             && self.kills_decreased_by(prc, gmf, "kills_col_roller_on_entry", &gmf.player_data_pointers.kills_col_roller, 10)?
             && self.kills_decreased_by(prc, gmf, "kills_col_miner_on_entry", &gmf.player_data_pointers.kills_col_miner, 9)?
             && self.kills_decreased_by(prc, gmf, "kills_col_shield_on_entry", &gmf.player_data_pointers.kills_col_shield, 5)?)
    }
    pub fn bronze10(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Baldur: {10} +3 {13}
        self.kills_decreased_by(prc, gmf, "kills_col_roller_on_entry", &gmf.player_data_pointers.kills_col_roller, 13)
    }
    pub fn bronze11a(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Infected Gruzzer: {0} +2 {2}
        self.kills_decreased_by(prc, gmf, "kills_bursting_bouncer_on_entry", &gmf.player_data_pointers.kills_bursting_bouncer, 2)
    }
    pub fn bronze11b(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Infected Gruzzer: {2} +3 {5}
        self.kills_decreased_by(prc, gmf, "kills_bursting_bouncer_on_entry", &gmf.player_data_pointers.kills_bursting_bouncer, 5)
    }
    pub fn bronze_end(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Gruz Mom: {0} +2 {2}
        self.kills_decreased_by(prc, gmf, "kills_big_fly_on_entry", &gmf.player_data_pointers.kills_big_fly, 2)
    }
    pub fn silver1(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        self.kills_on_entry(prc, gmf, "kills_col_worm_on_entry", &gmf.player_data_pointers.kills_col_worm);
        self.kills_on_entry(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry);
        // Heavy Fool: {0} +2 {2}
        // Winged Fool: {0} +3 {3}
        Some(self.kills_decreased_by(prc, gmf, "kills_col_worm_on_entry", &gmf.player_data_pointers.kills_col_worm, 2)?
             && self.kills_decreased_by(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry, 3)?)
    }
    pub fn silver2(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Squit: {0} +2 {2}
        self.kills_decreased_by(prc, gmf, "kills_col_mosquito_on_entry", &gmf.player_data_pointers.kills_col_mosquito, 2)
    }
    pub fn silver3(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Squit: {2} +2 {4}
        self.kills_decreased_by(prc, gmf, "kills_col_mosquito_on_entry", &gmf.player_data_pointers.kills_col_mosquito, 4)
    }
    pub fn silver4(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Squit: {4} +1 {5}
        // Winged Fool: {3} +1 {4}
        Some(self.kills_decreased_by(prc, gmf, "kills_col_mosquito_on_entry", &gmf.player_data_pointers.kills_col_mosquito, 5)?
             && self.kills_decreased_by(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry, 4)?)
    }
    pub fn silver5(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        self.kills_on_entry(prc, gmf, "kills_super_spitter_on_entry", &gmf.player_data_pointers.kills_super_spitter);
        // Squit: {5} +2 {7}
        // Infected Gruzzer: {0} +5 {5}
        // Aspid: {0} +2 {2}
        // In Colo 1 and Colo 3, the game uses killsSpitter when you kill a Primal Aspid,
        // but in Colo 2, the game uses killsSuperSpitter when you kill a Primal Aspid.
        Some(self.kills_decreased_by(prc, gmf, "kills_bursting_bouncer_on_entry", &gmf.player_data_pointers.kills_bursting_bouncer, 5)?
             && self.kills_decreased_by(prc, gmf, "kills_col_mosquito_on_entry", &gmf.player_data_pointers.kills_col_mosquito, 7)?
             && self.kills_decreased_by(prc, gmf, "kills_super_spitter_on_entry", &gmf.player_data_pointers.kills_super_spitter, 2)?)
    }
    pub fn silver6(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        self.kills_on_entry(prc, gmf, "kills_ceiling_dropper_on_entry", &gmf.player_data_pointers.kills_ceiling_dropper);
        // Heavy Fool: {2} +1 {3}
        // Belfly: {0} +3 {3}
        Some(self.kills_decreased_by(prc, gmf, "kills_col_worm_on_entry", &gmf.player_data_pointers.kills_col_worm, 3)?
             && self.kills_decreased_by(prc, gmf, "kills_ceiling_dropper_on_entry", &gmf.player_data_pointers.kills_ceiling_dropper, 3)?)
    }
    pub fn silver7(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Belfly: {3} +1 {4}
        self.kills_decreased_by(prc, gmf, "kills_ceiling_dropper_on_entry", &gmf.player_data_pointers.kills_ceiling_dropper, 4)
    }
    pub fn silver8(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Great Hopper: {0} +1 {1}
        // only checking great hopper, not the 8 little hoppers: fine because the game doesn't let you leave one alive
        self.kills_decreased_by(prc, gmf, "kills_giant_hopper_on_entry", &gmf.player_data_pointers.kills_giant_hopper, 1)
    }
    pub fn silver9(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Great Hopper: {1} +1 {2}
        self.kills_decreased_by(prc, gmf, "kills_giant_hopper_on_entry", &gmf.player_data_pointers.kills_giant_hopper, 2)
    }
    pub fn silver10(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Mimic: {0} +1 {1}
        self.kills_decreased_by(prc, gmf, "kills_grub_mimic_on_entry", &gmf.player_data_pointers.kills_grub_mimic, 1)
    }
    pub fn silver11(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Winged Fool: {4} +2 {6}
        // Heavy Fool: {3} +1 {4}
        // Squit: {7} +2 {9}
        // not checking for Shielded Fool kills here: fine because the game doesn't let you leave one alive
        Some(self.kills_decreased_by(prc, gmf, "kills_col_mosquito_on_entry", &gmf.player_data_pointers.kills_col_mosquito, 9)?
             && self.kills_decreased_by(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry, 6)?
             && self.kills_decreased_by(prc, gmf, "kills_col_worm_on_entry", &gmf.player_data_pointers.kills_col_worm, 4)?)
    }
    pub fn silver12(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Heavy Fool: {4} +1 {5}
        // Winged Fool: {6} +1 {7}
        Some(self.kills_decreased_by(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry, 7)?
             && self.kills_decreased_by(prc, gmf, "kills_col_worm_on_entry", &gmf.player_data_pointers.kills_col_worm, 5)?)
    }
    pub fn silver13(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Winged Fool: {7} +1 {8}
        // Squit: {9} +3 {12}
        Some(self.kills_decreased_by(prc, gmf, "kills_col_mosquito_on_entry", &gmf.player_data_pointers.kills_col_mosquito, 12)?
             && self.kills_decreased_by(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry, 8)?)
    }
    pub fn silver14(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Winged Fool: {8} +3 {11}
        // Squit: {12} +2 {14}
        Some(self.kills_decreased_by(prc, gmf, "kills_col_mosquito_on_entry", &gmf.player_data_pointers.kills_col_mosquito, 14)?
             && self.kills_decreased_by(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry, 11)?)
    }
    pub fn silver15(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Obbles: {0} +9 {9}
        self.kills_decreased_by(prc, gmf, "kills_blobble_on_entry", &gmf.player_data_pointers.kills_blobble, 9)
    }
    pub fn silver16(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Obbles: {9} +4 {13}
        self.kills_decreased_by(prc, gmf, "kills_blobble_on_entry", &gmf.player_data_pointers.kills_blobble, 13)
    }
    pub fn silver_end(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Oblobbles: {0} +2 {2}
        self.kills_decreased_by(prc, gmf, "kills_oblobble_on_entry", &gmf.player_data_pointers.kills_oblobble, 2)
    }

    /// Produces Some(true) when 2 Oblobbles have been killed in a row,
    /// produces Some(false) when the journal kills have reached 0 without that,
    /// or produces None when neither has happened yet.
    pub fn killed_oblobbles(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        self.kills_decreased_by(prc, gmf, "kills_oblobble_on_entry", &gmf.player_data_pointers.kills_oblobble, 2)
    }

    pub fn gold1(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        self.kills_on_entry(prc, gmf, "kills_col_worm_on_entry", &gmf.player_data_pointers.kills_col_worm);
        self.kills_on_entry(prc, gmf, "kills_col_miner_on_entry", &gmf.player_data_pointers.kills_col_miner);
        self.kills_on_entry(prc, gmf, "kills_col_mosquito_on_entry", &gmf.player_data_pointers.kills_col_mosquito);
        self.kills_on_entry(prc, gmf, "kills_col_shield_on_entry", &gmf.player_data_pointers.kills_col_shield);
        self.kills_on_entry(prc, gmf, "kills_spitter_on_entry", &gmf.player_data_pointers.kills_spitter);
        self.kills_on_entry(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry);
        self.kills_on_entry(prc, gmf, "kills_col_roller_on_entry", &gmf.player_data_pointers.kills_col_roller);
        // Heavy Fool: {0} +1 {1}
        // Sturdy Fool: {0} +1 {1}
        // Squit: {0} +2 {2}
        // Shielded Fool: {0} +2 {2}
        // Aspid: {0} +1 {1}
        // Winged Fool: {0} +2 {2}
        // Baldurs: {0} +2 {2}
        // not checking for Squit kills here: TODO: make sure the game doesn't let you leave one alive
        // but kills_on_entry is still necessary for Squit here!
        Some(self.kills_decreased_by(prc, gmf, "kills_col_worm_on_entry", &gmf.player_data_pointers.kills_col_worm, 1)?
             && self.kills_decreased_by(prc, gmf, "kills_col_miner_on_entry", &gmf.player_data_pointers.kills_col_miner, 1)?
             && self.kills_decreased_by(prc, gmf, "kills_col_shield_on_entry", &gmf.player_data_pointers.kills_col_shield, 2)?
             && self.kills_decreased_by(prc, gmf, "kills_spitter_on_entry", &gmf.player_data_pointers.kills_spitter, 1)?
             && self.kills_decreased_by(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry, 2)?
             && self.kills_decreased_by(prc, gmf, "kills_col_roller_on_entry", &gmf.player_data_pointers.kills_col_roller, 2)?)
    }
    // Wave 2 splits inconsistently since the enemies are killed by the spikes on the floor automatically
    // Sturdy Fool: {1} +2 {3}
    // Aspid: {1} +1 {2}
    pub fn gold3(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        self.kills_on_entry(prc, gmf, "kills_blobble_on_entry", &gmf.player_data_pointers.kills_blobble);
        self.kills_on_entry(prc, gmf, "kills_angry_buzzer_on_entry", &gmf.player_data_pointers.kills_angry_buzzer);
        // Obble: {0} +3 {3}
        // Winged Fool: {2} +1 {3}
        // Infected Vengefly: {0} +2 {2}
        // not checking for Obble kills here: TODO: make sure the game doesn't let you leave one alive
        // but kills_on_entry is still necessary for Obble here!
        Some(self.kills_decreased_by(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry, 3)?
             && self.kills_decreased_by(prc, gmf, "kills_angry_buzzer_on_entry", &gmf.player_data_pointers.kills_angry_buzzer, 2)?)
    }
    pub fn gold4(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        self.kills_on_entry(prc, gmf, "kills_ceiling_dropper_on_entry", &gmf.player_data_pointers.kills_ceiling_dropper);
        // Heavy Fool: {1} +2 {3}
        // Belflies: {0} +6 {6}
        Some(self.kills_decreased_by(prc, gmf, "kills_col_worm_on_entry", &gmf.player_data_pointers.kills_col_worm, 3)?
             && self.kills_decreased_by(prc, gmf, "kills_ceiling_dropper_on_entry", &gmf.player_data_pointers.kills_ceiling_dropper, 6)?)
    }
    pub fn gold5(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Loodle: {0} +3 {3}
        self.kills_decreased_by(prc, gmf, "kills_col_hopper_on_entry", &gmf.player_data_pointers.kills_col_hopper, 3)
    }
    pub fn gold6(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Loodle: {3} +5 {8}
        self.kills_decreased_by(prc, gmf, "kills_col_hopper_on_entry", &gmf.player_data_pointers.kills_col_hopper, 8)
    }
    pub fn gold7(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Loodle: {8} +3 {11}
        self.kills_decreased_by(prc, gmf, "kills_col_hopper_on_entry", &gmf.player_data_pointers.kills_col_hopper, 11)
    }
    pub fn gold8a(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Squit: {2} +2 {4}
        // Aspid: {2} +3 {5}
        // Winged Fool: {3} +1 {4}
        Some(self.kills_decreased_by(prc, gmf, "kills_col_mosquito_on_entry", &gmf.player_data_pointers.kills_col_mosquito, 4)?
             && self.kills_decreased_by(prc, gmf, "kills_spitter_on_entry", &gmf.player_data_pointers.kills_spitter, 5)?
             && self.kills_decreased_by(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry, 4)?)
    }
    pub fn gold8(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Squit: {4} +2 {6}
        // Winged Fool: {4} +1 {5}
        Some(self.kills_decreased_by(prc, gmf, "kills_col_mosquito_on_entry", &gmf.player_data_pointers.kills_col_mosquito, 6)?
             && self.kills_decreased_by(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry, 5)?)
    }
    pub fn gold9a(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        self.kills_on_entry(prc, gmf, "kills_heavy_mantis_on_entry", &gmf.player_data_pointers.kills_heavy_mantis);
        self.kills_on_entry(prc, gmf, "kills_mantis_heavy_flyer_on_entry", &gmf.player_data_pointers.kills_mantis_heavy_flyer);
        // Shielded Fool: {2} +1 {3}
        // Heavy Fool: {3} +2 {5}
        // Aspid: {5} +1 {6}
        // Mantis Traitor: {0} +2 {2}
        // Mantis Petra: {0} +4 {4}
        Some(self.kills_decreased_by(prc, gmf, "kills_col_shield_on_entry", &gmf.player_data_pointers.kills_col_shield, 3)?
             && self.kills_decreased_by(prc, gmf, "kills_col_worm_on_entry", &gmf.player_data_pointers.kills_col_worm, 5)?
             && self.kills_decreased_by(prc, gmf, "kills_spitter_on_entry", &gmf.player_data_pointers.kills_spitter, 6)?
             && self.kills_decreased_by(prc, gmf, "kills_heavy_mantis_on_entry", &gmf.player_data_pointers.kills_heavy_mantis, 2)?
             && self.kills_decreased_by(prc, gmf, "kills_mantis_heavy_flyer_on_entry", &gmf.player_data_pointers.kills_mantis_heavy_flyer, 4)?)
    }
    pub fn gold9b(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        self.kills_on_entry(prc, gmf, "kills_mage_on_entry", &gmf.player_data_pointers.kills_mage);
        // Soul Twister: {0} +2 {2}
        // Soul Warrior: {0} +1 {1}
        // not checking for Soul Twister kills here: fine because the game doesn't let you leave one alive
        // but kills_on_entry is still necessary for Soul Twister here!
        self.kills_decreased_by(prc, gmf, "kills_mage_knight_on_entry", &gmf.player_data_pointers.kills_mage_knight, 1)
    }
    pub fn gold10(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        self.kills_on_entry(prc, gmf, "kills_electric_mage_on_entry", &gmf.player_data_pointers.kills_electric_mage);
        self.kills_on_entry(prc, gmf, "kills_mage_on_entry", &gmf.player_data_pointers.kills_mage);
        // Volt Twister: {0} +3 {3}
        // Soul Twister: {2} +2 {4}
        Some(self.kills_decreased_by(prc, gmf, "kills_electric_mage_on_entry", &gmf.player_data_pointers.kills_electric_mage, 3)?
             && self.kills_decreased_by(prc, gmf, "kills_mage_on_entry", &gmf.player_data_pointers.kills_mage, 4)?)
    }
    pub fn gold11(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Soul Warrior: {1} +1 {2}
        // Soul Twister: {4} +1 {5}
        Some(self.kills_decreased_by(prc, gmf, "kills_mage_knight_on_entry", &gmf.player_data_pointers.kills_mage_knight, 2)?
             && self.kills_decreased_by(prc, gmf, "kills_mage_on_entry", &gmf.player_data_pointers.kills_mage, 5)?)
    }
    pub fn gold12a(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        self.kills_on_entry(prc, gmf, "kills_lesser_mawlek_on_entry", &gmf.player_data_pointers.kills_lesser_mawlek);
        // Winged Fool: {5} +2 {7}
        // Sturdy Fool: {3} +1 {4}
        // Lesser Mawlek: {0} +4 {4}
        Some(self.kills_decreased_by(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry, 7)?
             && self.kills_decreased_by(prc, gmf, "kills_col_miner_on_entry", &gmf.player_data_pointers.kills_col_miner, 4)?
             && self.kills_decreased_by(prc, gmf, "kills_lesser_mawlek_on_entry", &gmf.player_data_pointers.kills_lesser_mawlek, 4)?)
    }
    pub fn gold12b(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Brooding Mawlek: {0} +1 {1}
        self.kills_decreased_by(prc, gmf, "kills_mawlek_on_entry", &gmf.player_data_pointers.kills_mawlek, 1)
    }
    // Wave 13 doesn't really exist, it's just vertical Garpedes so there's nothing to Split on
    pub fn gold14a(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Squit: {6} +4 {10}
        // Aspid: {6} +1 {7}
        // Mantis Petra: {4} +1 {5}
        // Winged Fool: {7} +1 {8}
        // not checking for Winged Fool kills here: fine because the game doesn't let you leave one alive
        Some(self.kills_decreased_by(prc, gmf, "kills_col_mosquito_on_entry", &gmf.player_data_pointers.kills_col_mosquito, 10)?
             && self.kills_decreased_by(prc, gmf, "kills_spitter_on_entry", &gmf.player_data_pointers.kills_spitter, 7)?
             && self.kills_decreased_by(prc, gmf, "kills_mantis_heavy_flyer_on_entry", &gmf.player_data_pointers.kills_mantis_heavy_flyer, 5)?)
    }
    pub fn gold14b(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Winged Fool: {8} +2 {10}
        // Obble: {3} +4 {7}
        Some(self.kills_decreased_by(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry, 10)?
             && self.kills_decreased_by(prc, gmf, "kills_blobble_on_entry", &gmf.player_data_pointers.kills_blobble, 7)?)
    }
    pub fn gold15(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Squit: {10} +2 {12}
        self.kills_decreased_by(prc, gmf, "kills_col_mosquito_on_entry", &gmf.player_data_pointers.kills_col_mosquito, 12)
    }
    pub fn gold16(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Loodle: {11} +14 {25}
        self.kills_decreased_by(prc, gmf, "kills_col_hopper_on_entry", &gmf.player_data_pointers.kills_col_hopper, 25)
    }
    pub fn gold17a(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Heavy Fool: {5} +1 {6}
        // Sturdy Fool: {4} +1 {5}
        // Shielded Fool: {3} +1 {4}
        // Mantis Petra: {5} +1 {6}
        // Mantis Traitor: {2} +1 {3}
        // Winged Fool: {10} +1 {11}
        Some(self.kills_decreased_by(prc, gmf, "kills_col_worm_on_entry", &gmf.player_data_pointers.kills_col_worm, 6)?
             && self.kills_decreased_by(prc, gmf, "kills_col_miner_on_entry", &gmf.player_data_pointers.kills_col_miner, 5)?
             && self.kills_decreased_by(prc, gmf, "kills_col_shield_on_entry", &gmf.player_data_pointers.kills_col_shield, 4)?
             && self.kills_decreased_by(prc, gmf, "kills_mantis_heavy_flyer_on_entry", &gmf.player_data_pointers.kills_mantis_heavy_flyer, 6)?
             && self.kills_decreased_by(prc, gmf, "kills_heavy_mantis_on_entry", &gmf.player_data_pointers.kills_heavy_mantis, 3)?
             && self.kills_decreased_by(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry, 11)?)
    }
    pub fn gold17b(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Heavy Fool: {6} +1 {7}
        // Shielded Fool: {4} +1 {5}
        // Soul Twister: {5} +1 {6}
        // Volt Twister: {3} +1 {4}
        Some(self.kills_decreased_by(prc, gmf, "kills_col_worm_on_entry", &gmf.player_data_pointers.kills_col_worm, 7)?
             && self.kills_decreased_by(prc, gmf, "kills_col_shield_on_entry", &gmf.player_data_pointers.kills_col_shield, 5)?
             && self.kills_decreased_by(prc, gmf, "kills_mage_on_entry", &gmf.player_data_pointers.kills_mage, 6)?
             && self.kills_decreased_by(prc, gmf, "kills_electric_mage_on_entry", &gmf.player_data_pointers.kills_electric_mage, 4)?)
    }
    pub fn gold17c(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // Baldur: {2} +2 {4}
        // Squit: {12} +2 {14}
        // Heavy Fool: {7} +1 {8}
        // Shielded Fool: {5} +1 {6}
        // Sturdy Fool: {5} +1 {6}
        // Winged Fool: {11} +1 {12}
        // not checking for Squit kills here: TODO: make sure the game doesn't let you leave one alive
        Some(self.kills_decreased_by(prc, gmf, "kills_col_roller_on_entry", &gmf.player_data_pointers.kills_col_roller, 4)?
             && self.kills_decreased_by(prc, gmf, "kills_col_worm_on_entry", &gmf.player_data_pointers.kills_col_worm, 8)?
             && self.kills_decreased_by(prc, gmf, "kills_col_shield_on_entry", &gmf.player_data_pointers.kills_col_shield, 6)?
             && self.kills_decreased_by(prc, gmf, "kills_col_miner_on_entry", &gmf.player_data_pointers.kills_col_miner, 6)?
             && self.kills_decreased_by(prc, gmf, "kills_col_flying_sentry_on_entry", &gmf.player_data_pointers.kills_col_flying_sentry, 12)?)
    }
    pub fn gold_end(&mut self, prc: &Process, gmf: &GameManagerFinder) -> Option<bool> {
        // God Tamer: {0} +1 {1}
        self.kills_decreased_by(prc, gmf, "kills_lobster_lancer_on_entry", &gmf.player_data_pointers.kills_lobster_lancer, 1)
    }
}

// --------------------------------------------------------

pub async fn wait_attach_hollow_knight<G: StoreGui>(gui: &mut G) -> Process {
    retry(|| {
        gui.loop_load_update_store();
        HOLLOW_KNIGHT_NAMES.into_iter().find_map(Process::attach)
    }).await
}

fn process_pointer_size(process: &Process) -> Option<PointerSize> {
    let path = process.get_path().ok()?;
    let bytes = file::file_read_all_bytes(path).ok()?;
    if bytes.starts_with(&[0x4D, 0x5A]) {
        // PE
        let mono_addr = ["mono.dll", "mono-2.0-bdwgc.dll"].into_iter().find_map(|mono_name| {
            process.get_module_address(mono_name).ok()
        })?;
        pe::MachineType::read(process, mono_addr)?.pointer_size()
    } else if bytes.starts_with(&[0x7F, 0x45, 0x4C, 0x46]) {
        // ELF
        let mono_addr = ["libmono.so", "libmonobdwgc-2.0.so"].into_iter().find_map(|mono_name| {
            process.get_module_address(mono_name).ok()
        })?;
        elf::pointer_size(process, mono_addr)
    } else if bytes.starts_with(&[0xFE, 0xED, 0xFA, 0xCE])
            | bytes.starts_with(&[0xCE, 0xFA, 0xED, 0xFE]) {
        // MachO 32-bit
        Some(PointerSize::Bit32)
    } else if bytes.starts_with(&[0xFE, 0xED, 0xFA, 0xCF])
            | bytes.starts_with(&[0xCF, 0xFA, 0xED, 0xFE]) {
        // MachO 64-bit
        Some(PointerSize::Bit64)
    } else {
        None
    }
}

fn read_string_object<const N: usize>(process: &Process, offsets: &StringListOffsets, a: Address) -> Option<String> {
    let n: u32 = process.read(a + offsets.string_len).ok()?;
    if !(n < 2048) { return None; }
    let w: ArrayWString<N> = process.read(a + offsets.string_contents).ok()?;
    if !(w.len() == min(n as usize, N)) { return None; }
    String::from_utf16(&w.to_vec()).ok()
}

fn read_string_list_object<const SN: usize>(process: &Process, offsets: &StringListOffsets, a: Address) -> Option<Vec<String>> {
    let array_ptr: Address = process.read_pointer(a + offsets.list_array, offsets.pointer_size).ok()?;
    let vn: u32 = process.read(array_ptr + offsets.array_len).ok()?;

    let mut v = Vec::with_capacity(vn as usize);
    for i in 0..(vn as u64) {
        let item_offset = offsets.array_contents + (offsets.pointer_size as u64) * i;
        let item_ptr: Address = process.read_pointer(array_ptr + item_offset, offsets.pointer_size).ok()?;
        if item_ptr.is_null() {
            continue;
        }
        let s = read_string_object::<SN>(process, offsets, item_ptr)?;
        v.push(s);
    }
    Some(v)
}

// --------------------------------------------------------
// --------------------------------------------------------

pub fn is_menu(s: &str) -> bool {
    s == MENU_TITLE || s == QUIT_TO_MENU || s == PERMA_DEATH
}

pub fn is_play_scene(s: &str) -> bool {
    !NON_PLAY_SCENES.contains(&s) && !BAD_SCENE_NAMES.contains(&s)
}

pub fn is_debug_save_state_scene(s: &str) -> bool {
    DEBUG_SAVE_STATE_SCENE_NAMES.contains(&s)
}

pub fn starts_with_any(full: &str, prefixes: &[&str]) -> bool {
    prefixes.into_iter().any(|prefix| full.starts_with(prefix))
}

// --------------------------------------------------------
