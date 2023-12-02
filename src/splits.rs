use std::str::FromStr;

use asr::Process;
use asr::watcher::Pair;
use serde::{Deserialize, Serialize};

use super::auto_splitter_settings::Settings;
use super::hollow_knight_memory::*;

#[derive(Clone, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Split {
    // region: Start, End, and Menu
    /// Start New Game (Start)
    /// 
    /// Splits when starting a new save file, including Normal, Steel Soul, and Godseeker mode
    StartNewGame,
    /// Start Any Game (Start)
    /// 
    /// Splits when entering a new or existing save file
    StartAnyGame,
    /// Credits Roll (Event)
    /// 
    /// Splits on any credits rolling
    #[default]
    EndingSplit,
    /// The Hollow Knight (Ending)
    /// 
    /// Splits on The Hollow Knight ending
    EndingA,
    /// Sealed Siblings (Ending)
    /// 
    /// Splits on Sealed Siblings ending
    EndingB,
    /// Dream No More (Ending)
    /// 
    /// Splits on Dream No More ending
    EndingC,
    /// Embrace the Void (Ending)
    /// 
    /// Splits on Embrace the Void ending
    EndingD,
    /// Delicate Flower (Ending)
    /// 
    /// Splits on Delicate Flower ending
    EndingE,
    /// Absolute Radiance (Pantheon)
    /// 
    /// Splits after killing Absolute Radiance in Pantheon 5
    RadianceP,
    /// Main Menu (Menu)
    /// 
    /// Splits on the main menu
    Menu,
    /// Any Transition (Transition)
    /// 
    /// Splits when the knight enters a transition (only one will split per transition)
    AnyTransition,
    // endregion: Start, End, and Menu

    // region: Dreamers
    /// Lurien the Watcher (Dreamer)
    /// 
    /// Splits when you see the mask for Lurien
    Lurien,
    /// Monomon the Teacher (Dreamer)
    /// 
    /// Splits when you see the mask for Monomon
    Monomon,
    /// Herrah the Beast (Dreamer)
    /// 
    /// Splits when you see the mask for Herrah
    Hegemol,
    /// First Dreamer (Dreamer)
    /// 
    /// Splits when you see the mask for the first dreamer killed
    Dreamer1,
    /// Second Dreamer (Dreamer)
    /// 
    /// Splits when you see the mask for the second dreamer killed
    Dreamer2,
    /// Third Dreamer (Dreamer)
    /// 
    /// Splits when you see the mask for the third dreamer killed
    Dreamer3,
    /// Main Menu w/ 3 Dreamers (Menu)
    /// 
    /// Splits on transition to the main menu after 3 Dreamers acquired
    MenuDreamer3,
    // endregion: Dreamers

    // region: Spell Levels
    /// Vengeful Spirit (Skill)
    /// 
    /// Splits when obtaining Vengeful Spirit
    VengefulSpirit,
    /// Shade Soul (Skill)
    /// 
    /// Splits when obtaining Shade Soul
    ShadeSoul,
    MenuShadeSoul,
    /// Desolate Dive (Skill)
    /// 
    /// Splits when obtaining Desolate Dive
    DesolateDive,
    /// Descending Dark (Skill)
    /// 
    /// Splits when obtaining Descending Dark
    DescendingDark,
    /// Has Descending Dark (Transition)
    /// 
    /// Splits on transition after Descending Dark acquired
    TransDescendingDark,
    /// Howling Wraiths (Skill)
    /// 
    /// Splits when obtaining Howling Wraiths
    HowlingWraiths,
    /// Abyss Shriek (Skill)
    /// 
    /// Splits when obtaining Abyss Shriek
    AbyssShriek,
    // endregion: Spell Levels

    // region: Movement Abilities
    /// Mothwing Cloak (Skill)
    /// 
    /// Splits when obtaining Mothwing Cloak
    MothwingCloak,
    /// Main Menu w/ Mothwing Cloak (Menu)
    /// 
    /// Splits on transition to the main menu after Mothwing Cloak acquired
    MenuCloak,
    /// Shade Cloak (Skill)
    /// 
    /// Splits when obtaining Shade Cloak
    ShadeCloak,
    /// Mantis Claw (Skill)
    /// 
    /// Splits when obtaining Mantis Claw
    MantisClaw,
    /// Main Menu w/ Claw (Menu)
    /// 
    /// Splits on transition to the main menu after Mantis Claw acquired
    MenuClaw,
    /// Monarch Wings (Skill)
    /// 
    /// Splits when obtaining Monarch Wings
    MonarchWings,
    MenuWings,
    /// Crystal Heart (Skill)
    /// 
    /// Splits when obtaining Crystal Heart
    CrystalHeart,
    /// Isma's Tear (Skill)
    /// 
    /// Splits when obtaining Isma's Tear
    IsmasTear,
    /// Main Menu w/ Isma's Tear (Menu)
    /// 
    /// Splits on transition to the main menu after Isma's Tear acquired
    MenuIsmasTear,
    // endregion: Movement Abilities

    // region: Nail Arts
    /// Cyclone Slash (Skill)
    /// 
    /// Splits when obtaining Cyclone Slash
    CycloneSlash,
    /// Dash Slash (Skill)
    /// 
    /// Splits when obtaining Dash Slash
    DashSlash,
    /// Great Slash (Skill)
    /// 
    /// Splits when obtaining Great Slash
    GreatSlash,
    // endregion: Nail Arts

    // region: Dream Nail Levels
    /// Dream Nail (Skill)
    /// 
    /// Splits when obtaining Dream Nail
    DreamNail,
    /// Main Menu w/ Dream Nail (Menu)
    /// 
    /// Splits on transition to the main menu after Dream Nail acquired
    MenuDreamNail,
    /// Dream Gate (Skill)
    /// 
    /// Splits when obtaining Dream Gate
    DreamGate,
    /// Main Menu w/ Dream Gate (Menu)
    /// 
    /// Splits on transition to the main menu after Dream Gate acquired
    MenuDreamGate,
    /// Dream Nail - Awoken (Skill)
    /// 
    /// Splits when Awkening the Dream Nail
    DreamNail2,
    // endregion: Dream Nail Levels

    // region: Keys
    /// City Crest (Item)
    /// 
    /// Splits when obtaining the City Crest
    CityKey,
    /// Lumafly Lantern (Item)
    /// 
    /// Splits when obtaining the Lumafly Lantern
    LumaflyLantern,
    /// Shop Lumafly Lantern (Transition)
    /// 
    /// Splits on transition after Lantern has been acquired
    LumaflyLanternTransition,
    /// Simple Key (Obtain)
    /// 
    /// Splits when obtaining a Simple Key
    OnObtainSimpleKey,
    /// Shopkeeper's Key (Item)
    /// 
    /// Splits when obtaining the Shopkeeper's Key
    SlyKey,
    /// Elegant Key (Item)
    /// 
    /// Splits when obtaining the Elegant Key
    ElegantKey,
    /// Love Key (Item)
    /// 
    /// Splits when obtaining the Love Key
    LoveKey,
    /// Pale Lurker Key (Item)
    /// 
    /// Splits when obtaining the Simple Key from the Pale Lurker
    PaleLurkerKey,
    /// Sly Simple Key (Item)
    /// 
    /// Splits when buying the simple key from Sly
    SlySimpleKey,
    /// King's Brand (Item)
    /// 
    /// Splits when obtaining the King's Brand
    KingsBrand,
    /// Tram Pass (Item)
    /// 
    /// Splits when obtaining the Tram Pass
    TramPass,
    // endregion: Keys

    // region: Nail and Pale Ore
    /// Pale Ore (Obtain)
    /// 
    /// Splits when obtaining a Pale Ore
    OnObtainPaleOre,
    /// Pale Ore 1 (Ore)
    /// 
    /// Splits after obtaining the first pale ore.
    Ore1,
    /// Pale Ore 2 (Ore)
    /// 
    /// Splits after obtaining the second pale ore.
    Ore2,
    /// Pale Ore 3 (Ore)
    /// 
    /// Splits after obtaining the third pale ore.
    Ore3,
    /// Pale Ore 4 (Ore)
    /// 
    /// Splits after obtaining the fourth pale ore.
    Ore4,
    /// Pale Ore 5 (Ore)
    /// 
    /// Splits after obtaining the fifth pale ore.
    Ore5,
    /// Pale Ore 6 (Ore)
    /// 
    /// Splits after obtaining the sixth pale ore.
    Ore6,
    /// Nail 1 (Upgrade)
    /// 
    /// Splits upon upgrading to the Sharpened Nail
    NailUpgrade1,
    /// Nail 2 (Upgrade)
    /// 
    /// Splits upon upgrading to the Channeled Nail
    NailUpgrade2,
    /// Nail 3 (Upgrade)
    /// 
    /// Splits upon upgrading to the Coiled Nail
    NailUpgrade3,
    /// Nail 4 (Upgrade)
    /// 
    /// Splits upon upgrading to the Pure Nail
    NailUpgrade4,
    // endregion: Nail and Pale Ore

    // region: Masks and Mask Shards
    /// Mask Shard 1 (Fragment)
    /// 
    /// Splits when getting 1st Mask Shard
    MaskFragment1,
    /// Mask Shard 2 (Fragment)
    /// 
    /// Splits when getting 2nd Mask Shard
    MaskFragment2,
    /// Mask Shard 3 (Fragment)
    /// 
    /// Splits when getting 3rd Mask Shard
    MaskFragment3,
    /// Mask Upgrade 4 (Upgrade)
    /// 
    /// Splits when getting 1 extra Mask (6 base HP)
    Mask1,
    // endregion: Masks and Mask Shards

    // region: Charm Notches
    /// Shrumal Ogres (Charm Notch)
    /// 
    /// Splits when obtaining the charm notch after defeating the Shrumal Ogres
    NotchShrumalOgres,
    /// Salubra 1 (Charm Notch)
    /// 
    /// Splits when obtaining the first charm notch from Salubra
    NotchSalubra1,
    /// Salubra 2 (Charm Notch)
    /// 
    /// Splits when obtaining the second charm notch from Salubra
    NotchSalubra2,
    /// Salubra 3 (Charm Notch)
    /// 
    /// Splits when obtaining the third charm notch from Salubra
    NotchSalubra3,
    /// Salubra 4 (Charm Notch)
    /// 
    /// Splits when obtaining the fourth charm notch from Salubra
    NotchSalubra4,
    /// Fog Canyon (Charm Notch)
    /// 
    /// Splits when obtaining the charm notch in Fog Canyon
    NotchFogCanyon,
    /// Grimm (Charm Notch)
    /// 
    /// Splits when obtaining the charm notch after Grimm
    NotchGrimm,
    /// Charm Notch (Obtain)
    /// 
    /// Splits when obtaining a new Charm Slot
    OnObtainCharmNotch,
    // endregion: Charm Notches

    // region: Charms
    /// Gathering Swarm (Charm)
    /// 
    /// Splits when obtaining the Gathering Swarm charm
    GatheringSwarm,
    /// Wayward Compass (Charm)
    /// 
    /// Splits when obtaining Wayward Compass charm
    WaywardCompass,
    /// Grubsong (Charm)
    /// 
    /// Splits when obtaining the Grubsong charm
    Grubsong,
    /// Stalwart Shell (Charm)
    /// 
    /// Splits when obtaining Stalwart Shell charm
    StalwartShell,
    /// Baldur Shell (Charm)
    /// 
    /// Splits when obtaining the Baldur Shell charm
    BaldurShell,
    /// Fury of the Fallen (Charm)
    /// 
    /// Splits when obtaining the Fury of the Fallen charm
    FuryOfTheFallen,
    /// Quick Focus (Charm)
    /// 
    /// Splits when obtaining the Quick Focus charm
    QuickFocus,
    /// Lifeblood Heart (Charm)
    /// 
    /// Splits when obtaining the Lifeblood Heart charm
    LifebloodHeart,
    /// Lifeblood Core (Charm)
    /// 
    /// Splits when obtaining the Lifeblood Core charm
    LifebloodCore,
    /// Defenders Crest (Charm)
    /// 
    /// Splits when obtaining the Defenders Crest charm
    DefendersCrest,
    /// Flukenest (Charm)
    /// 
    /// Splits when obtaining the Flukenest charm
    Flukenest,
    /// Thorns of Agony (Charm)
    /// 
    /// Splits when obtaining Thorns of Agony charm
    ThornsOfAgony,
    /// Mark of Pride (Charm)
    /// 
    /// Splits when obtaining the Mark of Pride charm
    MarkOfPride,
    /// Steady Body (Charm)
    /// 
    /// Splits when obtaining the Steady Body charm
    SteadyBody,
    /// Heavy Blow (Charm)
    /// 
    /// Splits when obtaining the Heavy Blow charm
    HeavyBlow,
    /// Sharp Shadow (Charm)
    /// 
    /// Splits when obtaining Sharp Shadow charm
    SharpShadow,
    /// Spore Shroom (Charm)
    /// 
    /// Splits when obtaining the Spore Shroom charm
    SporeShroom,
    /// Longnail (Charm)
    /// 
    /// Splits when obtaining the Longnail charm
    Longnail,
    /// Shaman Stone (Charm)
    /// 
    /// Splits when obtaining Shaman Stone charm
    ShamanStone,
    /// Soul Catcher (Charm)
    /// 
    /// Splits when obtaining the Soul Catcher charm
    SoulCatcher,
    /// Soul Eater (Charm)
    /// 
    /// Splits when obtaining the Soul Eater charm
    SoulEater,
    /// Glowing Womb (Charm)
    /// 
    /// Splits when obtaining the Glowing Womb charm
    GlowingWomb,
    /// Nailmaster's Glory (Charm)
    /// 
    /// Splits when obtaining the Nailmaster's Glory charm
    NailmastersGlory,
    /// Joni's Blessing (Charm)
    /// 
    /// Splits when obtaining the Joni's Blessing charm
    JonisBlessing,
    /// Shape of Unn (Charm)
    /// 
    /// Splits when obtaining Shape of Unn charm
    ShapeOfUnn,
    /// Hiveblood (Charm)
    /// 
    /// Splits when obtaining the Hiveblood charm
    Hiveblood,
    /// Dream Wielder (Charm)
    /// 
    /// Splits when obtaining the Dream Wielder charm
    DreamWielder,
    /// Dashmaster (Charm)
    /// 
    /// Splits when obtaining the Dashmaster charm
    Dashmaster,
    /// Main Menu w/ Dashmaster (Menu)
    /// 
    /// Splits on transition to the main menu after Dashmaster acquired
    MenuDashmaster,
    /// Quick Slash (Charm)
    /// 
    /// Splits when obtaining the Quick Slash charm
    QuickSlash,
    /// Spell Twister (Charm)
    /// 
    /// Splits when obtaining the Spell Twister charm
    SpellTwister,
    /// Deep Focus (Charm)
    /// 
    /// Splits when obtaining the Deep Focus charm
    DeepFocus,
    /// Grubberfly's Elegy (Charm)
    /// 
    /// Splits when obtaining the Grubberfly's Elegy charm
    GrubberflysElegy,
    /// Sprintmaster (Charm)
    /// 
    /// Splits when obtaining the Sprintmaster charm
    Sprintmaster,
    /// Dreamshield (Charm)
    /// 
    /// Splits when obtaining the Dreamshield charm
    Dreamshield,
    /// Weaversong (Charm)
    /// 
    /// Splits when obtaining the Weaversong charm
    Weaversong,
    // Fragile / Unbreakable Charms
    /// Fragile Heart (Charm)
    /// 
    /// Splits when obtaining the Fragile Heart charm
    FragileHeart,
    /// Unbreakable Heart (Charm)
    /// 
    /// Splits when obtaining the Unbreakable Heart charm
    UnbreakableHeart,
    /// Fragile Greed (Charm)
    /// 
    /// Splits when obtaining the Fragile Greed charm
    FragileGreed,
    /// Unbreakable Greed (Charm)
    /// 
    /// Splits when obtaining the Unbreakable Greed charm
    UnbreakableGreed,
    /// Fragile Strength (Charm)
    /// 
    /// Splits when obtaining the Fragile Strength charm
    FragileStrength,
    /// Unbreakable Strength (Charm)
    /// 
    /// Splits when obtaining the Unbreakable Strength charm
    UnbreakableStrength,
    /// All Breakables (Event)
    /// 
    /// Splits when all 3 fragile charms are broken
    AllBreakables,
    /// All Unbreakables (Charm)
    /// 
    /// Splits when all 3 unbreakable charms are obtained
    AllUnbreakables,
    // Grimmchild / Carefree Melody
    /// Grimmchild (Charm)
    /// 
    /// Splits when obtaining the Grimmchild charm
    Grimmchild,
    /// Grimmchild Lvl 2 (Charm)
    /// 
    /// Splits when upgrading Grimmchild to level 2
    Grimmchild2,
    /// Grimmchild Lvl 3 (Charm)
    /// 
    /// Splits when upgrading Grimmchild to level 3
    Grimmchild3,
    /// Grimmchild Lvl 4 (Charm)
    /// 
    /// Splits when upgrading Grimmchild to level 4
    Grimmchild4,
    /// Carefree Melody (Charm)
    /// 
    /// Splits when obtaining the Carefree Melody charm
    CarefreeMelody,
    /// Grimm Flame 1 (Flame)
    /// 
    /// Splits after obtaining the first flame.
    Flame1,
    /// Grimm Flame 2 (Flame)
    /// 
    /// Splits after obtaining the second flame.
    Flame2,
    /// Grimm Flame 3 (Flame)
    /// 
    /// Splits after obtaining the third flame.
    Flame3,
    /// Brumm Flame (NPC)
    /// 
    /// Splits when collecting Brumm's flame in Deepnest
    BrummFlame,
    // Kingsoul / VoidHeart
    /// White Fragment - Queen's (Charm)
    /// 
    /// Splits on picking up the left White Fragment from the White Lady
    WhiteFragmentLeft,
    /// White Fragment - King's (Charm)
    /// 
    /// Splits on picking up the right White Fragment from the Pale King
    WhiteFragmentRight,
    /// White Fragment (Obtain)
    /// 
    /// Splits when obtaining any White Fragment, or Void Heart
    OnObtainWhiteFragment,
    /// Kingsoul (Charm)
    /// 
    /// Splits when obtaining the completed Kingsoul charm
    Kingsoul,
    /// Void Heart (Charm)
    /// 
    /// Splits when changing the Kingsoul to the Void Heart charm
    VoidHeart,
    /// Main Menu w/ Void Heart (Menu)
    /// 
    /// Splits on transition to the main menu after Void Heart acquired
    MenuVoidHeart,
    // endregion: Charms

    // region: Stags
    /// Riding Stag (Event)
    /// 
    /// Splits while riding the stag
    RidingStag,
    /// Stag Position Updated (Event)
    /// 
    /// Splits when the stag is called
    StagMoved,
    /// Forgotten Crossroads (Stag Station)
    /// 
    /// Splits when opening the Forgotten Crossroads Stag Station
    CrossroadsStation,
    /// Greenpath (Stag Station)
    /// 
    /// Splits when obtaining Greenpath Stag Station
    GreenpathStation,
    /// Queen's Station (Stag Station)
    /// 
    /// Splits when obtaining Queen's Station Stag Station
    QueensStationStation,
    /// City Storerooms (Stag Station)
    /// 
    /// Splits when obtaining City Storerooms Stag Station
    StoreroomsStation,
    /// King's Station (Stag Station)
    /// 
    /// Splits when obtaining King's Station Stag Station
    KingsStationStation,
    /// Resting Grounds (Stag Station)
    /// 
    /// Splits when obtaining Resting Grounds Stag Station
    RestingGroundsStation,
    /// Hidden Station (Stag Station)
    /// 
    /// Splits when obtaining to Hidden Station Stag Station
    HiddenStationStation,
    /// Distant Village (Stag Station)
    /// 
    /// Splits when obtaining Distant Village Stag Station
    DeepnestStation,
    /// Queen's Gardens (Stag Station)
    /// 
    /// Splits when obtaining Queen's Gardens Stag Station
    QueensGardensStation,
    /// Stagnest (Stag Station)
    /// 
    /// Splits when traveling to Stagnest (Requires Ordered Splits)
    StagnestStation,
    // endregion: Stags

    // region: Relics
    /// Wanderer's Journal (Obtain)
    /// 
    /// Splits when obtaining a Wanderer's Journal
    OnObtainWanderersJournal,
    /// All Seals (Item)
    /// 
    /// Splits when 17 Hallownest Seals have been collected
    AllSeals,
    /// Hallownest Seal (Obtain)
    /// 
    /// Splits when obtaining a Hallownest Seal
    OnObtainHallownestSeal,
    /// Soul Sanctum Hallownest Seal (Relic)
    /// 
    /// Splits when the Hallownest Seal in Soul Sanctum is collected
    SoulSanctumSeal,
    /// King's Idol (Obtain)
    /// 
    /// Splits when obtaining a King's Idol
    OnObtainKingsIdol,
    /// Glade Idol (Item)
    /// 
    /// Splits when picking up the King's Idol in the Spirits' Glade
    GladeIdol,
    /// Dung Defender Idol (Item)
    /// 
    /// Splits when picking up Dung Defender idol as the first idol
    DungDefenderIdol,
    /// Arcane Egg 8 (Obtain)
    /// 
    /// Splits when obtaining 8 Arcane Eggs
    ArcaneEgg8,
    /// Arcane Egg (Obtain)
    /// 
    /// Splits when obtaining an Arcane Egg
    OnObtainArcaneEgg,
    /// Rancid Egg (Obtain)
    /// 
    /// Splits when obtaining a Rancid Egg
    OnObtainRancidEgg,
    // endregion: Relics

    // region: Grubs
    /// Rescued Grub 1 (Grub)
    /// 
    /// Splits when rescuing grub #1
    Grub1,
    /// Rescued Grub 2 (Grub)
    /// 
    /// Splits when rescuing grub #2
    Grub2,
    /// Rescued Grub 3 (Grub)
    /// 
    /// Splits when rescuing grub #3
    Grub3,
    /// Rescued Grub 4 (Grub)
    /// 
    /// Splits when rescuing grub #4
    Grub4,
    /// Rescued Grub 5 (Grub)
    /// 
    /// Splits when rescuing grub #5
    Grub5,
    // endregion: Grubs

    // region: Dirtmouth
    /// King's Pass (Transition)
    /// 
    /// Splits when entering Dirtmouth from King's Pass
    KingsPass,
    /// Dirtmouth (Transition)
    /// 
    /// Splits on any transition into Dirtmouth Town
    EnterDirtmouth,
    SlyShopExit,
    /// Enter Troupe Master Grimm (Transition)
    /// 
    /// Splits when entering Grimm tent with requirements to trigger Troupe Master Grimm boss
    EnterTMG,
    /// Troupe Master Grimm (Boss)
    /// 
    /// Splits when killing Troupe Master Grimm
    TroupeMasterGrimm,
    /// NKG Dream (Transition)
    /// 
    /// Splits on transition into Nightmare King Grimm dream
    EnterNKG,
    /// Nightmare King Grimm (Boss)
    /// 
    /// Splits when killing Nightmare King Grimm
    NightmareKingGrimm,
    /// Grey Prince Zote (Boss)
    /// 
    /// Splits when killing Grey Prince
    GreyPrince,
    /// Grey Prince Zote (Essence)
    /// 
    /// Splits when getting Grey Prince Zote essence
    GreyPrinceEssence,
    // endregion: Dirtmouth
    // region: Crossroads
    /// Menderbug (Killed)
    /// 
    /// Splits when killing Menderbug
    MenderBug,
    /// Enter Brooding Mawlek (Transition)
    /// 
    /// Splits when entering the Brooding Mawlek arena transition in Forgotten Crossroads
    EnterBroodingMawlek,
    /// Brooding Mawlek (Boss)
    /// 
    /// Splits when killing Brooding Mawlek
    BroodingMawlek,
    /// Ancestral Mound (Transition)
    /// 
    /// Splits on transition into Ancestral Mound
    AncestralMound,
    /// Gruz Mother (Boss)
    /// 
    /// Splits when killing Gruz Mother
    GruzMother,
    /// Sly Rescued (NPC)
    /// 
    /// Splits when saving Sly
    SlyRescued,
    /// False Knight (Boss)
    /// 
    /// Splits when killing False Knight
    FalseKnight,
    /// Failed Champion (Boss)
    /// 
    /// Splits when killing Failed Champion
    FailedKnight,
    /// Failed Champion (Essence)
    /// 
    /// Splits when getting Failed Champion essence
    FailedChampionEssence,
    /// Salubra Exit (Transition)
    /// 
    /// Splits on the transition out of Salubra's Hut
    SalubraExit,
    EnterHollowKnight,
    /// Chains Broken - Hollow Knight (Event)
    /// 
    /// Splits at the end of the first Hollow Knight scream after the chains are broken
    UnchainedHollowKnight,
    /// Segment Practice - THK (Boss)
    /// 
    /// Splits when killing The Hollow Knight
    HollowKnightBoss,
    /// Radiance Dream Entry (Event)
    /// 
    /// Splits upon entering the Radiance dream
    // TODO: Skips upon killing the Hollow Knight (requires ordered splits)
    HollowKnightDreamnail,
    /// Segment Practice - Radiance (Boss)
    /// 
    /// Splits when killing The Radiance
    RadianceBoss,
    // endregion: Crossroads
    // region: Greenpath
    /// Greenpath (Transition)
    /// 
    /// Splits when entering Greenpath
    EnterGreenpath,
    /// Moss Knight (Mini Boss)
    /// 
    /// Splits when killing Moss Knight
    MossKnight,
    /// Zote Rescued - Vengefly King (Mini Boss)
    /// 
    /// Splits when rescuing Zote from the Vengefly King
    Zote1,
    /// Vengefly King Killed (Transition)
    /// 
    /// Splits on transition after Vengefly King in Greenpath killed
    VengeflyKingTrans,
    /// Enter Hornet 1 (Transition)
    /// 
    /// Splits when entering Hornet boss arena transition in Greenpath
    EnterHornet1,
    /// Hornet 1 (Boss)
    /// 
    /// Splits when killing Hornet Protector in Greenpath
    Hornet1,
    /// Aluba (Killed)
    /// 
    /// Splits when killing an Aluba
    Aluba,
    /// No Eyes (Boss)
    /// 
    /// Splits when killing No Eyes
    NoEyes,
    /// No Eyes (Essence)
    /// 
    /// Splits when absorbing essence from No Eyes
    NoEyesEssence,
    /// Massive Moss Charger (Boss)
    /// 
    /// Splits when killing Massive Moss Charger
    MegaMossCharger,
    /// Massive Moss Charger Killed (Transition)
    /// 
    /// Splits on transition after Massive Moss Charger is killed
    MegaMossChargerTrans,
    // endregion: Greenpath
    // region: Fungal
    /// Fungal Wastes Entry (Transition)
    /// 
    /// Splits on transition to Fungal Wastes
    /// 
    /// (Room below Crossroads, right of Queen's Station, left of Waterways or Spore Shroom room)
    FungalWastesEntry,
    /// Elder Hu (Boss)
    /// 
    /// Splits when killing Elder Hu
    ElderHu,
    /// Elder Hu (Essence)
    /// 
    /// Splits when absorbing essence from Elder Hu
    ElderHuEssence,
    /// Elder Hu Killed (Transition)
    /// 
    /// Splits on the transition after killing Elder Hu
    ElderHuTrans,
    MenuMantisJournal,
    /// Mantis Lords (Boss)
    /// 
    /// Splits when killing Mantis Lords
    MantisLords,
    // endregion: Fungal
    // region: Cliffs
    /// Gorb (Boss)
    /// 
    /// Splits when killing Gorb
    Gorb,
    /// Gorb (Essence)
    /// 
    /// Splits when absorbing essence from Gorb
    GorbEssence,
    /// Nightmare Lantern Lit (Event)
    /// 
    /// Splits when initially lighting the Nightmare Lantern
    NightmareLantern,
    /// Nightmare Lantern Destroyed (Event)
    /// 
    /// Splits when destroying the Nightmare Lantern
    NightmareLanternDestroyed,
    // endregion: Cliffs
    // region: Resting Grounds
    /// Blue Lake (Transition)
    /// 
    /// Splits on transition to Blue Lake from either side
    BlueLake,
    /// Enter Any Dream (Transition)
    /// 
    /// Splits when entering any dream world
    EnterAnyDream,
    DreamNailExit,
    /// Xero (Boss)
    /// 
    /// Splits when killing Xero
    Xero,
    /// Xero (Essence)
    /// 
    /// Splits when absorbing essence from Xero
    XeroEssence,
    /// Catacombs Entry (Transition)
    /// 
    /// Splits on entry to the catacombs below Resting Grounds
    CatacombsEntry,
    // endregion: Resting Grounds
    // region: City
    /// City Gate (Event)
    /// 
    /// Splits when using the City Crest to open the gate
    CityGateOpen,
    /// City Gate w/ Mantis Lords defeated (Event)
    /// 
    /// To make sure you don't forget Mantis Lords
    CityGateAndMantisLords,
    /// Gorgeous Husk (Killed)
    /// 
    /// Splits when killing Gorgeous Husk
    GorgeousHusk,
    /// Gorgeous Husk Killed (Transition)
    /// 
    /// Splits on transition after Gorgeous Husk defeated
    TransGorgeousHusk,
    /// Main Menu w/ Ghusk (Menu)
    /// 
    /// Splits on transition to the main menu after Gorgeous Husk defeated
    MenuGorgeousHusk,
    /// Rafters (Transition)
    /// 
    /// Splits on any transition into the City Rafters room
    EnterRafters,
    /// Lemm Shop (NPC)
    /// 
    /// Splits when talking to Lemm in the shop for the first time
    Lemm2,
    /// Soul Sanctum (Transition)
    /// 
    /// Splits when entering Soul Sanctum
    EnterSanctum,
    /// Enter Soul Master (Transition)
    /// 
    /// Splits when entering Soul Master boss arena transition
    EnterSoulMaster,
    /// Soul Master Encountered (Boss)
    /// 
    /// Splits when Soul Master is activated the first time as the gate closes
    SoulMasterEncountered,
    /// Soul Master - Fake Spell Pickup (Boss)
    /// 
    /// Splits when triggering Soul Master phase 2 the first time
    SoulMasterPhase1,
    /// Soul Master (Boss)
    /// 
    /// Splits when killing Soul Master
    SoulMaster,
    /// Soul Tyrant (Boss)
    /// 
    /// Splits when killing Soul Tyrant
    SoulTyrant,
    /// Soul Tyrant (Essence)
    /// 
    /// Splits when getting Soul Tyrant essence
    SoulTyrantEssence,
    MenuStoreroomsSimpleKey,
    EnterBlackKnight,
    /// Chandelier - Watcher Knights (Event)
    /// 
    /// Splits when dropping the chandelier on one of the Watcher Knights
    WatcherChandelier,
    /// Watcher Knight (Boss)
    /// 
    /// Splits when killing Watcher Knights
    BlackKnight,
    /// Watcher Knight Killed (Transition)
    /// 
    /// Splits on the transition after killing Watcher Knights
    BlackKnightTrans,
    /// Tower of Love (Transition)
    /// 
    /// Splits when entering the Tower of Love
    EnterLoveTower,
    /// Collector (Boss)
    /// 
    /// Splits when killing Collector
    Collector,
    /// Collector Defeated (Transition)
    /// 
    /// Splits on transition after defeating the Collector
    TransCollector,
    // endregion: City
    // region: Peak
    /// Crystal Peak Entry (Transition)
    /// 
    /// Splits on transition to the room where the dive and toll entrances meet, or the room right of Dirtmouth
    CrystalPeakEntry,
    /// Husk Miner (Killed)
    /// 
    /// Splits when killing a Husk Miner
    HuskMiner,
    MenuSlyKey,
    /// Crystal Guardian (Boss)
    /// 
    /// Splits when killing the Crystal Guardian
    CrystalGuardian1,
    /// Enraged Guardian (Boss)
    /// 
    /// Splits when killing the Enraged Guardian
    CrystalGuardian2,
    /// Hallownest's Crown (Transition)
    /// 
    /// Splits on transition into the room with the Whispering Root at the base of Hallownest's Crown
    EnterCrown,
    /// Crystal Mound Exit (Transition)
    /// 
    /// Splits on transition from Crystal Mound
    CrystalMoundExit,
    /// Crystal Peak Lift Opened (Event)
    /// 
    /// Splits when opening the lever for the lift between Dirtmouth and Crystal Peak
    MineLiftOpened,
    // endregion: Peak
    // region: Waterways
    /// Dung Defender (Boss)
    /// 
    /// Splits when killing Dung Defender
    DungDefender,
    DungDefenderExit,
    /// White Defender (Boss)
    /// 
    /// Splits when killing White Defender
    WhiteDefender,
    /// White Defender (Essence)
    /// 
    /// Splits when getting White Defender essence
    WhiteDefenderEssence,
    /// Flukemarm (Boss)
    /// 
    /// Splits when killing Flukemarm
    Flukemarm,
    /// Junk Pit (Transition)
    /// 
    /// Splits on transition into Junk Pit
    EnterJunkPit,
    // endregion: Waterways
    // region: Basin
    /// Ancient Basin (Transition)
    /// 
    /// Splits on transition to Ancient Basin
    BasinEntry,
    Abyss19from18,
    /// Broken Vessel (Boss)
    /// 
    /// Splits when killing Broken Vessel
    BrokenVessel,
    /// Broken Vessel (Transition)
    /// 
    /// Splits on any non-death transition after defeating Broken Vessel
    BrokenVesselTrans,
    /// Lost Kin (Boss)
    /// 
    /// Splits when killing Lost Kin
    LostKin,
    /// Lost Kin (Essence)
    /// 
    /// Splits when getting Lost Kin essence
    LostKinEssence,
    // endregion: Basin
    // region: Kingdom's Edge
    /// Hive (Transition)
    /// 
    /// Splits on transition to Hive
    HiveEntry,
    /// Enter Hive Knight (Transition)
    /// 
    /// Splits when entering Hive Knight boss arena transition
    EnterHiveKnight,
    /// Hive Knight (Boss)
    /// 
    /// Splits when killing Hive Knight
    HiveKnight,
    /// Great Hopper (Killed)
    /// 
    /// Splits when killing a Great Hopper
    GreatHopper,
    /// Enter Hornet 2 (Transition)
    /// 
    /// Splits when entering Hornet boss arena transition in Kingdom's Edge
    EnterHornet2,
    /// Hornet 2 (Boss)
    /// 
    /// Splits when killing Hornet Sentinel in Kingdom's Edge
    Hornet2,
    /// Markoth (Boss)
    /// 
    /// Splits when killing Markoth
    Markoth,
    /// Markoth (Essence)
    /// 
    /// Splits when absorbing essence from Markoth
    MarkothEssence,
    /// Zote Defeated - Colosseum (Mini Boss)
    /// 
    /// Splits when defeating Zote in the Colosseum
    ZoteKilled,
    /// God Tamer (Boss)
    /// 
    /// Splits when killing the God Tamer
    GodTamer,
    // endregion: Kingdom's Edge
    // region: Fog Canyon
    /// Teachers Archive (Area)
    /// 
    /// Splits when entering Teachers Archive for the first time
    TeachersArchive,
    /// Uumuu Encountered (Boss)
    /// 
    /// Splits Uumuu is activated the first time as the gate closes
    UumuuEncountered,
    /// Uumuu (Boss)
    /// 
    /// Splits when killing Uumuu
    Uumuu,
    // endregion: Fog Canyon
    // region: Queen's Gardens
    /// Queen's Garden Entry (Transition)
    /// 
    /// Splits on transition to QG scene following QGA or above Deepnest
    QueensGardensEntry,
    /// Marmu (Boss)
    /// 
    /// Splits when killing Marmu
    Marmu,
    /// Marmu (Essence)
    /// 
    /// Splits when absorbing essence from Marmu
    MarmuEssence,
    /// Traitor Lord (Boss)
    /// 
    /// Splits when killing Traitor Lord
    TraitorLord,
    // endregion: Queen's Gardens
    // region: Deepnest
    /// Deepnest (Transition)
    /// 
    /// Splits on transition into Deepnest
    EnterDeepnest,
    /// Zote Rescued - Deepnest (Mini Boss)
    /// 
    /// Splits when rescuing Zote in Deepnest
    Zote2,
    /// Nosk (Transition)
    /// 
    /// Splits when entering Nosk boss arena transition
    EnterNosk,
    /// Nosk (Boss)
    /// 
    /// Splits when killing Nosk
    Nosk,
    /// Galien (Boss)
    /// 
    /// Splits when killing Galien
    Galien,
    /// Galien (Essence)
    /// 
    /// Splits when absorbing essence from Galien
    GalienEssence,
    /// Trap Bench (Event)
    /// 
    /// Splits when getting the trap bench in Beasts Den
    BeastsDenTrapBench,
    // endregion: Deepnest
    // region: Godhome
    /// Godhome (Transition)
    /// 
    /// Splits on transition to Godhome
    EnterGodhome,
    /// Oro & Mato Nail Bros (Boss)
    /// 
    /// Splits when defeating Brothers Oro & Mato
    MatoOroNailBros,
    /// Paintmaster Sheo (Boss)
    /// 
    /// Splits when killing Paintmaster Sheo
    SheoPaintmaster,
    /// Great Nailsage Sly (Boss)
    /// 
    /// Splits when killing Great Nailsage Sly
    SlyNailsage,
    /// Pure Vessel (Boss)
    /// 
    /// Splits when killing Pure Vessel
    PureVessel,
    // endregion: Godhome
}

