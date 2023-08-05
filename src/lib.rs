// #![no_std]

mod splits;

use std::collections::BTreeMap;
use std::string::String;
use asr::future::{next_tick, retry};
use asr::{Process, Address};
use asr::game_engine::unity::{SceneManager, get_scene_name};
use asr::string::ArrayCString;
// use asr::time::Duration;
// use asr::timer::TimerState;
use asr::watcher::Pair;
use serde::{Deserialize, Serialize};

asr::async_main!(stable);
// asr::panic_handler!();

const SCENE_PATH_SIZE: usize = 64;

const HOLLOW_KNIGHT_NAMES: [&str; 2] = [
    "hollow_knight.exe", // Windows
    "Hollow Knight", // Mac
];

const SCENE_ASSET_PATH_OFFSET: u64 = 0x10;
const SCENE_BUILD_INDEX_OFFSET: u64 = 0x98;
const ACTIVE_SCENE_OFFSET: u64 = 0x48;
const UNITY_PLAYER_HAS_ACTIVE_SCENE_OFFSETS: [u64; 6] = [
    0x01A1AC30, // Windows
    0x01A982E8, // Mac?
    0x01BBE2E8, // Mac?
    0x01AB02E8, // Mac?
    0x01BB42E8, // Mac?
    0x01AAF2E8, // Mac?
];
const UNITY_PLAYER_NAMES: [&str; 2] = [
    "UnityPlayer.dll", // Windows
    "UnityPlayer.dylib", // Mac
];
const ASSETS_SCENES: &str = "Assets/Scenes/";
const ASSETS_SCENES_LEN: usize = ASSETS_SCENES.len();

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct SceneInfo {
    name: String,
    path: String
}

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
    async fn wait_attach(process: &Process, _scene_table: &SceneTable) -> SceneFinder {
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

    fn get_current_scene_name(&self, process: &Process) -> String {
        self.get_current_scene_path::<SCENE_PATH_SIZE>(process).map(get_scene_name_string).unwrap_or("".to_string())
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
                let mut scene_table: SceneTable = serde_json::from_str(include_str!("scene_table.json")).unwrap_or_default();
                asr::print_message("Trying to attach SceneFinder...");
                let mut scene_finder = SceneFinder::wait_attach(&process, &scene_table).await;
                asr::print_message("Attached SceneFinder.");
                let mut scene_name = get_scene_name_string(wait_get_current_scene_path::<SCENE_PATH_SIZE>(&process, &scene_finder).await);
                asr::print_message(&scene_name);

                scene_finder.attempt_scan(&process).await;
                on_scene(&process, &scene_finder, &mut scene_table);

                let splits = splits::default_splits();
                let mut i = 0;
                loop {
                    let current_split = &splits[i];
                    let next_scene_name = scene_finder.get_current_scene_name(&process);
                    if !next_scene_name.is_empty() && next_scene_name != scene_name {
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
                        scene_finder.attempt_scan(&process).await;
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

// --------------------------------------------------------

fn log_scene_table(scene_table: &BTreeMap<i32, SceneInfo>) {
    // Log scene_table as json
    if let Ok(j) = serde_json::to_string_pretty(&scene_table) {
        asr::print_message("begin scene_table.json");
        asr::print_message(&j);
    }
}

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

fn get_unity_player_range(process: &Process) -> Option<(Address, u64)> {
    UNITY_PLAYER_NAMES.into_iter().find_map(|name| {
        process.get_module_range(name).ok()
    })
}
