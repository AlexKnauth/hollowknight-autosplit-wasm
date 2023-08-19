use asr::Process;
use asr::watcher::Pair;
use serde::{Deserialize, Serialize};

use super::hollow_knight_memory::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub enum Split {
    // Start, End, and Menu
    StartNewGame,
    StartAnyGame,
    EndingSplit,
    Menu,

    // Dreamers
    Lurien,
    Monomon,
    Hegemol,

    // Spell Levels
    VengefulSpirit,
    ShadeSoul,
    MenuShadeSoul,

    // Movement Abilities
    MothwingCloak,
    MenuCloak,
    ShadeCloak,
    MantisClaw,
    MenuClaw,
    MonarchWings,
    MenuWings,
    CrystalHeart,
    IsmasTear,
    MenuIsmasTear,

    // Dream Nail Levels
    DreamNail,
    DreamGate,
    DreamNail2,

    // Charms
    Dashmaster,

    // Other Items
    LumaflyLantern,
    OnObtainSimpleKey,
    SlyKey,
    ElegantKey,

    // Dirtmouth
    KingsPass,
    SlyShopExit,
    // Crossroads
    EnterBroodingMawlek,
    AncestralMound,
    SalubraExit,
    EnterHollowKnight,
    UnchainedHollowKnight,
    // Greenpath
    EnterGreenpath,
    // Fungal
    MenuMantisJournal,
    // Resting Grounds
    DreamNailExit,
    // City
    GorgeousHusk,
    TransGorgeousHusk,
    MenuGorgeousHusk,
    Lemm2,
    MenuStoreroomsSimpleKey,
    EnterBlackKnight,
    WatcherChandelier,
    BlackKnight,
    BlackKnightTrans,
    // Peak
    MenuSlyKey,
    // Waterways
    DungDefenderExit,
    // Basin
    Abyss19from18,
    // Fog Canyon
    TeachersArchive,
    Uumuu,
    // Queen's Gardens
    QueensGardensEntry,
    // Deepnest
    BeastsDenTrapBench,
}