impl ToString for Split {
    fn to_string(&self) -> String {
        serde_json::to_value(self).unwrap_or_default().as_str().unwrap_or_default().to_string()
    }
}

impl FromStr for Split {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Split, serde_json::Error> {
        serde_json::value::from_value(serde_json::Value::String(s.to_string()))
    }
}

impl Split {
    fn from_settings_str<S: Settings>(s: S) -> Option<Split> {
        Split::from_str(&s.as_string()?).ok()
    }
    fn from_settings_split<S: Settings>(s: S) -> Option<Split> {
        Split::from_settings_str(s.dict_get("Split").unwrap_or(s))
    }
}

pub fn transition_splits(s: &Split, p: &Pair<&str>, prc: &Process, g: &GameManagerFinder, pds: &mut PlayerDataStore) -> bool {
    match s {
        // region: Start, End, and Menu
        Split::EndingSplit => p.current.starts_with("Cinematic_Ending"),
        Split::EndingA => p.current == "Cinematic_Ending_A",
        Split::EndingB => p.current == "Cinematic_Ending_B",
        Split::EndingC => p.current == "Cinematic_Ending_C",
        Split::EndingD => p.current == "Cinematic_Ending_D",
        Split::EndingE => p.current == "Cinematic_Ending_E",
        Split::RadianceP => p.old.starts_with("GG_Radiance") && p.current.starts_with("Cinematic_Ending"),
        Split::Menu => is_menu(p.current),
        Split::AnyTransition => p.current != p.old && !(p.old.is_empty() || p.current.is_empty() || is_menu(p.old)),
        // endregion: Start, End, and Menu
        
        // region: Dreamers
        /*
        // Old scene-transition based dreamer splits from when I only knew how to read the scene name
        Split::Lurien => p.old == "Dream_Guardian_Lurien" && p.current == "Cutscene_Boss_Door",
        Split::Monomon => p.old == "Dream_Guardian_Monomon" && p.current == "Cutscene_Boss_Door",
        Split::Hegemol => p.old == "Dream_Guardian_Hegemol" && p.current == "Cutscene_Boss_Door",
        */
        Split::MenuDreamer3 => 3 <= pds.guardians_defeated(prc, g) && is_menu(p.current),
        // endregion: Dreamers

        // region: Dirtmouth
        Split::KingsPass => p.old == "Tutorial_01" && p.current == "Town",
        Split::EnterDirtmouth => p.current == "Town" && p.current != p.old,
        Split::SlyShopExit => p.old == "Room_shop" && p.current != p.old,
        Split::LumaflyLanternTransition => pds.has_lantern(prc, g) && !p.current.starts_with("Room_shop"),
        // TODO: should EnterTMG check that Grimmchild is actually equipped?
        Split::EnterTMG => p.current.starts_with("Grimm_Main_Tent")
                        && p.current != p.old
                        && g.grimm_child_level(prc).is_some_and(|l| l == 2)
                        && g.flames_collected(prc).is_some_and(|f| 3 <= f),
        Split::EnterNKG => p.old.starts_with("Grimm_Main_Tent") && p.current.starts_with("Grimm_Nightmare"),
        // endregion: Dirtmouth
        // region: Crossroads
        Split::EnterBroodingMawlek => p.current == "Crossroads_09" && p.current != p.old,
        Split::AncestralMound => p.current == "Crossroads_ShamanTemple" && p.current != p.old,
        Split::SalubraExit => p.old == "Room_Charm_Shop" && p.current != p.old,
        Split::EnterHollowKnight => p.current == "Room_Final_Boss_Core" && p.current != p.old,
        Split::HollowKnightDreamnail => p.current.starts_with("Dream_Final") && p.current != p.old,
        // endregion: Crossroads
        // region: Greenpath
        Split::EnterGreenpath => p.current.starts_with("Fungus1_01") && !p.old.starts_with("Fungus1_01"),
        Split::VengeflyKingTrans => pds.zote_rescued_buzzer(prc, g) && p.current != p.old,
        Split::EnterHornet1 => p.current.starts_with("Fungus1_04") && p.current != p.old,
        Split::MenuCloak => pds.has_dash(prc, g) && is_menu(p.current),
        Split::MegaMossChargerTrans => pds.mega_moss_charger_defeated(prc, g) && p.current != p.old,
        // endregion: Greenpath
        // region: Fungal
        Split::FungalWastesEntry => starts_with_any(p.current, FUNGAL_WASTES_ENTRY_SCENES) && p.current != p.old,
        Split::ElderHuTrans => pds.killed_ghost_hu(prc, g) && p.current != p.old,
        Split::MenuDashmaster => pds.got_charm_31(prc, g) && is_menu(p.current),
        Split::MenuClaw => pds.has_wall_jump(prc, g) && is_menu(p.current),
        Split::MenuMantisJournal => is_menu(p.current) && p.old == "Fungus2_17",
        // endregion: Fungal
        // region: Resting Grounds
        Split::BlueLake => p.current.starts_with("Crossroads_50") && !p.old.starts_with("Crossroads_50"), // blue lake is Crossroads_50
        Split::EnterAnyDream => p.current.starts_with("Dream_") && p.current != p.old,
        Split::DreamNailExit => p.old == "Dream_Nailcollection" && p.current == "RestingGrounds_07",
        Split::MenuDreamNail => pds.has_dream_nail(prc, g) && is_menu(p.current),
        Split::MenuDreamGate => pds.has_dream_gate(prc, g) && is_menu(p.current),
        Split::CatacombsEntry => p.current.starts_with("RestingGrounds_10") && !p.old.starts_with("RestingGrounds_10"),
        // endregion: Resting Grounds
        // region: City
        Split::TransGorgeousHusk => pds.killed_gorgeous_husk(prc, g) && p.current != p.old,
        Split::MenuGorgeousHusk => pds.killed_gorgeous_husk(prc, g) && is_menu(p.current),
        Split::EnterRafters => p.current == "Ruins1_03" && p.current != p.old,
        Split::EnterSanctum => p.current.starts_with("Ruins1_23") && !p.old.starts_with("Ruins1_23"),
        Split::EnterSoulMaster => p.current.starts_with("Ruins1_24") && p.current != p.old,
        Split::MenuStoreroomsSimpleKey => is_menu(p.current) && p.old == "Ruins1_17",
        Split::MenuShadeSoul => 2 <= pds.get_fireball_level(prc, g) && is_menu(p.current),
        Split::EnterBlackKnight => p.current == "Ruins2_03" && p.current != p.old,
        Split::BlackKnightTrans => pds.killed_black_knight(prc, g) && p.current != p.old,
        Split::EnterLoveTower => p.current.starts_with("Ruins2_11") && p.current != p.old,
        Split::TransCollector => pds.collector_defeated(prc, g) && p.current != p.old,
        // endregion: City
        // region: Peak
        Split::CrystalPeakEntry => starts_with_any(p.current, CRYSTAL_PEAK_ENTRY_SCENES) && p.current != p.old,
        Split::MenuSlyKey => is_menu(p.current) && p.old == "Mines_11",
        Split::EnterCrown => p.current == "Mines_23" && p.current != p.old,
        Split::TransDescendingDark => 2 <= pds.get_quake_level(prc, g) && p.current != p.old,
        Split::CrystalMoundExit => p.old.starts_with("Mines_35") && p.current != p.old,
        // endregion: Peak
        // region: Waterways
        Split::DungDefenderExit => p.old == "Waterways_05" && p.current == "Abyss_01",
        Split::MenuIsmasTear => pds.has_acid_armour(prc, g) && is_menu(p.current),
        Split::EnterJunkPit => p.current == "GG_Waterways" && p.current != p.old,
        // endregion: Waterways
        // region: Basin
        Split::BasinEntry => p.current.starts_with("Abyss_04") && p.current != p.old,
        Split::Abyss19from18 => p.old == "Abyss_18" && p.current == "Abyss_19",
        Split::BrokenVesselTrans => pds.killed_infected_knight(prc, g) && g.get_health(prc).is_some_and(|h| 0 < h),
        Split::MenuWings => pds.has_double_jump(prc, g) && is_menu(p.current),
        Split::MenuVoidHeart => pds.got_shade_charm(prc, g) && is_menu(p.current),
        // endregion: Basin
        // region: Kingdom's Edge
        Split::HiveEntry => p.current.starts_with("Hive_01") && p.current != p.old,
        Split::EnterHiveKnight => p.current.starts_with("Hive_05") && p.current != p.old,
        Split::EnterHornet2 => p.current.starts_with("Deepnest_East_Hornet") && p.current != p.old,
        // endregion: Kingdom's Edge
        // region: Fog Canyon
        Split::TeachersArchive => p.current.starts_with("Fungus3_archive") && !p.old.starts_with("Fungus3_archive"),
        // endregion: Fog Canyon
        // region: Queen's Gardens
        Split::QueensGardensEntry => starts_with_any(p.current, QUEENS_GARDENS_ENTRY_SCENES) && p.current != p.old,
        // endregion: Queen's Gardens
        // region: Deepnest
        Split::EnterDeepnest => starts_with_any(p.current, DEEPNEST_ENTRY_SCENES) && p.current != p.old,
        Split::EnterNosk => p.current.starts_with("Deepnest_32") && p.current != p.old,
        // endregion: Deepnest
        // region: Godhome
        Split::EnterGodhome => p.current.starts_with("GG_Atrium") && p.current != p.old,
        // endregion: Godhome
        // else
        _ => false
    }
}

