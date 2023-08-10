
use std::cmp::min;
use asr::future::{next_tick, retry};
use asr::{Process, Address, Address64};
use asr::game_engine::unity::{SceneManager, get_scene_name};
use asr::string::{ArrayCString, ArrayWString};

// --------------------------------------------------------

const HOLLOW_KNIGHT_NAMES: [&str; 2] = [
    "hollow_knight.exe", // Windows
    "Hollow Knight", // Mac
];

const UNITY_PLAYER_NAMES: [&str; 2] = [
    "UnityPlayer.dll", // Windows
    "UnityPlayer.dylib", // Mac
];

pub const SCENE_PATH_SIZE: usize = 64;

const STRING_LEN_OFFSET: u64 = 0x10;
const STRING_CONTENTS_OFFSET: u64 = 0x14;

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

// --------------------------------------------------------

pub struct UnityPlayerHasActiveScene(Address);

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

pub enum SceneFinder {
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
    pub async fn wait_attach(process: &Process) -> SceneFinder {
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
    pub fn get_current_scene_index(&self, process: &Process) -> Result<i32, asr::Error> {
        match self {
            SceneFinder::SceneManager(scene_manager) => {
                scene_manager.get_current_scene_index(process)
            }
            SceneFinder::UnityPlayerHasActiveScene(uphas) => {
                uphas.get_current_scene_index(process)
            }
        }
    }

    pub fn get_current_scene_path<const N: usize>(&self, process: &Process) -> Result<ArrayCString<N>, asr::Error> {
        match self {
            SceneFinder::SceneManager(scene_manager) => {
                scene_manager.get_current_scene_path(process)
            }
            SceneFinder::UnityPlayerHasActiveScene(uphas) => {
                uphas.get_current_scene_path(process)
            }
        }
    }

    async fn wait_get_current_scene_path<const N: usize>(&self, process: &Process) -> ArrayCString<N> {
        retry(|| self.get_current_scene_path(&process)).await
    }

    pub async fn wait_get_current_scene_name(&self, process: &Process) -> String {
        scene_path_to_name_string(self.wait_get_current_scene_path::<SCENE_PATH_SIZE>(&process).await)
    }
}

pub struct GameManagerFinder(Address);

impl GameManagerFinder {
    pub async fn wait_attach(process: &Process, scene_finder: &SceneFinder) -> GameManagerFinder {
        asr::print_message("Trying to attach GameManagerFinder...");
        let g = retry(|| GameManagerFinder::attach_scan(process, scene_finder)).await;
        asr::print_message("Attached GameManagerFinder.");
        g
    }
    fn attach_scan(process: &Process, scene_finder: &SceneFinder) -> Option<GameManagerFinder> {
        let scene_name = scene_finder.get_current_scene_path::<SCENE_PATH_SIZE>(&process).map(scene_path_to_name_string).ok()?;
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

    pub fn get_scene_name(&self, process: &Process) -> Option<String> {
        let s = process.read_pointer_path64(self.0, SCENE_NAME_PATH).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, s)
    }

    pub fn get_next_scene_name(&self, process: &Process) -> Option<String> {
        let s = process.read_pointer_path64(self.0, NEXT_SCENE_NAME_PATH).ok()?;
        read_string_object::<SCENE_PATH_SIZE>(process, s)
    }

    #[cfg(debug_assertions)]
    pub fn get_geo(&self, process: &Process) -> Option<i32> {
        process.read_pointer_path64(self.0, GEO_PATH).ok()
    }
}

// --------------------------------------------------------

pub async fn wait_attach_hollow_knight() -> Process {
    retry(|| {
        HOLLOW_KNIGHT_NAMES.into_iter().find_map(Process::attach)
    }).await
}

pub fn scene_path_to_name_string<const N: usize>(scene_path: ArrayCString<N>) -> String {
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
