// #![no_std]

mod splits;

use std::string::String;
use asr::future::{next_tick, retry};
use asr::{Process, Address};
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

const HOLLOW_KNIGHT_NAMES: [&str; 2] = [
    "hollow_knight.exe", // Windows
    "Hollow Knight", // Mac
];

const SCENE_ASSET_PATH_OFFSET: u64 = 0x10;
#[cfg(debug_assertions)]
const SCENE_BUILD_INDEX_OFFSET: u64 = 0x98;
const ACTIVE_SCENE_OFFSET: u64 = 0x48;
const UNITY_PLAYER_HAS_ACTIVE_SCENE_OFFSETS: [u64; 7] = [
    0x01A1AC30, // Windows
    0x01A982E8, // Mac?
    0x01BBE2E8, // Mac?
    0x01AB02E8, // Mac?
    0x01BB42E8, // Mac?
    0x01AAF2E8, // Mac?
    0x01AA32E8, // Mac?
];
const UNITY_PLAYER_NAMES: [&str; 2] = [
    "UnityPlayer.dll", // Windows
    "UnityPlayer.dylib", // Mac
];
const ASSETS_SCENES: &str = "Assets/Scenes/";
const ASSETS_SCENES_LEN: usize = ASSETS_SCENES.len();

const PLAYER_DATA_OFFSET: u64 = 0xc8;