pub fn transition_once_splits(s: &Split, p: &Pair<&str>, prc: &Process, g: &GameManagerFinder, pds: &mut PlayerDataStore) -> bool {
    match s {
        // region: Start
        Split::StartNewGame => {
            starting_kings_pass(p, prc, g)
            || (is_menu(p.old) && p.current == GG_ENTRANCE_CUTSCENE)
        },
        Split::StartAnyGame => {
            starting_kings_pass(p, prc, g)
            || (is_menu(p.old) && (p.current == GG_ENTRANCE_CUTSCENE || is_play_scene(p.current)))
        }
        // endregion: Start
        // region: Stags
        Split::RidingStag => p.current == "Cinematic_Stag_travel"
                          && g.travelling(prc).is_some_and(|t| t),
        Split::StagnestStation => p.current == "Cliffs_03"
                               && g.travelling(prc).is_some_and(|t| t)
                               && g.opened_stag_nest(prc).is_some_and(|o| o),
        // endregion: Stags
        // else
        _ => false
    }
}

pub fn continuous_splits(s: &Split, p: &Process, g: &GameManagerFinder, pds: &mut PlayerDataStore) -> bool {
    match s {
        // region: Dreamers
        Split::Lurien => g.mask_broken_lurien(p).is_some_and(|b| b),
        Split::Monomon => g.mask_broken_monomon(p).is_some_and(|b| b),
        Split::Hegemol => g.mask_broken_hegemol(p).is_some_and(|b| b),
        Split::Dreamer1 => g.guardians_defeated(p).is_some_and(|d| 1 <= d),
        Split::Dreamer2 => g.guardians_defeated(p).is_some_and(|d| 2 <= d),
        Split::Dreamer3 => g.guardians_defeated(p).is_some_and(|d| 3 <= d),
        Split::MenuDreamer3 => { pds.guardians_defeated(p, g); false },
        // endregion: Dreamers
        // region: Spell Levels
        Split::VengefulSpirit => g.get_fireball_level(p).is_some_and(|l| 1 <= l),
        Split::ShadeSoul => g.get_fireball_level(p).is_some_and(|l| 2 <= l),
        Split::MenuShadeSoul => { pds.get_fireball_level(p, g); false },
        Split::DesolateDive => g.get_quake_level(p).is_some_and(|l| 1 <= l),
        Split::DescendingDark => g.get_quake_level(p).is_some_and(|l| 2 <= l),
        Split::TransDescendingDark => { pds.get_quake_level(p, g); false },
        Split::HowlingWraiths => g.get_scream_level(p).is_some_and(|l| 1 <= l),
        Split::AbyssShriek => g.get_scream_level(p).is_some_and(|l| 2 <= l),
        // endregion: Spell Levels
        // region: Movement Abilities
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
        // endregion: Movement Abilities
        // region: Nail Arts
        Split::CycloneSlash => g.has_cyclone(p).is_some_and(|s| s),
        // hasUpwardSlash: secretly means Dash Slash, from Oro
        Split::DashSlash => g.has_upward_slash(p).is_some_and(|s| s),
        // hasDashSlash: secretly means Great Slash, from Sheo
        Split::GreatSlash => g.has_dash_slash(p).is_some_and(|s| s),
        // endregion: Nail Arts
        // region: Dream Nail Levels
        Split::DreamNail => g.has_dream_nail(p).is_some_and(|d| d),
        Split::MenuDreamNail => { pds.has_dream_nail(p, g); false },
        Split::DreamGate => g.has_dream_gate(p).is_some_and(|d| d),
        Split::MenuDreamGate => { pds.has_dream_gate(p, g); false },
        Split::DreamNail2 => g.dream_nail_upgraded(p).is_some_and(|d| d),
        // endregion: Dream Nail Levels
        // region: Keys
        Split::CityKey => g.has_city_key(p).is_some_and(|k| k),
        Split::LumaflyLantern => g.has_lantern(p).is_some_and(|l| l),
        Split::LumaflyLanternTransition => { pds.has_lantern(p, g); false },
        Split::OnObtainSimpleKey => pds.incremented_simple_keys(p, g),
        Split::SlyKey => g.has_sly_key(p).is_some_and(|k| k),
        Split::ElegantKey => g.has_white_key(p).is_some_and(|k| k),
        Split::LoveKey => g.has_love_key(p).is_some_and(|k| k),
        Split::PaleLurkerKey => g.got_lurker_key(p).is_some_and(|k| k),
        Split::SlySimpleKey => g.sly_simple_key(p).is_some_and(|k| k),
        Split::KingsBrand => g.has_kings_brand(p).is_some_and(|k| k),
        Split::TramPass => g.has_tram_pass(p).is_some_and(|k| k),
        // endregion: Keys
        // region: Nail and Pale Ore
        Split::NailUpgrade1 => g.nail_smith_upgrades(p).is_some_and(|n| 1 <= n),
        Split::NailUpgrade2 => g.nail_smith_upgrades(p).is_some_and(|n| 2 <= n),
        Split::NailUpgrade3 => g.nail_smith_upgrades(p).is_some_and(|n| 3 <= n),
        Split::NailUpgrade4 => g.nail_smith_upgrades(p).is_some_and(|n| 4 <= n),
        Split::OnObtainPaleOre => pds.incremented_ore(p, g),
        Split::Ore1 => g.ore_gross(p).is_some_and(|o| 1 <= o),
        Split::Ore2 => g.ore_gross(p).is_some_and(|o| 2 <= o),
        Split::Ore3 => g.ore_gross(p).is_some_and(|o| 3 <= o),
        Split::Ore4 => g.ore_gross(p).is_some_and(|o| 4 <= o),
        Split::Ore5 => g.ore_gross(p).is_some_and(|o| 5 <= o),
        Split::Ore6 => g.ore_gross(p).is_some_and(|o| 6 <= o),
        // endregion: Nail and Pale Ore
        // region: Masks and Mask Shards
        Split::MaskFragment1 => g.max_health_base(p).is_some_and(|h| h == 5) && g.heart_pieces(p).is_some_and(|p| p == 1),
        Split::MaskFragment2 => g.max_health_base(p).is_some_and(|h| h == 5) && g.heart_pieces(p).is_some_and(|p| p == 2),
        Split::MaskFragment3 => g.max_health_base(p).is_some_and(|h| h == 5) && g.heart_pieces(p).is_some_and(|p| p == 3),
        Split::Mask1 => g.max_health_base(p).is_some_and(|h| h == 6),
        // endregion: Masks and Mask Shards
        // region: Charm Notches
        Split::NotchShrumalOgres => g.notch_shroom_ogres(p).is_some_and(|n| n),
        Split::NotchSalubra1 => g.salubra_notch1(p).is_some_and(|n| n),
        Split::NotchSalubra2 => g.salubra_notch2(p).is_some_and(|n| n),
        Split::NotchSalubra3 => g.salubra_notch3(p).is_some_and(|n| n),
        Split::NotchSalubra4 => g.salubra_notch4(p).is_some_and(|n| n),
        Split::NotchFogCanyon => g.notch_fog_canyon(p).is_some_and(|n| n),
        Split::NotchGrimm => g.got_grimm_notch(p).is_some_and(|n| n),
        Split::OnObtainCharmNotch => pds.incremented_charm_slots(p, g),
        // endregion: Charm Notches
        // region: Charms
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
        Split::MenuDashmaster => { pds.got_charm_31(p, g); false },
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
        Split::Flame1 => g.flames_collected(p).is_some_and(|f| 1 <= f),
        Split::Flame2 => g.flames_collected(p).is_some_and(|f| 2 <= f),
        Split::Flame3 => g.flames_collected(p).is_some_and(|f| 3 <= f),
        Split::BrummFlame => g.got_brumms_flame(p).is_some_and(|f| f),
        // Kingsoul / VoidHeart
        Split::WhiteFragmentLeft => g.got_queen_fragment(p).is_some_and(|c| c),
        Split::WhiteFragmentRight => g.got_king_fragment(p).is_some_and(|c| c),
        Split::OnObtainWhiteFragment => pds.increased_royal_charm_state(p, g),
        Split::Kingsoul => g.charm_cost_36(p).is_some_and(|c| c == 5) && g.royal_charm_state(p).is_some_and(|s| s == 3),
        Split::VoidHeart => g.got_shade_charm(p).is_some_and(|c| c),
        Split::MenuVoidHeart => { pds.got_shade_charm(p, g); false },
        // endregion: Charms
        // region: Stags
        Split::StagMoved => pds.changed_stag_position(p, g),
        Split::CrossroadsStation => g.opened_crossroads(p).is_some_and(|o| o),
        Split::GreenpathStation => g.opened_greenpath(p).is_some_and(|o| o),
        Split::QueensStationStation => g.opened_fungal_wastes(p).is_some_and(|o| o),
        Split::StoreroomsStation => g.opened_ruins1(p).is_some_and(|o| o),
        Split::KingsStationStation => g.opened_ruins2(p).is_some_and(|o| o),
        Split::RestingGroundsStation => g.opened_resting_grounds(p).is_some_and(|o| o),
        Split::HiddenStationStation => g.opened_hidden_station(p).is_some_and(|o| o),
        Split::DeepnestStation => g.opened_deepnest(p).is_some_and(|o| o),
        Split::QueensGardensStation => g.opened_royal_gardens(p).is_some_and(|o| o),
        // endregion: Stags
        // region: Relics
        Split::OnObtainWanderersJournal => pds.incremented_trinket1(p, g),
        Split::AllSeals => 17 <= g.trinket2(p).unwrap_or_default() + g.sold_trinket2(p).unwrap_or_default(),
        Split::OnObtainHallownestSeal => pds.incremented_trinket2(p, g),
        Split::SoulSanctumSeal => pds.incremented_trinket2(p, g) && g.get_scene_name(p).is_some_and(|s| s.starts_with("Ruins1_32")),
        Split::OnObtainKingsIdol => pds.incremented_trinket3(p, g),
        Split::GladeIdol => pds.incremented_trinket3(p, g) && g.get_scene_name(p).is_some_and(|s| s.starts_with("RestingGrounds_08")),
        Split::DungDefenderIdol => pds.incremented_trinket3(p, g) && g.get_scene_name(p).is_some_and(|s| s.starts_with("Waterways_15")),
        Split::ArcaneEgg8 => 8 <= g.trinket4(p).unwrap_or_default() + g.sold_trinket4(p).unwrap_or_default(),
        Split::OnObtainArcaneEgg => pds.incremented_trinket4(p, g),
        Split::OnObtainRancidEgg => pds.incremented_rancid_eggs(p, g),
        // endregion: Relics
        // region: Grubs
        Split::Grub1 => g.grubs_collected(p).is_some_and(|g| g == 1),
        Split::Grub2 => g.grubs_collected(p).is_some_and(|g| g == 2),
        Split::Grub3 => g.grubs_collected(p).is_some_and(|g| g == 3),
        Split::Grub4 => g.grubs_collected(p).is_some_and(|g| g == 4),
        Split::Grub5 => g.grubs_collected(p).is_some_and(|g| g == 5),
        // endregion: Grubs
        // region: Dirtmouth
        Split::TroupeMasterGrimm => g.killed_grimm(p).is_some_and(|k| k),
        Split::NightmareKingGrimm => g.killed_nightmare_grimm(p).is_some_and(|k| k),
        Split::GreyPrince => g.killed_grey_prince(p).is_some_and(|k| k),
        Split::GreyPrinceEssence => g.grey_prince_orbs_collected(p).is_some_and(|o| o),
        // endregion: Dirtmouth
        // region: Crossroads
        Split::MenderBug => g.killed_mender_bug(p).is_some_and(|k| k),
        Split::BroodingMawlek => g.killed_mawlek(p).is_some_and(|k| k),
        Split::GruzMother => g.killed_big_fly(p).is_some_and(|f| f),
        Split::SlyRescued => g.sly_rescued(p).is_some_and(|s| s),
        Split::FalseKnight => g.killed_false_knight(p).is_some_and(|k| k),
        Split::FailedKnight => g.false_knight_dream_defeated(p).is_some_and(|k| k),
        Split::FailedChampionEssence => g.false_knight_orbs_collected(p).is_some_and(|o| o),
        Split::UnchainedHollowKnight => g.unchained_hollow_knight(p).is_some_and(|u| u),
        Split::HollowKnightBoss => g.killed_hollow_knight(p).is_some_and(|k| k),
        Split::RadianceBoss => g.killed_final_boss(p).is_some_and(|k| k),
        // endregion: Crossroads
        // region: Greenpath
        Split::MossKnight => g.killed_moss_knight(p).is_some_and(|k| k),
        Split::Zote1 => g.zote_rescued_buzzer(p).is_some_and(|z| z),
        Split::VengeflyKingTrans => { pds.zote_rescued_buzzer(p, g); false },
        Split::Hornet1 => g.killed_hornet(p).is_some_and(|k| k),
        Split::Aluba => g.killed_lazy_flyer(p).is_some_and(|k| k),
        Split::NoEyes => g.killed_ghost_no_eyes(p).is_some_and(|k| k),
        Split::NoEyesEssence => g.no_eyes_defeated(p).is_some_and(|d| d == 2),
        Split::MegaMossCharger => g.mega_moss_charger_defeated(p).is_some_and(|k| k),
        Split::MegaMossChargerTrans => { pds.mega_moss_charger_defeated(p, g); false },
        // endregion: Greenpath
        // region: Fungal
        Split::ElderHu => g.killed_ghost_hu(p).is_some_and(|k| k),
        Split::ElderHuEssence => g.elder_hu_defeated(p).is_some_and(|d| d == 2),
        Split::ElderHuTrans => { pds.killed_ghost_hu(p, g); false },
        Split::MantisLords => g.defeated_mantis_lords(p).is_some_and(|k| k),
        // endregion: Fungal
        // region: Cliffs
        Split::Gorb => g.killed_ghost_aladar(p).is_some_and(|k| k),
        Split::GorbEssence => g.aladar_slug_defeated(p).is_some_and(|d| d == 2),
        Split::NightmareLantern => g.nightmare_lantern_lit(p).is_some_and(|l| l),
        Split::NightmareLanternDestroyed => g.destroyed_nightmare_lantern(p).is_some_and(|l| l),
        // endregion: Cliffs
        // region: Resting Grounds
        Split::Xero => g.killed_ghost_xero(p).is_some_and(|k| k),
        Split::XeroEssence => g.xero_defeated(p).is_some_and(|d| d == 2),
        // endregion: Resting Grounds
        // region: City
        Split::CityGateOpen => g.opened_city_gate(p).is_some_and(|o| o),
        Split::CityGateAndMantisLords => g.opened_city_gate(p).is_some_and(|o| o) && g.defeated_mantis_lords(p).is_some_and(|k| k),
        Split::GorgeousHusk => pds.killed_gorgeous_husk(p, g),
        Split::TransGorgeousHusk => { pds.killed_gorgeous_husk(p, g); false },
        Split::MenuGorgeousHusk => { pds.killed_gorgeous_husk(p, g); false },
        Split::Lemm2 => g.met_relic_dealer_shop(p).is_some_and(|m| m),
        Split::SoulMasterEncountered => g.mage_lord_encountered(p).is_some_and(|b| b),
        Split::SoulMasterPhase1 => g.mage_lord_encountered_2(p).is_some_and(|b| b),
        Split::SoulMaster => g.killed_mage_lord(p).is_some_and(|k| k),
        Split::SoulTyrant => g.mage_lord_dream_defeated(p).is_some_and(|k| k),
        Split::SoulTyrantEssence => g.mage_lord_orbs_collected(p).is_some_and(|o| o),
        Split::WatcherChandelier => g.watcher_chandelier(p).is_some_and(|c| c),
        Split::BlackKnight => g.killed_black_knight(p).is_some_and(|k| k),
        Split::BlackKnightTrans => { pds.killed_black_knight(p, g); false },
        Split::Collector => g.collector_defeated(p).is_some_and(|k| k),
        Split::TransCollector => { pds.collector_defeated(p, g); false },
        // endregion: City
        // region: Peak
        Split::HuskMiner => pds.decremented_kills_zombie_miner(p, g),
        Split::CrystalGuardian1 => g.defeated_mega_beam_miner(p).is_some_and(|k| k),
        Split::MineLiftOpened => g.mine_lift_opened(p).is_some_and(|o| o),
        // endregion: Peak
        // region: Waterways
        Split::DungDefender => g.killed_dung_defender(p).is_some_and(|k| k),
        Split::WhiteDefender => g.killed_white_defender(p).is_some_and(|k| k),
        Split::WhiteDefenderEssence => g.white_defender_orbs_collected(p).is_some_and(|o| o),
        Split::Flukemarm => g.killed_fluke_mother(p).is_some_and(|k| k),
        Split::BrokenVessel => g.killed_infected_knight(p).is_some_and(|k| k),
        Split::BrokenVesselTrans => { pds.killed_infected_knight(p, g); false },
        Split::LostKin => g.infected_knight_dream_defeated(p).is_some_and(|k| k),
        Split::LostKinEssence => g.infected_knight_orbs_collected(p).is_some_and(|o| o),
        // endregion: Waterways
        // region: Kingdom's Edge
        Split::HiveKnight => g.killed_hive_knight(p).is_some_and(|k| k),
        Split::GreatHopper => g.killed_giant_hopper(p).is_some_and(|k| k),
        Split::Hornet2 => g.hornet_outskirts_defeated(p).is_some_and(|k| k),
        Split::Markoth => g.killed_ghost_markoth(p).is_some_and(|k| k),
        Split::MarkothEssence => g.markoth_defeated(p).is_some_and(|d| d == 2),
        Split::ZoteKilled => g.killed_zote(p).is_some_and(|k| k),
        Split::GodTamer => g.killed_lobster_lancer(p).is_some_and(|k| k),
        // endregion: Kingdom's Edge
        // region: Fog Canyon
        Split::UumuuEncountered => g.encountered_mega_jelly(p).is_some_and(|b| b),
        Split::Uumuu => g.killed_mega_jellyfish(p).is_some_and(|k| k),
        // endregion: Fog Canyon
        // region: Queen's Gardens
        Split::Marmu => g.killed_ghost_marmu(p).is_some_and(|k| k),
        Split::MarmuEssence => g.mum_caterpillar_defeated(p).is_some_and(|d| d == 2),
        Split::TraitorLord => g.killed_traitor_lord(p).is_some_and(|k| k),
        // endregion: Queen's Gardens
        // region: Deepnest
        Split::Zote2 => g.zote_rescued_deepnest(p).is_some_and(|z| z),
        Split::Nosk => g.killed_mimic_spider(p).is_some_and(|k| k),
        Split::Galien => g.killed_ghost_galien(p).is_some_and(|k| k),
        Split::GalienEssence => g.galien_defeated(p).is_some_and(|d| d == 2),
        Split::BeastsDenTrapBench => g.spider_capture(p).is_some_and(|c| c),
        // endregion: Deepnest
        // region: Godhome
        Split::MatoOroNailBros => g.killed_nail_bros(p).is_some_and(|k| k),
        Split::SheoPaintmaster => g.killed_paintmaster(p).is_some_and(|k| k),
        Split::SlyNailsage => g.killed_nailsage(p).is_some_and(|k| k),
        Split::PureVessel => g.killed_hollow_knight_prime(p).is_some_and(|k| k),
        // endregion: Godhome
        // else
        _ => false
    }
}

