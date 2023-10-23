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

    // Masks and Mask Shards
    MaskFragment1,
    MaskFragment2,
    MaskFragment3,
    Mask1,

    // Charms
    GatheringSwarm,
    WaywardCompass,
    Grubsong,
    StalwartShell,
    BaldurShell,
    FuryOfTheFallen,
    QuickFocus,
    LifebloodHeart,
    LifebloodCore,
    DefendersCrest,
    Flukenest,
    ThornsOfAgony,
    MarkOfPride,
    SteadyBody,
    HeavyBlow,
    SharpShadow,
    SporeShroom,
    Longnail,
    ShamanStone,
    SoulCatcher,
    SoulEater,
    GlowingWomb,
    NailmastersGlory,
    JonisBlessing,
    ShapeOfUnn,
    Hiveblood,
    DreamWielder,
    Dashmaster,
    QuickSlash,
    SpellTwister,
    DeepFocus,
    GrubberflysElegy,
    Sprintmaster,
    Dreamshield,
    Weaversong,
    // Fragile / Unbreakable Charms
    FragileHeart,
    UnbreakableHeart,
    FragileGreed,
    UnbreakableGreed,
    FragileStrength,
    UnbreakableStrength,
    AllBreakables,
    AllUnbreakables,
    // Grimmchild / Carefree Melody
    Grimmchild,
    Grimmchild2,
    Grimmchild3,
    Grimmchild4,
    CarefreeMelody,
    // TODO: more multi-level charms Grimmchild/Carefree, Kingsoul/VoidHeart

    // Other Items
    LumaflyLantern,
    OnObtainSimpleKey,
    SlyKey,
    ElegantKey,

    // Grubs
    Grub1,
    Grub2,
    Grub3,
    Grub4,
    Grub5,

    // Dirtmouth
    KingsPass,
    SlyShopExit,
    TroupeMasterGrimm,
    NightmareKingGrimm,
    GreyPrince,
    // Crossroads
    EnterBroodingMawlek,
    BroodingMawlek,
    AncestralMound,
    GruzMother,
    SlyRescued,
    FalseKnight,
    FailedKnight,
    SalubraExit,
    EnterHollowKnight,
    UnchainedHollowKnight,
    HollowKnightBoss,
    HollowKnightDreamnail,
    RadianceBoss,
    // Greenpath
    EnterGreenpath,
    Hornet1,
    NoEyes,
    MegaMossCharger,
    // Fungal
    ElderHu,
    MenuMantisJournal,
    MantisLords,
    // Cliffs
    Gorb,
    // Resting Grounds
    DreamNailExit,
    Xero,
    // City
    GorgeousHusk,
    TransGorgeousHusk,
    MenuGorgeousHusk,
    Lemm2,
    SoulMasterEncountered,
    SoulMasterPhase1,
    SoulMaster,
    SoulTyrant,
    MenuStoreroomsSimpleKey,
    EnterBlackKnight,
    WatcherChandelier,
    BlackKnight,
    BlackKnightTrans,
    Collector,
    // Peak
    MenuSlyKey,
    CrystalGuardian1,
    CrystalGuardian2,
    // Waterways
    DungDefender,
    DungDefenderExit,
    WhiteDefender,
    Flukemarm,
    // Basin
    Abyss19from18,
    BrokenVessel,
    LostKin,
    // Kindgom's Edge
    HiveKnight,
    Hornet2,
    Markoth,
    GodTamer,
    // Fog Canyon
    TeachersArchive,
    UumuuEncountered,
    Uumuu,
    // Queen's Gardens
    QueensGardensEntry,
    Marmu,
    TraitorLord,
    // Deepnest
    Nosk,
    Galien,
    BeastsDenTrapBench,
    // Godhome
    MatoOroNailBros,
    SheoPaintmaster,
    SlyNailsage,
    PureVessel,
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
        Split::HollowKnightDreamnail => p.current.starts_with("Dream_Final") && p.current != p.old,
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
        Split::MenuShadeSoul => { pds.get_fireball_level(p, g); false },
        // Movement Abilities
        Split::MothwingCloak => g.has_dash(p).is_some_and(|d| d),
        Split::MenuCloak => { pds.has_dash(p, g); false },
        Split::ShadeCloak => g.has_shadow_dash(p).is_some_and(|s| s),
        Split::MantisClaw => g.has_wall_jump(p).is_some_and(|w| w),
        Split::MenuClaw => { pds.has_wall_jump(p, g); false },
        Split::MonarchWings => g.has_double_jump(p).is_some_and(|w| w),
        Split::MenuWings => { pds.has_double_jump(p, g); false },
        Split::CrystalHeart => g.has_super_dash(p).is_some_and(|s| s),
        Split::IsmasTear => g.has_acid_armour(p).is_some_and(|a| a),
        Split::MenuIsmasTear => { pds.has_acid_armour(p, g); false },
        // Dream Nail Levels
        Split::DreamNail => g.has_dream_nail(p).is_some_and(|d| d),
        Split::DreamGate => g.has_dream_gate(p).is_some_and(|d| d),
        Split::DreamNail2 => g.dream_nail_upgraded(p).is_some_and(|d| d),
        // Masks and Mask Shards
        Split::MaskFragment1 => g.max_health_base(p).is_some_and(|h| h == 5) && g.heart_pieces(p).is_some_and(|p| p == 1),
        Split::MaskFragment2 => g.max_health_base(p).is_some_and(|h| h == 5) && g.heart_pieces(p).is_some_and(|p| p == 2),
        Split::MaskFragment3 => g.max_health_base(p).is_some_and(|h| h == 5) && g.heart_pieces(p).is_some_and(|p| p == 3),
        Split::Mask1 => g.max_health_base(p).is_some_and(|h| h == 6),
        // Charms
        Split::GatheringSwarm => g.got_charm_1(p).is_some_and(|c| c),
        Split::WaywardCompass => g.got_charm_2(p).is_some_and(|c| c),
        Split::Grubsong => g.got_charm_3(p).is_some_and(|c| c),
        Split::StalwartShell => g.got_charm_4(p).is_some_and(|c| c),
        Split::BaldurShell => g.got_charm_5(p).is_some_and(|c| c),
        Split::FuryOfTheFallen => g.got_charm_6(p).is_some_and(|c| c),
        Split::QuickFocus => g.got_charm_7(p).is_some_and(|c| c),
        Split::LifebloodHeart => g.got_charm_8(p).is_some_and(|c| c),
        Split::LifebloodCore => g.got_charm_9(p).is_some_and(|c| c),
        Split::DefendersCrest => g.got_charm_10(p).is_some_and(|c| c),
        Split::Flukenest => g.got_charm_11(p).is_some_and(|c| c),
        Split::ThornsOfAgony => g.got_charm_12(p).is_some_and(|c| c),
        Split::MarkOfPride => g.got_charm_13(p).is_some_and(|c| c),
        Split::SteadyBody => g.got_charm_14(p).is_some_and(|c| c),
        Split::HeavyBlow => g.got_charm_15(p).is_some_and(|c| c),
        Split::SharpShadow => g.got_charm_16(p).is_some_and(|c| c),
        Split::SporeShroom => g.got_charm_17(p).is_some_and(|c| c),
        Split::Longnail => g.got_charm_18(p).is_some_and(|c| c),
        Split::ShamanStone => g.got_charm_19(p).is_some_and(|c| c),
        Split::SoulCatcher => g.got_charm_20(p).is_some_and(|c| c),
        Split::SoulEater => g.got_charm_21(p).is_some_and(|c| c),
        Split::GlowingWomb => g.got_charm_22(p).is_some_and(|c| c),
        Split::NailmastersGlory => g.got_charm_26(p).is_some_and(|c| c),
        Split::JonisBlessing => g.got_charm_27(p).is_some_and(|c| c),
        Split::ShapeOfUnn => g.got_charm_28(p).is_some_and(|c| c),
        Split::Hiveblood => g.got_charm_29(p).is_some_and(|c| c),
        Split::DreamWielder => g.got_charm_30(p).is_some_and(|c| c),
        Split::Dashmaster => g.got_charm_31(p).is_some_and(|c| c),
        Split::QuickSlash => g.got_charm_32(p).is_some_and(|c| c),
        Split::SpellTwister => g.got_charm_33(p).is_some_and(|c| c),
        Split::DeepFocus => g.got_charm_34(p).is_some_and(|c| c),
        Split::GrubberflysElegy => g.got_charm_35(p).is_some_and(|c| c),
        Split::Sprintmaster => g.got_charm_37(p).is_some_and(|c| c),
        Split::Dreamshield => g.got_charm_38(p).is_some_and(|c| c),
        Split::Weaversong => g.got_charm_39(p).is_some_and(|c| c),
        // Fragile / Unbreakable Charms
        Split::FragileHeart => g.got_charm_23(p).is_some_and(|c| c),
        Split::UnbreakableHeart => g.fragile_health_unbreakable(p).is_some_and(|c| c),
        Split::FragileGreed => g.got_charm_24(p).is_some_and(|c| c),
        Split::UnbreakableGreed => g.fragile_greed_unbreakable(p).is_some_and(|c| c),
        Split::FragileStrength => g.got_charm_25(p).is_some_and(|c| c),
        Split::UnbreakableStrength => g.fragile_strength_unbreakable(p).is_some_and(|c| c),
        Split::AllBreakables => g.broken_charm_23(p).is_some_and(|b| b)
                             && g.broken_charm_24(p).is_some_and(|b| b)
                             && g.broken_charm_25(p).is_some_and(|b| b),
        Split::AllUnbreakables => g.fragile_greed_unbreakable(p).is_some_and(|u| u)
                               && g.fragile_health_unbreakable(p).is_some_and(|u| u)
                               && g.fragile_strength_unbreakable(p).is_some_and(|u| u),
        // Grimmchild / Carefree Melody
        Split::Grimmchild => g.got_charm_40(p).is_some_and(|c| c) && g.grimm_child_level(p).is_some_and(|l| l <= 4),
        Split::Grimmchild2 => g.grimm_child_level(p).is_some_and(|l| 2 <= l && l <= 4),
        Split::Grimmchild3 => g.grimm_child_level(p).is_some_and(|l| 3 <= l && l <= 4),
        Split::Grimmchild4 => g.grimm_child_level(p).is_some_and(|l| l == 4),
        Split::CarefreeMelody => g.got_charm_40(p).is_some_and(|c| c) && g.grimm_child_level(p).is_some_and(|l| l == 5),
        // TODO: other multi-level charms Kingsoul/VoidHeart
        // Other Items
        Split::LumaflyLantern => g.has_lantern(p).is_some_and(|l| l),
        Split::OnObtainSimpleKey => pds.incremented_simple_keys(p, g),
        Split::SlyKey => g.has_sly_key(p).is_some_and(|k| k),
        Split::ElegantKey => g.has_white_key(p).is_some_and(|k| k),
        // Grubs
        Split::Grub1 => g.grubs_collected(p).is_some_and(|g| g == 1),
        Split::Grub2 => g.grubs_collected(p).is_some_and(|g| g == 2),
        Split::Grub3 => g.grubs_collected(p).is_some_and(|g| g == 3),
        Split::Grub4 => g.grubs_collected(p).is_some_and(|g| g == 4),
        Split::Grub5 => g.grubs_collected(p).is_some_and(|g| g == 5),
        // Dirtmouth
        Split::TroupeMasterGrimm => g.killed_grimm(p).is_some_and(|k| k),
        Split::NightmareKingGrimm => g.killed_nightmare_grimm(p).is_some_and(|k| k),
        Split::GreyPrince => g.killed_grey_prince(p).is_some_and(|k| k),
        // Crossroads
        Split::BroodingMawlek => g.killed_mawlek(p).is_some_and(|k| k),
        Split::GruzMother => g.killed_big_fly(p).is_some_and(|f| f),
        Split::SlyRescued => g.sly_rescued(p).is_some_and(|s| s),
        Split::FalseKnight => g.killed_false_knight(p).is_some_and(|k| k),
        Split::FailedKnight => g.false_knight_dream_defeated(p).is_some_and(|k| k),
        Split::UnchainedHollowKnight => g.unchained_hollow_knight(p).is_some_and(|u| u),
        Split::HollowKnightBoss => g.killed_hollow_knight(p).is_some_and(|k| k),
        Split::RadianceBoss => g.killed_final_boss(p).is_some_and(|k| k),
        // Greenpath
        Split::Hornet1 => g.killed_hornet(p).is_some_and(|k| k),
        Split::NoEyes => g.killed_ghost_no_eyes(p).is_some_and(|k| k),
        Split::MegaMossCharger => g.mega_moss_charger_defeated(p).is_some_and(|k| k),
        // Fungal
        Split::ElderHu => g.killed_ghost_hu(p).is_some_and(|k| k),
        Split::MantisLords => g.defeated_mantis_lords(p).is_some_and(|k| k),
        // Cliffs
        Split::Gorb => g.killed_ghost_aladar(p).is_some_and(|k| k),
        // Resting Grounds
        Split::Xero => g.killed_ghost_xero(p).is_some_and(|k| k),
        // City
        Split::GorgeousHusk => pds.killed_gorgeous_husk(p, g),
        Split::TransGorgeousHusk => { pds.killed_gorgeous_husk(p, g); false },
        Split::MenuGorgeousHusk => { pds.killed_gorgeous_husk(p, g); false },
        Split::Lemm2 => g.met_relic_dealer_shop(p).is_some_and(|m| m),
        Split::SoulMasterEncountered => g.mage_lord_encountered(p).is_some_and(|b| b),
        Split::SoulMasterPhase1 => g.mage_lord_encountered_2(p).is_some_and(|b| b),
        Split::SoulMaster => g.killed_mage_lord(p).is_some_and(|k| k),
        Split::SoulTyrant => g.mage_lord_dream_defeated(p).is_some_and(|k| k),
        Split::WatcherChandelier => g.watcher_chandelier(p).is_some_and(|c| c),
        Split::BlackKnight => g.killed_black_knight(p).is_some_and(|k| k),
        Split::Collector => g.collector_defeated(p).is_some_and(|k| k),
        // Peak
        Split::CrystalGuardian1 => g.defeated_mega_beam_miner(p).is_some_and(|k| k),
        // Waterways
        Split::DungDefender => g.killed_dung_defender(p).is_some_and(|k| k),
        Split::WhiteDefender => g.killed_white_defender(p).is_some_and(|k| k),
        Split::Flukemarm => g.killed_fluke_mother(p).is_some_and(|k| k),
        Split::BrokenVessel => g.killed_infected_knight(p).is_some_and(|k| k),
        Split::LostKin => g.infected_knight_dream_defeated(p).is_some_and(|k| k),
        // Kindgom's Edge
        Split::HiveKnight => g.killed_hive_knight(p).is_some_and(|k| k),
        Split::Hornet2 => g.hornet_outskirts_defeated(p).is_some_and(|k| k),
        Split::Markoth => g.killed_ghost_markoth(p).is_some_and(|k| k),
        Split::GodTamer => g.killed_lobster_lancer(p).is_some_and(|k| k),
        // Fog Canyon
        Split::UumuuEncountered => g.encountered_mega_jelly(p).is_some_and(|b| b),
        Split::Uumuu => g.killed_mega_jellyfish(p).is_some_and(|k| k),
        // Queen's Gardens
        Split::Marmu => g.killed_ghost_marmu(p).is_some_and(|k| k),
        Split::TraitorLord => g.killed_traitor_lord(p).is_some_and(|k| k),
        // Deepnest
        Split::Nosk => g.killed_mimic_spider(p).is_some_and(|k| k),
        Split::Galien => g.killed_ghost_galien(p).is_some_and(|k| k),
        Split::BeastsDenTrapBench => g.spider_capture(p).is_some_and(|c| c),
        // Godhome
        Split::MatoOroNailBros => g.killed_nail_bros(p).is_some_and(|k| k),
        Split::SheoPaintmaster => g.killed_paintmaster(p).is_some_and(|k| k),
        Split::SlyNailsage => g.killed_nailsage(p).is_some_and(|k| k),
        Split::PureVessel => g.killed_hollow_knight_prime(p).is_some_and(|k| k),
        // else
        _ => false
    }
}

pub fn default_splits() -> Vec<Split> {
    vec![Split::StartNewGame,
         Split::EndingSplit]
}

pub fn auto_reset_safe(s: &[Split]) -> bool {
    s.first() == Some(&Split::StartNewGame)
    && !s[1..].contains(&Split::StartNewGame)
    && !s[0..(s.len()-1)].contains(&Split::EndingSplit)
}
