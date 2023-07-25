use asr::watcher::Pair;

pub enum Split {
    // Start and End
    StartNewGame,
    EndingSplit,

    // Dreamers
    Lurien,
    Monomon,
    Hegemol,

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
            (p.old == "Opening_Sequence" && p.current == "Tutorial_01") || (is_menu(p.old) && p.current == "GG_Entrance_Cutscene")
        },
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
    }
}

fn is_menu(s: &str) -> bool {
    s == "Menu_Title" || s == "Quit_To_Menu"
}

pub fn default_splits() -> Vec<Split> {
    vec![Split::StartNewGame,
         Split::AncestralMound,
         Split::EnterGreenpath,
         Split::MenuCloak,
         Split::MenuMantisJournal,
         Split::SalubraExit,
         Split::DreamNailExit,
         Split::TransGorgeousHusk,
         Split::MenuStoreroomsSimpleKey,
         Split::SlyShopExit,
         Split::MenuSlyKey,
         Split::DungDefenderExit,
         Split::Abyss19from18,
         Split::MenuWings,
         Split::MenuIsmasTear,
         Split::MenuShadeSoul,
         Split::EnterBlackKnight,
         Split::BlackKnightTrans,
         Split::Lurien,
         Split::TeachersArchive,
         Split::Monomon,
         Split::QueensGardensEntry,
         Split::Hegemol,
         Split::EnterHollowKnight,
         Split::EndingSplit]
}
