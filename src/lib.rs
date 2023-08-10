// #![no_std]

mod splits;

use std::cmp::min;
use std::string::String;
use asr::future::{next_tick, retry};
use asr::{Process, Address, Address64};
use asr::game_engine::unity::{SceneManager, get_scene_name};
use asr::string::{ArrayCString, ArrayWString};
// use asr::time::Duration;
// use asr::timer::TimerState;
use asr::watcher::Pair;

#[cfg(debug_assertions)]
use std::collections::BTreeMap;
#[cfg(debug_assertions)]
use serde::{Deserialize, Serialize};

asr::async_main!(stable);
// asr::panic_handler!();

const SCENE_PATH_SIZE: usize = 64;

const STRING_LEN_OFFSET: u64 = 0x10;
const STRING_CONTENTS_OFFSET: u64 = 0x14;

const HOLLOW_KNIGHT_NAMES: [&str; 2] = [
    "hollow_knight.exe", // Windows
    "Hollow Knight", // Mac
];

const SCENE_ASSET_PATH_OFFSET: u64 = 0x10;
#[cfg(debug_assertions)]
const SCENE_BUILD_INDEX_OFFSET: u64 = 0x98;
const ACTIVE_SCENE_OFFSET: u64 = 0x48;
const ACTIVE_SCENE_CONTENTS_PATH: &[u64] = &[0, ACTIVE_SCENE_OFFSET, SCENE_ASSET_PATH_OFFSET, 0];
const UNITY_PLAYER_HAS_ACTIVE_SCENE_OFFSETS: [u64; 10] = [
    0x01A1AC30, // Windows
    0x01A982E8, // Mac?
    0x01AA32E8, // Mac?
    0x01AAF2E8, // Mac?
    0x01AB02E8, // Mac?
    0x01BB32E8, // Mac?
    0x01BB42E8, // Mac?
    0x01BBD2E8, // Mac?
    0x01BBE2E8, // Mac?
    0x01BD82E8, // Mac?
];
const UNITY_PLAYER_NAMES: [&str; 2] = [
    "UnityPlayer.dll", // Windows
    "UnityPlayer.dylib", // Mac
];
const ASSETS_SCENES: &str = "Assets/Scenes/";
const ASSETS_SCENES_LEN: usize = ASSETS_SCENES.len();

const PRE_MENU_INTRO: &str = "Pre_Menu_Intro";

const UNITY_PLAYER_HAS_GAME_MANAGER_OFFSETS: [u64; 5] = [
    0x019D7CF0, // Windows
    0x01BF8A80, // Mac?
    0x01C02A80, // Mac?
    0x01C03A80, // Mac?
    0x01C1DA80, // Mac?
];

const UPHGM_OFFSET_0: u64 = 0;
const UPHGM_OFFSET_1: u64 = 0x10;
const UPHGM_OFFSET_2: u64 = 0x80;
const UPHGM_OFFSET_3: u64 = 0x28;
const UPHGM_OFFSET_4: u64 = 0x38;

const SCENE_NAME_OFFSET: u64 = 0x18;
const NEXT_SCENE_NAME_OFFSET: u64 = 0x20;
const SCENE_NAME_PATH: &[u64] = &[
    UPHGM_OFFSET_0,
    UPHGM_OFFSET_1,
    UPHGM_OFFSET_2,
    UPHGM_OFFSET_3,
    UPHGM_OFFSET_4,
    SCENE_NAME_OFFSET
];
const NEXT_SCENE_NAME_PATH: &[u64] = &[
    UPHGM_OFFSET_0,
    UPHGM_OFFSET_1,
    UPHGM_OFFSET_2,
    UPHGM_OFFSET_3,
    UPHGM_OFFSET_4,
    NEXT_SCENE_NAME_OFFSET
];

#[allow(unused)]
const PLAYER_DATA_OFFSET: u64 = 0xc8;

#[cfg(debug_assertions)]
const GEO_OFFSET: u64 = 0x1c4;
#[cfg(debug_assertions)]
const GEO_PATH: &[u64] = &[
    UPHGM_OFFSET_0,
    UPHGM_OFFSET_1,
    UPHGM_OFFSET_2,
    UPHGM_OFFSET_3,
    UPHGM_OFFSET_4,
    PLAYER_DATA_OFFSET,
    GEO_OFFSET
];