pub fn splits(s: &Split, prc: &Process, g: &GameManagerFinder, trans_now: bool, ss: &mut SceneStore, pds: &mut PlayerDataStore) -> bool {
    #[cfg(debug_assertions)]
    pds.get_game_state(prc, g);
    let b = continuous_splits(s, prc, g, pds)
        || {
            let pair = ss.pair();
            (!ss.split_this_transition && transition_once_splits(s, &pair, prc, g, pds))
            || (trans_now && transition_splits(s, &pair, prc, g, pds))
        };
    if b { ss.split_this_transition = true; }
    b
}

fn starting_kings_pass(p: &Pair<&str>, prc: &Process, g: &GameManagerFinder) -> bool {
    OPENING_SCENES.contains(&p.old)
    && p.current == "Tutorial_01"
    && g.get_game_state(prc).is_some_and(|gs| {
        gs == GAME_STATE_ENTERING_LEVEL
    })
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

pub fn splits_from_settings<S: Settings>(s: &S) -> Vec<Split> {
    let maybe_ordered = s.dict_get("Ordered");
    let maybe_start = s.dict_get("AutosplitStartRuns");
    let maybe_end = s.dict_get("AutosplitEndRuns");
    let maybe_splits = s.dict_get("Splits");
    if maybe_ordered.is_some() || maybe_start.is_some() || maybe_end.is_some() {
        // Splits files from up through version 3 of ShootMe/LiveSplit.HollowKnight
        let start = maybe_start.and_then(Split::from_settings_str).unwrap_or(Split::StartNewGame);
        let end = maybe_end.and_then(|s| s.as_bool()).unwrap_or_default();
        let mut result = vec![start];
        if let Some(splits) = maybe_splits {
            result.append(&mut splits_from_settings_split_list(&splits));
        }
        if !end {
            result.push(Split::EndingSplit);
        }
        result
    } else if let Some(splits) = maybe_splits {
        // Splits files from after version 4 of mayonnaisical/LiveSplit.HollowKnight
        splits_from_settings_split_list(&splits)
    } else {
        default_splits()
    }
}

fn splits_from_settings_split_list<S: Settings>(s: &S) -> Vec<Split> {
    s.as_list().unwrap_or_default().into_iter().filter_map(Split::from_settings_split).collect()
}