const DREAM_RETURN_SCENE_OFFSET: u64 = 0x58;
const STRING_CONTENTS_OFFSET: u64 = 0x14;
const DREAM_RETURN_SCENE_LEN: usize = "Dream_NailCollection".len();
const DREAM_RETURN_SCENE_PATH: &[u64] = &[
    0,
    0x10,
    0x80,
    0x28,
    0x38,
    PLAYER_DATA_OFFSET,
    DREAM_RETURN_SCENE_OFFSET,
    STRING_CONTENTS_OFFSET
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
    fn attach_address(process: &Process, a: Address) -> Option<UnityPlayerHasActiveScene> {
        let s1: ArrayCString<ASSETS_SCENES_LEN> = process.read_pointer_path64(a, &[0, ACTIVE_SCENE_OFFSET, SCENE_ASSET_PATH_OFFSET, 0]).ok()?;
        let s2: String = String::from_utf8(s1.to_vec()).ok()?;
        if s2 == ASSETS_SCENES {
            Some(UnityPlayerHasActiveScene(a))
        } else {
            None
        }
    }
    fn attach_unity_player(process: &Process, unity_player: (Address, u64)) -> Option<UnityPlayerHasActiveScene> {
        let (addr, _) = unity_player;
        for offset in UNITY_PLAYER_HAS_ACTIVE_SCENE_OFFSETS.iter() {
            if let Some(uphas) = UnityPlayerHasActiveScene::attach_address(process, addr.add(*offset)) {
                return Some(uphas);
            }
        }
        None
    }
    fn attempt_scan_unity_player(process: &Process, unity_player: (Address, u64)) -> Option<UnityPlayerHasActiveScene> {
        let (addr, len) = unity_player;
        for i in 0 .. (len / 8) {
            let a = addr.add(i * 8);
            if let Some(uphas) = UnityPlayerHasActiveScene::attach_address(process, a) {
                let offset = a.value() - addr.value();
                asr::print_message(&format!("Found UnityPlayer + 0x{:X}", offset));
                return Some(uphas);
            }
        }
        None
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
    SceneManager(SceneManager, Box<Option<UnityPlayerHasActiveScene>>),
    UnityPlayerHasActiveScene(UnityPlayerHasActiveScene)
}

impl SceneFinder {
    fn attach(process: &Process) -> Option<SceneFinder> {
        if let Some(scene_manager) = SceneManager::attach(process) {
            return Some(SceneFinder::SceneManager(scene_manager, Box::new(None)));
        }
        if let Some(uphas) = UnityPlayerHasActiveScene::attach_scan(process) {
            return Some(SceneFinder::UnityPlayerHasActiveScene(uphas))
        }
        None
    }
    async fn wait_attach(process: &Process) -> SceneFinder {
        let mut fuel = 1000;
        let maybe_scene_manager = retry(|| {
            if 0 < fuel {
                fuel -= 1;
                SceneManager::attach(&process).map(Some)
            } else {
                Some(None)
            }
        }).await;
        if let Some(scene_manager) = maybe_scene_manager {
            return SceneFinder::SceneManager(scene_manager, Box::new(None))
        }
        retry(|| SceneFinder::attach(&process)).await
    }

    #[cfg(debug_assertions)]
    async fn attempt_scan(&mut self, process: &Process) {
        match self {
            SceneFinder::SceneManager(_, b) => {
                if b.is_none() {
                    if let Some(uphas) = UnityPlayerHasActiveScene::attach_scan(process) {
                        **b = Some(uphas);
                        asr::print_message("And now with both.");
                    }
                }
            }
            _ => ()
        }
    }

    #[cfg(debug_assertions)]
    fn get_current_scene_index(&self, process: &Process) -> Result<i32, asr::Error> {
        match self {
            SceneFinder::SceneManager(scene_manager, muphas) => {
                let i = scene_manager.get_current_scene_index(process)?;
                if let Some(uphas) = muphas.as_ref() {
                    assert_eq!(uphas.get_current_scene_index(process).expect("uphas get_current_scene_index"), i);
                }
                Ok(i)
            }
            SceneFinder::UnityPlayerHasActiveScene(uphas) => {
                uphas.get_current_scene_index(process)
            }
        }
    }

    fn get_current_scene_path<const N: usize>(&self, process: &Process) -> Result<ArrayCString<N>, asr::Error> {
        match self {
            SceneFinder::SceneManager(scene_manager, muphas) => {
                let p = scene_manager.get_current_scene_path::<N>(process)?;
                if let Some(uphas) = muphas.as_ref() {
                    assert_eq!(uphas.get_current_scene_path::<N>(process).expect("uphas get_current_scene_path").as_bytes(), p.as_bytes());
                }
                Ok(p)
            }
            SceneFinder::UnityPlayerHasActiveScene(uphas) => {
                uphas.get_current_scene_path(process)
            }
        }
    }
}

async fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        asr::print_message(&panic_info.to_string());
    }));

    // TODO: Set up some general state and settings.

    asr::print_message("Hello, World!");

    loop {
        let process = retry(|| {
            HOLLOW_KNIGHT_NAMES.into_iter().find_map(Process::attach)
        }).await;
        process
            .until_closes(async {
                // TODO: Load some initial information from the process.
                #[cfg(debug_assertions)]
                let mut scene_table: SceneTable = serde_json::from_str(include_str!("scene_table.json")).unwrap_or_default();

                asr::print_message("Trying to attach SceneFinder...");
                next_tick().await;
                #[allow(unused_mut)]
                let mut scene_finder = SceneFinder::wait_attach(&process).await;
                asr::print_message("Attached SceneFinder.");
                let mut scene_name = get_scene_name_string(wait_get_current_scene_path::<SCENE_PATH_SIZE>(&process, &scene_finder).await);
                asr::print_message(&scene_name);

                next_tick().await;
                asr::print_message("Scanning for dream_return_scene roots...");
                next_tick().await;
                let maybe_root2 = attempt_scan_dream_return_scene_roots(&process);
                asr::print_message(&format!("maybe_root2: {:?}", maybe_root2));
                next_tick().await;

                #[cfg(debug_assertions)]
                scene_finder.attempt_scan(&process).await;
                #[cfg(debug_assertions)]
                on_scene(&process, &scene_finder, &mut scene_table);

                let splits = serde_json::from_str(include_str!("splits.json")).ok().unwrap_or_else(splits::default_splits);
                let mut i = 0;
                loop {
                    let current_split = &splits[i];
                    if let Ok(next_scene_name) = scene_finder.get_current_scene_path::<SCENE_PATH_SIZE>(&process).map(get_scene_name_string) {
                        if next_scene_name != scene_name {
                            #[cfg(debug_assertions)]
                            asr::print_message(&next_scene_name);

                            let scene_pair: Pair<&str> = Pair{old: &scene_name.clone(), current: &next_scene_name.clone()};
                            scene_name = next_scene_name;
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
                            scene_finder.attempt_scan(&process).await;
                            #[cfg(debug_assertions)]
                            on_scene(&process, &scene_finder, &mut scene_table);
                        }
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

// --------------------------------------------------------

// Logging in debug_assertions mode

#[cfg(debug_assertions)]
fn log_scene_table(scene_table: &BTreeMap<i32, SceneInfo>) {
    // Log scene_table as json
    if let Ok(j) = serde_json::to_string_pretty(&scene_table) {
        asr::print_message("begin scene_table.json");
        asr::print_message(&j);
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

fn attach_dream_return_scene_root(process: &Process, a: Address) -> Option<Address> {
    let s1: ArrayWString<DREAM_RETURN_SCENE_LEN> = process.read_pointer_path64(a, DREAM_RETURN_SCENE_PATH).ok()?;
    let s2: String = String::from_utf16(&s1.to_vec()).ok()?;
    if s2.is_empty() { return None; }
    for b in s2.as_bytes() {
        let c = char::from_u32(*b as u32)?;
        if !(c.is_ascii_alphanumeric() || c.is_ascii_punctuation()) {
            return None;
        }
    }
    Some(a)
}

fn attempt_scan_dream_return_scene_roots(process: &Process) -> Option<Address> {
    let unity_player = get_unity_player_range(process)?;
    let (addr, len) = unity_player;
    for i in 0 .. (len / 8) {
        let a = addr.add(i * 8);
        if let Some(a) = attach_dream_return_scene_root(process, a) {
            let offset = a.value() - addr.value();
            asr::print_message(&format!("Found UnityPlayer + 0x{:X}", offset));
            return Some(a);
        }
    }
    None
}
