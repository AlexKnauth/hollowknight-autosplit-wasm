
use core::cell::OnceCell;
use std::cmp::min;
use std::mem;
use std::collections::BTreeMap;
use asr::future::{next_tick, retry};
use asr::watcher::Pair;
use asr::{Process, Address64};
use asr::game_engine::unity::mono::{self, UnityPointer};
use asr::string::ArrayWString;
use ugly_widget::store::StoreGui;

#[cfg(debug_assertions)]
use std::string::String;

// --------------------------------------------------------

const HOLLOW_KNIGHT_NAMES: [&str; 3] = [
    "hollow_knight.exe", // Windows
    "Hollow Knight", // Mac
    "hollow_knight", // Mac
];

pub const SCENE_PATH_SIZE: usize = 64;

const STRING_LEN_OFFSET: u64 = 0x10;
const STRING_CONTENTS_OFFSET: u64 = 0x14;

const PRE_MENU_INTRO: &str = "Pre_Menu_Intro";
pub const MENU_TITLE: &str = "Menu_Title";
pub const QUIT_TO_MENU: &str = "Quit_To_Menu";
pub const INTRO_CUTSCENE: &str = "Intro_Cutscene";
pub const OPENING_SEQUENCE: &str = "Opening_Sequence";
pub const GG_ENTRANCE_CUTSCENE: &str = "GG_Entrance_Cutscene";
pub const OPENING_SCENES: [&str; 2] = [
    INTRO_CUTSCENE,
    OPENING_SEQUENCE,
];

const NON_PLAY_SCENES: [&str; 16] = [
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

pub const FUNGAL_WASTES_ENTRY_SCENES: &[&str] = &[
    "Fungus2_06", // Room outside Leg Eater
    "Fungus2_03", // From Queens' Station
    "Fungus2_23", // Bretta from Waterways
    "Fungus2_20", // Spore Shroom room, from QG (this one's unlikely to come up)
];

pub const CRYSTAL_PEAK_ENTRY_SCENES: &[&str] = &[
    "Mines_02",
    "Mines_10",
];

pub const QUEENS_GARDENS_ENTRY_SCENES: &[&str] = &[
    "Fungus3_34",
    "Deepnest_43",
];

pub const DEEPNEST_ENTRY_SCENES: &[&str] = &[
    "Fungus2_25", // Room after Mantis Lords
    "Deepnest_42", // Room outside Mask Maker
    "Abyss_03b", // Deepnest Tram
    "Deepnest_01b", // Near Spore Shroom
];

// --------------------------------------------------------

// const VERSION_VEC_MAJOR: usize = 0;
const VERSION_VEC_MINOR: usize = 1;
// const VERSION_VEC_BUILD: usize = 2;
// const VERSION_VEC_REVISION: usize = 3;

pub const GAME_STATE_INACTIVE: i32 = 0;
pub const GAME_STATE_MAIN_MENU: i32 = 1;
pub const GAME_STATE_LOADING: i32 = 2;
pub const GAME_STATE_ENTERING_LEVEL: i32 = 3;
pub const GAME_STATE_PLAYING: i32 = 4;
pub const GAME_STATE_EXITING_LEVEL: i32 = 6;
pub const GAME_STATE_CUTSCENE: i32 = 7;

pub const NON_MENU_GAME_STATES: [i32; 2] = [
    GAME_STATE_PLAYING,
    GAME_STATE_CUTSCENE,
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
    hero_transition_state: UnityPointer<3>,
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
            hero_transition_state: UnityPointer::new("GameManager", 0, &["_instance", "<hero_ctrl>k__BackingField", "transitionState"]),
        }
    }
}

// --------------------------------------------------------