#[cfg(debug_assertions)]
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct SceneInfo {
    name: String,
    path: String
}

#[cfg(debug_assertions)]
type SceneTable = BTreeMap<i32, SceneInfo>;

struct UnityPlayerHasActiveScene(Address);

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

enum SceneFinder {
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
    async fn wait_attach(process: &Process) -> SceneFinder {
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
    fn get_current_scene_index(&self, process: &Process) -> Result<i32, asr::Error> {
        match self {
            SceneFinder::SceneManager(scene_manager) => {
                scene_manager.get_current_scene_index(process)
            }
            SceneFinder::UnityPlayerHasActiveScene(uphas) => {
                uphas.get_current_scene_index(process)
            }
        }
    }

    fn get_current_scene_path<const N: usize>(&self, process: &Process) -> Result<ArrayCString<N>, asr::Error> {
        match self {
            SceneFinder::SceneManager(scene_manager) => {
                scene_manager.get_current_scene_path(process)
            }
            SceneFinder::UnityPlayerHasActiveScene(uphas) => {
                uphas.get_current_scene_path(process)
            }
        }
    }
}

struct GameManagerFinder(Address);

impl GameManagerFinder {
    async fn wait_attach(process: &Process, scene_finder: &SceneFinder) -> GameManagerFinder {
        asr::print_message("Trying to attach GameManagerFinder...");
        let g = retry(|| GameManagerFinder::attach_scan(process, scene_finder)).await;
        asr::print_message("Attached GameManagerFinder.");
        g
    }
    fn attach_scan(process: &Process, scene_finder: &SceneFinder) -> Option<GameManagerFinder> {
        let scene_name = scene_finder.get_current_scene_path::<SCENE_PATH_SIZE>(&process).map(get_scene_name_string).ok()?;
        if scene_name == PRE_MENU_INTRO { return None; }
        let unity_player = get_unity_player_range(process)?;
        GameManagerFinder::attach_unity_player(process, unity_player, &scene_name).or_else(|| {
            GameManagerFinder::attempt_scan_unity_player(process, unity_player, &scene_name)
        })
    }
    fn attach_unity_player(process: &Process, unity_player: (Address, u64), scene_name: &str) -> Option<GameManagerFinder> {
        let (addr, _) = unity_player;
        for offset in UNITY_PLAYER_HAS_GAME_MANAGER_OFFSETS.iter() {
            if let Some(a) = attach_game_manager_scene_name(process, addr.add(*offset), scene_name) {
                return Some(GameManagerFinder(a));
            }
        }
        None
    }
    fn attempt_scan_unity_player(process: &Process, unity_player: (Address, u64), scene_name: &str) -> Option<GameManagerFinder> {
        asr::print_message(&format!("Scanning for game_manager_scene_name {}...", scene_name));
        let a = attempt_scan_roots(process, unity_player, attach_game_manager_scene_name, scene_name)?;
        Some(GameManagerFinder(a))
    }

    fn get_scene_name(&self, process: &Process) -> Option<String> {
        let s = process.read_pointer_path64(self.0, SCENE_NAME_PATH).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, s)
    }

    fn get_next_scene_name(&self, process: &Process) -> Option<String> {
        let s = process.read_pointer_path64(self.0, NEXT_SCENE_NAME_PATH).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, s)
    }

    #[cfg(debug_assertions)]
    fn get_geo(&self, process: &Process) -> Option<i32> {
        process.read_pointer_path64(self.0, GEO_PATH).ok()
    }
}

async fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        asr::print_message(&panic_info.to_string());
    }));

    // TODO: Set up some general state and settings.

    asr::print_message("Hello, World!");

    #[cfg(debug_assertions)]
    let mut scene_table: SceneTable = serde_json::from_str(include_str!("scene_table.json")).unwrap_or_default();

    let splits: Vec<splits::Split> = serde_json::from_str(include_str!("splits.json")).ok().unwrap_or_else(splits::default_splits);

    loop {
        let process = retry(|| {
            HOLLOW_KNIGHT_NAMES.into_iter().find_map(Process::attach)
        }).await;
        process
            .until_closes(async {
                // TODO: Load some initial information from the process.
                let scene_finder = SceneFinder::wait_attach(&process).await;
                let mut curr_scene_name = get_scene_name_string(wait_get_current_scene_path::<SCENE_PATH_SIZE>(&process, &scene_finder).await);
                let mut prev_scene_name = curr_scene_name.clone();
                let mut next_scene_name = "".to_string();
                #[cfg(debug_assertions)]
                asr::print_message(&curr_scene_name);

                next_tick().await;
                let game_manager_finder = GameManagerFinder::wait_attach(&process, &scene_finder).await;

                #[cfg(debug_assertions)]
                asr::print_message(&format!("geo: {:?}", game_manager_finder.get_geo(&process)));
                #[cfg(debug_assertions)]
                on_scene(&process, &scene_finder, &mut scene_table);

                let mut i = 0;
                loop {
                    let current_split = &splits[i];
                    let mut new_data_curr = false;
                    let mut new_data_next = false;
                    if let Some(csn) = game_manager_finder.get_scene_name(&process) {
                        if csn != curr_scene_name {
                            prev_scene_name = curr_scene_name;
                            curr_scene_name = csn;
                            if curr_scene_name != next_scene_name { new_data_curr = true; }
                            #[cfg(debug_assertions)]
                            asr::print_message(&format!("curr_scene_name: {}", curr_scene_name));
                        }
                    }
                    if let Some(nsn) = game_manager_finder.get_next_scene_name(&process) {
                        if nsn != next_scene_name {
                            next_scene_name = nsn;
                            new_data_next = !next_scene_name.is_empty();
                            #[cfg(debug_assertions)]
                            asr::print_message(&format!("next_scene_name: {}", next_scene_name));
                        }
                    }
                    if new_data_next || new_data_curr {
                        let scene_pair: Pair<&str> = if new_data_next {
                            Pair{old: &curr_scene_name, current: &next_scene_name}
                        } else {
                            Pair{old: &prev_scene_name, current: &curr_scene_name}
                        };
                        if splits::transition_splits(current_split, &scene_pair) {
                            if i == 0 {
                                asr::timer::start();
                            } else {
                                asr::timer::split();
                            }
                            i += 1;
                            if splits.len() <= i {
                                i = 0;
                            }
                        }

                        #[cfg(debug_assertions)]
                        asr::print_message(&format!("{} -> {}", scene_pair.old, scene_pair.current));
                        #[cfg(debug_assertions)]
                        asr::print_message(&format!("geo: {:?}", game_manager_finder.get_geo(&process)));
                        #[cfg(debug_assertions)]
                        on_scene(&process, &scene_finder, &mut scene_table);
                    }
                    next_tick().await;
                }
            })
            .await;
    }
}

async fn wait_get_current_scene_path<const N: usize>(process: &Process, scene_finder: &SceneFinder) -> ArrayCString<N> {
    retry(|| scene_finder.get_current_scene_path(&process)).await
}

fn get_scene_name_string<const N: usize>(scene_path: ArrayCString<N>) -> String {
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

// Logging in debug_assertions mode

#[cfg(debug_assertions)]
fn log_scene_table(scene_table: &BTreeMap<i32, SceneInfo>) {
    // Log scene_table as json
    if let Ok(j) = serde_json::to_string_pretty(&scene_table) {
        asr::print_message(&format!("begin scene_table.json\n{}", j));
    }
}

#[cfg(debug_assertions)]
fn on_scene(process: &Process, scene_finder: &SceneFinder, scene_table: &mut BTreeMap<i32, SceneInfo>) {
    let si = scene_finder.get_current_scene_index(&process).unwrap_or(-1);
    let sp: ArrayCString<SCENE_PATH_SIZE> = scene_finder.get_current_scene_path(&process).unwrap_or_default();
    let sn = get_scene_name_string(sp);
    let sv = SceneInfo{name: sn.clone(), path: String::from_utf8(sp.to_vec()).unwrap()};
    if let Some(tv) = scene_table.get(&si) {
        assert_eq!(&sv, tv);
    } else {
        scene_table.insert(si, sv);
        log_scene_table(scene_table);
    }
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
    let s0: Address64 = process.read_pointer_path64(a, SCENE_NAME_PATH).ok()?;
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
