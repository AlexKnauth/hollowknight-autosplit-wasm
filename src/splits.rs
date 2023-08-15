use asr::Process;
use asr::watcher::Pair;
use serde::{Deserialize, Serialize};

use super::hollow_knight_memory::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub enum Split {
    // Start and End
    StartNewGame,
    StartAnyGame,
    EndingSplit,

    // Dreamers
    Lurien,
    Monomon,
    Hegemol,

    // Spell Levels
    VengefulSpirit,
    ShadeSoul,

    // Movement Abilities
    MothwingCloak,
    ShadeCloak,
    MantisClaw,
    MonarchWings,
    CrystalHeart,
    IsmasTear,

    // Dream Nail Levels
    DreamNail,
    DreamGate,
    DreamNail2,

    // Other Items
    LumaflyLantern,
    OnObtainSimpleKey,
    SlyKey,
    ElegantKey,

    // Dirtmouth
    SlyShopExit,
    // Crossroads
    AncestralMound,
    SalubraExit,
    EnterHollowKnight,
    // Greenpath
    EnterGreenpath,
    MenuCloak,
    // Fungal
    MenuMantisJournal,
    // Resting Grounds
    DreamNailExit,
    // City
    TransGorgeousHusk,
    MenuStoreroomsSimpleKey,
    MenuShadeSoul,
    EnterBlackKnight,
    BlackKnightTrans,
    // Peak
    MenuSlyKey,
    // Waterways
    DungDefenderExit,
    MenuIsmasTear,
    // Basin
    Abyss19from18,
    MenuWings,
    // Fog Canyon
    TeachersArchive,
    // Queen's Gardens
    QueensGardensEntry,
}

pub fn transition_splits(s: &Split, p: &Pair<&str>) -> bool {
    match s {
        // Start and End
        Split::StartNewGame => {
            (p.old == OPENING_SEQUENCE && p.current == "Tutorial_01") || (is_menu(p.old) && p.current == GG_ENTRANCE_CUTSCENE)
        },
        Split::StartAnyGame => {
            (is_menu(p.old) || p.old == OPENING_SEQUENCE) && (is_play_scene(p.current) || p.current == GG_ENTRANCE_CUTSCENE)
        }
        Split::EndingSplit => p.current.starts_with("Cinematic_Ending"),
        
        // Dreamers
        Split::Lurien => p.old == "Dream_Guardian_Lurien" && p.current == "Cutscene_Boss_Door",
        Split::Monomon => p.old == "Dream_Guardian_Monomon" && p.current == "Cutscene_Boss_Door",
        Split::Hegemol => p.old == "Dream_Guardian_Hegemol" && p.current == "Cutscene_Boss_Door",

        // Dirtmouth
        Split::SlyShopExit => p.old == "Room_shop" && p.current != p.old,
        // Crossroads
        Split::AncestralMound => p.current == "Crossroads_ShamanTemple" && p.current != p.old,
        Split::SalubraExit => p.old == "Room_Charm_Shop" && p.current != p.old,
        Split::EnterHollowKnight => p.current == "Room_Final_Boss_Core" && p.current != p.old,
        // Greenpath
        Split::EnterGreenpath => p.current.starts_with("Fungus1_01") && !p.old.starts_with("Fungus1_01"),
        Split::MenuCloak => is_menu(p.current) && p.old == "Fungus1_04",
        // Fungal
        Split::MenuMantisJournal => is_menu(p.current) && p.old == "Fungus2_17",
        // Resting Grounds
        Split::DreamNailExit => p.old == "Dream_Nailcollection" && p.current == "RestingGrounds_07",
        // City
        Split::TransGorgeousHusk => p.old == "Ruins_House_02" && p.current == "Ruins2_04",
        Split::MenuStoreroomsSimpleKey => is_menu(p.current) && p.old == "Ruins1_17",
        Split::MenuShadeSoul => is_menu(p.current) && p.old.starts_with("Ruins1_31"),
        Split::EnterBlackKnight => p.current == "Ruins2_03" && p.current != p.old,
        Split::BlackKnightTrans => p.current == "Ruins2_Watcher_Room" && p.old == "Ruins2_03",
        // Peak
        Split::MenuSlyKey => is_menu(p.current) && p.old == "Mines_11",
        // Waterways
        Split::DungDefenderExit => p.old == "Waterways_05" && p.current == "Abyss_01",
        Split::MenuIsmasTear => is_menu(p.current) && p.old == "Waterways_13",
        // Basin
        Split::Abyss19from18 => p.old == "Abyss_18" && p.current == "Abyss_19",
        Split::MenuWings => is_menu(p.current) && p.old == "Abyss_21",
        // Fog Canyon
        Split::TeachersArchive => p.current.starts_with("Fungus3_Archive") && !p.old.starts_with("Fungus3_Archive"),
        // Queen's Gardens
        Split::QueensGardensEntry => (p.current.starts_with("Fungus3_34") || p.current.starts_with("Deepnest_43")) && p.current != p.old,
        // else
        _ => false
    }
}

pub fn continuous_splits(s: &Split, p: &Process, g: &GameManagerFinder, pds: &mut PlayerDataStore) -> bool {
    match s {
        // Spell Levels
        Split::VengefulSpirit => g.get_fireball_level(p).is_some_and(|l| 1 <= l),
        Split::ShadeSoul => g.get_fireball_level(p).is_some_and(|l| 2 <= l),
        // Movement Abilities
        Split::MothwingCloak => g.has_dash(p).is_some_and(|d| d),
        Split::ShadeCloak => g.has_shadow_dash(p).is_some_and(|s| s),
        Split::MantisClaw => g.has_wall_jump(p).is_some_and(|w| w),
        Split::MonarchWings => g.has_double_jump(p).is_some_and(|w| w),
        Split::CrystalHeart => g.has_super_dash(p).is_some_and(|s| s),
        Split::IsmasTear => g.has_acid_armour(p).is_some_and(|a| a),
        // Dream Nail Levels
        Split::DreamNail => g.has_dream_nail(p).is_some_and(|d| d),
        Split::DreamGate => g.has_dream_gate(p).is_some_and(|d| d),
        Split::DreamNail2 => g.dream_nail_upgraded(p).is_some_and(|d| d),
        // Other Items
        Split::LumaflyLantern => g.has_lantern(p).is_some_and(|l| l),
        Split::OnObtainSimpleKey => pds.incremented_simple_keys(p, g),
        Split::SlyKey => g.has_sly_key(p).is_some_and(|k| k),
        Split::ElegantKey => g.has_white_key(p).is_some_and(|k| k),
        // else
        _ => false
    }
}

pub fn default_splits() -> Vec<Split> {
    vec![Split::StartNewGame,
         Split::EndingSplit]
}