struct PlayerDataPointers {
    version: UnityPointer<4>,
    health: UnityPointer<3>,
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
    /// Magic Power Reserve Max: amount of soul that can be held by soul vessels, 33 each
    mp_reserve_max: UnityPointer<3>,
    vessel_fragments: UnityPointer<3>,
    // Dreamers
    mask_broken_lurien: UnityPointer<3>,
    mask_broken_monomon: UnityPointer<3>,
    mask_broken_hegemol: UnityPointer<3>,
    guardians_defeated: UnityPointer<3>,
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
    sold_trinket2: UnityPointer<3>,
    sold_trinket4: UnityPointer<3>,
    rancid_eggs: UnityPointer<3>,
    // Charm Notches
    notch_shroom_ogres: UnityPointer<3>,
    salubra_notch1: UnityPointer<3>,
    salubra_notch2: UnityPointer<3>,
    salubra_notch3: UnityPointer<3>,
    salubra_notch4: UnityPointer<3>,
    notch_fog_canyon: UnityPointer<3>,
    got_grimm_notch: UnityPointer<3>,
    charm_slots: UnityPointer<3>,
    // Charms
    got_charm_1: UnityPointer<3>,
    got_charm_2: UnityPointer<3>,
    got_charm_3: UnityPointer<3>,
    got_charm_4: UnityPointer<3>,
    got_charm_5: UnityPointer<3>,
    got_charm_6: UnityPointer<3>,
    got_charm_7: UnityPointer<3>,
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
    got_charm_18: UnityPointer<3>,
    got_charm_19: UnityPointer<3>,
    got_charm_20: UnityPointer<3>,
    got_charm_21: UnityPointer<3>,
    got_charm_22: UnityPointer<3>,
    got_charm_26: UnityPointer<3>,
    got_charm_27: UnityPointer<3>,
    got_charm_28: UnityPointer<3>,
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
    killed_grimm: UnityPointer<3>,
    killed_nightmare_grimm: UnityPointer<3>,
    killed_grey_prince: UnityPointer<3>,
    grey_prince_orbs_collected: UnityPointer<3>,
    killed_mender_bug: UnityPointer<3>,
    killed_mawlek: UnityPointer<3>,
    // Gruz Mother
    killed_big_fly: UnityPointer<3>,
    sly_rescued: UnityPointer<3>,
    killed_false_knight: UnityPointer<3>,
    false_knight_dream_defeated: UnityPointer<3>,
    false_knight_orbs_collected: UnityPointer<3>,
    unchained_hollow_knight: UnityPointer<3>,
    killed_hollow_knight: UnityPointer<3>,
    killed_final_boss: UnityPointer<3>,
    killed_moss_knight: UnityPointer<3>,
    zote_rescued_buzzer: UnityPointer<3>,
    killed_hornet: UnityPointer<3>,
    /// killedLazyFlyer: Aluba
    killed_lazy_flyer: UnityPointer<3>,
    killed_ghost_no_eyes: UnityPointer<3>,
    no_eyes_defeated: UnityPointer<3>,
    mega_moss_charger_defeated: UnityPointer<3>,
    killed_ghost_hu: UnityPointer<3>,
    elder_hu_defeated: UnityPointer<3>,
    bretta_rescued: UnityPointer<3>,
    defeated_mantis_lords: UnityPointer<3>,
    // Gorb
    killed_ghost_aladar: UnityPointer<3>,
    aladar_slug_defeated: UnityPointer<3>,
    nightmare_lantern_lit: UnityPointer<3>,
    destroyed_nightmare_lantern: UnityPointer<3>,
    killed_ghost_xero: UnityPointer<3>,
    xero_defeated: UnityPointer<3>,
    opened_city_gate: UnityPointer<3>,
    killed_gorgeous_husk: UnityPointer<3>,
    // Lemm
    met_relic_dealer_shop: UnityPointer<3>,
    // Soul Master
    mage_lord_encountered: UnityPointer<3>,
    mage_lord_encountered_2: UnityPointer<3>,
    killed_mage_lord: UnityPointer<3>,
    mage_lord_dream_defeated: UnityPointer<3>,
    mage_lord_orbs_collected: UnityPointer<3>,
    watcher_chandelier: UnityPointer<3>,
    killed_black_knight: UnityPointer<3>,
    collector_defeated: UnityPointer<3>,
    kills_zombie_miner: UnityPointer<3>,
    // Crystal Guardian
    defeated_mega_beam_miner: UnityPointer<3>,
    kills_mega_beam_miner: UnityPointer<3>,
    mine_lift_opened: UnityPointer<3>,
    killed_dung_defender: UnityPointer<3>,
    killed_white_defender: UnityPointer<3>,
    white_defender_orbs_collected: UnityPointer<3>,
    killed_fluke_mother: UnityPointer<3>,
    // Broken Vessel
    killed_infected_knight: UnityPointer<3>,
    infected_knight_dream_defeated: UnityPointer<3>,
    infected_knight_orbs_collected: UnityPointer<3>,
    killed_hive_knight: UnityPointer<3>,
    killed_giant_hopper: UnityPointer<3>,
    hornet_outskirts_defeated: UnityPointer<3>,
    killed_ghost_markoth: UnityPointer<3>,
    markoth_defeated: UnityPointer<3>,
    colosseum_bronze_opened: UnityPointer<3>,
    killed_zote: UnityPointer<3>,
    colosseum_bronze_completed: UnityPointer<3>,
    colosseum_silver_opened: UnityPointer<3>,
    colosseum_silver_completed: UnityPointer<3>,
    colosseum_gold_opened: UnityPointer<3>,
    // God Tamer
    killed_lobster_lancer: UnityPointer<3>,
    colosseum_gold_completed: UnityPointer<3>,
    // Uumuu
    encountered_mega_jelly: UnityPointer<3>,
    killed_mega_jellyfish: UnityPointer<3>,
    killed_ghost_marmu: UnityPointer<3>,
    mum_caterpillar_defeated: UnityPointer<3>,
    killed_traitor_lord: UnityPointer<3>,
    zote_rescued_deepnest: UnityPointer<3>,
    // Nosk
    killed_mimic_spider: UnityPointer<3>,
    killed_ghost_galien: UnityPointer<3>,
    galien_defeated: UnityPointer<3>,
    spider_capture: UnityPointer<3>,
    has_godfinder: UnityPointer<3>,
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
            health: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "health"]),
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
            mp_reserve_max: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "MPReserveMax"]),
            vessel_fragments: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "vesselFragments"]),
            // Dreamers
            mask_broken_lurien: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "maskBrokenLurien"]),
            mask_broken_monomon: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "maskBrokenMonomon"]),
            mask_broken_hegemol: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "maskBrokenHegemol"]),
            guardians_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "guardiansDefeated"]),
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
            sold_trinket2: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "soldTrinket2"]),
            sold_trinket4: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "soldTrinket4"]),
            rancid_eggs: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "rancidEggs"]),
            // Charm Notches
            notch_shroom_ogres: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "notchShroomOgres"]),
            salubra_notch1: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "salubraNotch1"]),
            salubra_notch2: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "salubraNotch2"]),
            salubra_notch3: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "salubraNotch3"]),
            salubra_notch4: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "salubraNotch4"]),
            notch_fog_canyon: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "notchFogCanyon"]),
            got_grimm_notch: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotGrimmNotch"]),
            charm_slots: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "charmSlots"]),
            // Charms
            got_charm_1: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_1"]),
            got_charm_2: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_2"]),
            got_charm_3: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_3"]),
            got_charm_4: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_4"]),
            got_charm_5: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_5"]),
            got_charm_6: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_6"]),
            got_charm_7: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_7"]),
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
            got_charm_18: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_18"]),
            got_charm_19: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_19"]),
            got_charm_20: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_20"]),
            got_charm_21: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_21"]),
            got_charm_22: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_22"]),
            got_charm_26: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_26"]),
            got_charm_27: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_27"]),
            got_charm_28: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "gotCharm_28"]),
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
            killed_grimm: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGrimm"]),
            killed_nightmare_grimm: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedNightmareGrimm"]),
            killed_grey_prince: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGreyPrince"]),
            grey_prince_orbs_collected: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "greyPrinceOrbsCollected"]),
            killed_mender_bug: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedMenderBug"]),
            killed_mawlek: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedMawlek"]),
            killed_big_fly: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedBigFly"]),
            sly_rescued: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "slyRescued"]),
            killed_false_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedFalseKnight"]),
            false_knight_dream_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "falseKnightDreamDefeated"]),
            false_knight_orbs_collected: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "falseKnightOrbsCollected"]),
            unchained_hollow_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "unchainedHollowKnight"]),
            killed_hollow_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedHollowKnight"]),
            killed_final_boss: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedFinalBoss"]),
            killed_moss_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedMossKnight"]),
            zote_rescued_buzzer: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "zoteRescuedBuzzer"]),
            killed_hornet: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedHornet"]),
            killed_lazy_flyer: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedLazyFlyer"]),
            killed_ghost_no_eyes: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGhostNoEyes"]),
            no_eyes_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "noEyesDefeated"]),
            mega_moss_charger_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "megaMossChargerDefeated"]),
            killed_ghost_hu: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGhostHu"]),
            elder_hu_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "elderHuDefeated"]),
            bretta_rescued: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "brettaRescued"]),
            defeated_mantis_lords: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedMantisLords"]),
            // Gorb
            killed_ghost_aladar: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGhostAladar"]),
            aladar_slug_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "aladarSlugDefeated"]),
            nightmare_lantern_lit: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "nightmareLanternLit"]),
            destroyed_nightmare_lantern: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "destroyedNightmareLantern"]),
            killed_ghost_xero: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGhostXero"]),
            xero_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "xeroDefeated"]),
            opened_city_gate: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "openedCityGate"]),
            killed_gorgeous_husk: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGorgeousHusk"]),
            met_relic_dealer_shop: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "metRelicDealerShop"]),
            mage_lord_encountered: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mageLordEncountered"]),
            mage_lord_encountered_2: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mageLordEncountered_2"]),
            killed_mage_lord: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedMageLord"]),
            mage_lord_dream_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mageLordDreamDefeated"]),
            mage_lord_orbs_collected: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mageLordOrbsCollected"]),
            watcher_chandelier: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "watcherChandelier"]),
            killed_black_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedBlackKnight"]),
            collector_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "collectorDefeated"]),
            kills_zombie_miner: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsZombieMiner"]),
            defeated_mega_beam_miner: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedMegaBeamMiner"]),
            kills_mega_beam_miner: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killsMegaBeamMiner"]),
            mine_lift_opened: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mineLiftOpened"]),
            killed_dung_defender: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedDungDefender"]),
            killed_white_defender: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedWhiteDefender"]),
            white_defender_orbs_collected: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "whiteDefenderOrbsCollected"]),
            killed_fluke_mother: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedFlukeMother"]),
            killed_infected_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedInfectedKnight"]),
            infected_knight_dream_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "infectedKnightDreamDefeated"]),
            infected_knight_orbs_collected: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "infectedKnightOrbsCollected"]),
            killed_hive_knight: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedHiveKnight"]),
            killed_giant_hopper: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGiantHopper"]),
            hornet_outskirts_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hornetOutskirtsDefeated"]),
            killed_ghost_markoth: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGhostMarkoth"]),
            markoth_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "markothDefeated"]),
            colosseum_bronze_opened: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "colosseumBronzeOpened"]),
            killed_zote: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedZote"]),
            colosseum_bronze_completed: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "colosseumBronzeCompleted"]),
            colosseum_silver_opened: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "colosseumSilverOpened"]),
            colosseum_silver_completed: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "colosseumSilverCompleted"]),
            colosseum_gold_opened: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "colosseumGoldOpened"]),
            killed_lobster_lancer: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedLobsterLancer"]),
            colosseum_gold_completed: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "colosseumGoldCompleted"]),
            encountered_mega_jelly: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "encounteredMegaJelly"]),
            killed_mega_jellyfish: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedMegaJellyfish"]),
            killed_ghost_marmu: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGhostMarmu"]),
            mum_caterpillar_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "mumCaterpillarDefeated"]),
            killed_traitor_lord: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedTraitorLord"]),
            zote_rescued_deepnest: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "zoteRescuedDeepnest"]),
            killed_mimic_spider: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedMimicSpider"]),
            killed_ghost_galien: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedGhostGalien"]),
            galien_defeated: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "galienDefeated"]),
            spider_capture: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "spiderCapture"]),
            has_godfinder: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasGodfinder"]),
            killed_nail_bros: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedNailBros"]),
            killed_paintmaster: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedPaintmaster"]),
            killed_nailsage: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedNailsage"]),
            killed_hollow_knight_prime: UnityPointer::new("GameManager", 0, &["_instance", "playerData", "killedHollowKnightPrime"]),
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
                    return GameManagerFinder {
                        module,
                        image,
                        pointers: GameManagerPointers::new(),
                        player_data_pointers: PlayerDataPointers::new(),
                        ui_state_offset: OnceCell::new(),
                    };
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

    pub fn get_scene_name(&self, process: &Process) -> Option<String> {
        let s = self.pointers.scene_name.deref(process, &self.module, &self.image).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, s)
    }

    pub fn get_next_scene_name(&self, process: &Process) -> Option<String> {
        let s = self.pointers.next_scene_name.deref(process, &self.module, &self.image).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, s)
    }

    pub fn get_game_state(&self, process: &Process) -> Option<i32> {
        self.pointers.game_state.deref(process, &self.module, &self.image).ok()
    }

    fn is_game_state_non_menu(&self, process: &Process) -> bool {
        self.get_game_state(process).is_some_and(|gs| NON_MENU_GAME_STATES.contains(&gs))
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

    pub fn hero_recoiling(&self, process: &Process) -> Option<bool> {
        self.pointers.hero_recoiling.deref(process, &self.module, &self.image).ok()
    }

    pub fn get_version_string(&self, process: &Process) -> Option<String> {
        let s = [&self.pointers.version_number, &self.player_data_pointers.version].into_iter().find_map(|ptr| {
            ptr.deref(process, &self.module, &self.image).ok()
        })?;
        read_string_object::<SCENE_PATH_SIZE>(process, s)
    }

    pub fn get_version_vec(&self, process: &Process) -> Option<Vec<i32>> {
        Some(self.get_version_string(process)?.split('.').map(|s| {
            s.parse().unwrap_or(0)
        }).collect())
    }

    pub fn get_health(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.health.deref(process, &self.module, &self.image).ok()
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

    // Keys

    pub fn has_city_key(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_city_key.deref(process, &self.module, &self.image).ok()
    }

    pub fn has_lantern(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.has_lantern.deref(process, &self.module, &self.image).ok()
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

    pub fn sold_trinket2(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.sold_trinket2.deref(process, &self.module, &self.image).ok()
    }
    pub fn sold_trinket4(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.sold_trinket4.deref(process, &self.module, &self.image).ok()
    }

    #[allow(unused)]
    pub fn rancid_eggs(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.rancid_eggs.deref(process, &self.module, &self.image).ok()
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

    pub fn got_charm_6(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_6.deref(process, &self.module, &self.image).ok()
    }

    pub fn got_charm_7(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.got_charm_7.deref(process, &self.module, &self.image).ok()
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

    pub fn unchained_hollow_knight(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.unchained_hollow_knight.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_hollow_knight(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_hollow_knight.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_final_boss(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_final_boss.deref(process, &self.module, &self.image).ok()
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

    pub fn killed_ghost_no_eyes(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_ghost_no_eyes.deref(process, &self.module, &self.image).ok()
    }
    pub fn no_eyes_defeated(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.no_eyes_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn mega_moss_charger_defeated(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.mega_moss_charger_defeated.deref(process, &self.module, &self.image).ok()
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

    pub fn killed_ghost_xero(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_ghost_xero.deref(process, &self.module, &self.image).ok()
    }
    pub fn xero_defeated(&self, process: &Process) -> Option<i32> {
        self.player_data_pointers.xero_defeated.deref(process, &self.module, &self.image).ok()
    }

    pub fn opened_city_gate(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.opened_city_gate.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_gorgeous_husk(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_gorgeous_husk.deref(process, &self.module, &self.image).ok()
    }

    // Lemm
    pub fn met_relic_dealer_shop(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.met_relic_dealer_shop.deref(process, &self.module, &self.image).ok()
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

    pub fn watcher_chandelier(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.watcher_chandelier.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_black_knight(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_black_knight.deref(process, &self.module, &self.image).ok()
    }

    pub fn collector_defeated(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.collector_defeated.deref(process, &self.module, &self.image).ok()
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

    pub fn killed_dung_defender(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_dung_defender.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_white_defender(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_white_defender.deref(process, &self.module, &self.image).ok()
    }

    pub fn white_defender_orbs_collected(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.white_defender_orbs_collected.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_fluke_mother(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_fluke_mother.deref(process, &self.module, &self.image).ok()
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

    pub fn killed_hive_knight(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_hive_knight.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_giant_hopper(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_giant_hopper.deref(process, &self.module, &self.image).ok()
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

    pub fn colosseum_bronze_opened(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.colosseum_bronze_opened.deref(process, &self.module, &self.image).ok()
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

    // Uumuu
    pub fn encountered_mega_jelly(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.encountered_mega_jelly.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_mega_jellyfish(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_mega_jellyfish.deref(process, &self.module, &self.image).ok()
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

    pub fn zote_rescued_deepnest(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.zote_rescued_deepnest.deref(process, &self.module, &self.image).ok()
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

    pub fn killed_nail_bros(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_nail_bros.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_paintmaster(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_paintmaster.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_nailsage(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_nailsage.deref(process, &self.module, &self.image).ok()
    }

    pub fn killed_hollow_knight_prime(&self, process: &Process) -> Option<bool> {
        self.player_data_pointers.killed_hollow_knight_prime.deref(process, &self.module, &self.image).ok()
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

    pub fn got_charm_31(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "got_charm_31", &g.player_data_pointers.got_charm_31).unwrap_or(false)
    }

    pub fn got_shade_charm(&mut self, p: &Process, g: &GameManagerFinder) -> bool {
        self.get_bool(p, g, "got_shade_charm", &g.player_data_pointers.got_shade_charm).unwrap_or(false)
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

    pub fn incremented_charm_slots(&mut self, process: &Process, game_manager_finder: &GameManagerFinder) -> bool {
        self.incremented_i32(process, game_manager_finder, "charm_slots", &game_manager_finder.player_data_pointers.charm_slots)
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
}

// --------------------------------------------------------

pub async fn wait_attach_hollow_knight<G: StoreGui>(gui: &mut G) -> Process {
    retry(|| {
        gui.loop_load_update_store();
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

pub fn starts_with_any(full: &str, prefixes: &[&str]) -> bool {
    prefixes.into_iter().any(|prefix| full.starts_with(prefix))
}

// --------------------------------------------------------