pub fn transition_splits(s: &Split, p: &Pair<&str>, prc: &Process, g: &GameManagerFinder, pds: &mut PlayerDataStore) -> bool {
    match s {
        // Start, End, and Menu
        Split::StartNewGame => {
            (p.old == OPENING_SEQUENCE && p.current == "Tutorial_01") || (is_menu(p.old) && p.current == GG_ENTRANCE_CUTSCENE)
        },
        Split::StartAnyGame => {
            (is_menu(p.old) || p.old == OPENING_SEQUENCE) && (is_play_scene(p.current) || p.current == GG_ENTRANCE_CUTSCENE)
        }
        Split::EndingSplit => p.current.starts_with("Cinematic_Ending"),
        Split::Menu => is_menu(p.current),
        
        // Dreamers
        Split::Lurien => p.old == "Dream_Guardian_Lurien" && p.current == "Cutscene_Boss_Door",
        Split::Monomon => p.old == "Dream_Guardian_Monomon" && p.current == "Cutscene_Boss_Door",
        Split::Hegemol => p.old == "Dream_Guardian_Hegemol" && p.current == "Cutscene_Boss_Door",

        // Dirtmouth
        Split::KingsPass => p.old == "Tutorial_01" && p.current == "Town",
        Split::SlyShopExit => p.old == "Room_shop" && p.current != p.old,
        // Crossroads
        Split::EnterBroodingMawlek => p.current == "Crossroads_09" && p.current != p.old,
        Split::AncestralMound => p.current == "Crossroads_ShamanTemple" && p.current != p.old,
        Split::SalubraExit => p.old == "Room_Charm_Shop" && p.current != p.old,
        Split::EnterHollowKnight => p.current == "Room_Final_Boss_Core" && p.current != p.old,
        // Greenpath
        Split::EnterGreenpath => p.current.starts_with("Fungus1_01") && !p.old.starts_with("Fungus1_01"),
        Split::MenuCloak => pds.has_dash(prc, g) && is_menu(p.current),
        // Fungal
        Split::MenuClaw => pds.has_wall_jump(prc, g) && is_menu(p.current),
        Split::MenuMantisJournal => is_menu(p.current) && p.old == "Fungus2_17",
        // Resting Grounds
        Split::DreamNailExit => p.old == "Dream_Nailcollection" && p.current == "RestingGrounds_07",
        // City
        Split::TransGorgeousHusk => pds.killed_gorgeous_husk(prc, g) && p.current != p.old,
        Split::MenuGorgeousHusk => pds.killed_gorgeous_husk(prc, g) && is_menu(p.current),
        Split::MenuStoreroomsSimpleKey => is_menu(p.current) && p.old == "Ruins1_17",
        Split::MenuShadeSoul => 2 <= pds.get_fireball_level(prc, g) && is_menu(p.current),
        Split::EnterBlackKnight => p.current == "Ruins2_03" && p.current != p.old,
        Split::BlackKnightTrans => p.current == "Ruins2_Watcher_Room" && p.old == "Ruins2_03",
        // Peak
        Split::MenuSlyKey => is_menu(p.current) && p.old == "Mines_11",
        // Waterways
        Split::DungDefenderExit => p.old == "Waterways_05" && p.current == "Abyss_01",
        Split::MenuIsmasTear => pds.has_acid_armour(prc, g) && is_menu(p.current),
        // Basin
        Split::Abyss19from18 => p.old == "Abyss_18" && p.current == "Abyss_19",
        Split::MenuWings => pds.has_double_jump(prc, g) && is_menu(p.current),
        // Fog Canyon
        Split::TeachersArchive => p.current.starts_with("Fungus3_archive") && !p.old.starts_with("Fungus3_archive"),
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
        Split::MenuShadeSoul => 2 <= pds.get_fireball_level(p, g) && false,
        // Movement Abilities
        Split::MothwingCloak => g.has_dash(p).is_some_and(|d| d),
        Split::MenuCloak => pds.has_dash(p, g) && false,
        Split::ShadeCloak => g.has_shadow_dash(p).is_some_and(|s| s),
        Split::MantisClaw => g.has_wall_jump(p).is_some_and(|w| w),
        Split::MenuClaw => pds.has_wall_jump(p, g) && false,
        Split::MonarchWings => g.has_double_jump(p).is_some_and(|w| w),
        Split::MenuWings => pds.has_double_jump(p, g) && false,
        Split::CrystalHeart => g.has_super_dash(p).is_some_and(|s| s),
        Split::IsmasTear => g.has_acid_armour(p).is_some_and(|a| a),
        Split::MenuIsmasTear => pds.has_acid_armour(p, g) && false,
        // Dream Nail Levels
        Split::DreamNail => g.has_dream_nail(p).is_some_and(|d| d),
        Split::DreamGate => g.has_dream_gate(p).is_some_and(|d| d),
        Split::DreamNail2 => g.dream_nail_upgraded(p).is_some_and(|d| d),
        // Charms
        Split::Dashmaster => g.got_charm_31(p).is_some_and(|c| c),
        // Other Items
        Split::LumaflyLantern => g.has_lantern(p).is_some_and(|l| l),
        Split::OnObtainSimpleKey => pds.incremented_simple_keys(p, g),
        Split::SlyKey => g.has_sly_key(p).is_some_and(|k| k),
        Split::ElegantKey => g.has_white_key(p).is_some_and(|k| k),
        // Crossroads
        Split::UnchainedHollowKnight => g.unchained_hollow_knight(p).is_some_and(|u| u),
        // City
        Split::GorgeousHusk => pds.killed_gorgeous_husk(p, g),
        Split::TransGorgeousHusk => pds.killed_gorgeous_husk(p, g) && false,
        Split::MenuGorgeousHusk => pds.killed_gorgeous_husk(p, g) && false,
        Split::Lemm2 => g.met_relic_dealer_shop(p).is_some_and(|m| m),
        Split::WatcherChandelier => g.watcher_chandelier(p).is_some_and(|c| c),
        Split::BlackKnight => g.killed_black_knight(p).is_some_and(|k| k),
        // Fog Canyon
        Split::Uumuu => g.killed_mega_jellyfish(p).is_some_and(|k| k),
        // Deepnest
        Split::BeastsDenTrapBench => g.spider_capture(p).is_some_and(|c| c),
        // else
        _ => false
    }
}

pub fn default_splits() -> Vec<Split> {
    vec![Split::StartNewGame,
         Split::EndingSplit]
}
