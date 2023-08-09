// #![no_std]

mod splits;

use std::string::String;
use std::mem::MaybeUninit;
use asr::future::{next_tick, retry};
use asr::{Process, Address, Address64};
use asr::game_engine::unity::{SceneManager, get_scene_name};
use asr::string::{ArrayCString, ArrayWString};
// use asr::time::Duration;
// use asr::timer::TimerState;
use asr::watcher::Pair;

use memchr;
use memchr::memmem::Finder;

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

const GEO_POOL: i32 = 355335698;
const GEO_POOL_OFFSET: u64 = 0x248;
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
                asr::print_message("Scanning for geo pool roots...");
                next_tick().await;
                let maybe_root = attempt_scan_geo_pool_roots(&process);
                asr::print_message(&format!("maybe_root: {:?}", maybe_root));
                next_tick().await;
                asr::print_message("Scanning for dream_return_scene roots...");
                next_tick().await;
                let maybe_root2 = attempt_scan_dream_return_scene_roots(&process);
                asr::print_message(&format!("maybe_root2: {:?}", maybe_root2));
                next_tick().await;
                if maybe_root.is_none() {
                    asr::print_message(&format!("{:?}", attempt_scan_geo_pool_leaves(&process).await));
                    next_tick().await;
                }

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

fn scan_memory_range(process: &Process, range: (Address, u64), finder: &Finder<'_>) -> Vec<Address> {
    let mut rs: Vec<Address> = vec![];
    let (addr, len) = range;
    let mut addr: Address = Into::into(addr);
    // TODO: Handle the case where a signature may be cut in half by a page
    // boundary.
    let overall_end = addr.value() + len;
    // asr::print_message(&format!("Scanning length {}:\n  {}\n  {}", len, addr, Address::new(overall_end)));
    let mut buf = [MaybeUninit::uninit(); 4 << 10];
    while addr.value() < overall_end {
        // We round up to the 4 KiB address boundary as that's a single
        // page, which is safe to read either fully or not at all. We do
        // this to do a single read rather than many small ones as the
        // syscall overhead is a quite high.
        let end = (addr.value() & !((4 << 10) - 1)) + (4 << 10).min(overall_end);
        let len = end - addr.value();
        let current_read_buf = &mut buf[..len as usize];
        if let Ok(current_read_buf) = process.read_into_uninit_buf(addr, current_read_buf) {
            let haystack = current_read_buf;
            let ps = finder.find_iter(haystack);
            let addr_here = addr;
            rs.extend(ps.map(move |pos| addr_here.add(pos as u64)));
        };
        addr = Address::new(end);
    }
    rs
}

fn scan_unity_player(process: &Process, finder: &Finder<'_>) -> Vec<Address> {
    if let Some(unity_player) = get_unity_player_range(process) {
        scan_memory_range(process, unity_player, &finder)
    } else {
        vec![]
    }
}

async fn scan_all_memory_ranges<'a>(process: &'a Process, finder: &Finder<'_>) -> Vec<Address> {
    let mut rs: Vec<Address> = vec![];
    for mr in process.memory_ranges() {
        if let Ok(r) = mr.range() {
            let addrs = scan_memory_range(process, r, finder);
            next_tick().await;
            if !addrs.is_empty() {
                rs.extend(addrs);
            }
        }
    }
    rs
}

async fn scan_unity_player_first(process: &Process, needle: &[u8]) -> Vec<Address> {
    let mut rs: Vec<Address> = vec![];
    let finder = Finder::new(needle);
    let addrs = scan_unity_player(&process, &finder);
    if !addrs.is_empty() {
        asr::print_message("Found in UnityPlayer");
        rs.extend(addrs);
        return rs;
    } else {
        asr::print_message("Not found in UnityPlayer, scanning other memory ranges...");
    }
    next_tick().await;
    rs.extend(scan_all_memory_ranges(process, &finder).await);
    rs
}

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

fn attach_geo_pool_root(process: &Process, a: Address) -> Option<Address> {
    let g: i32 = process.read_pointer_path64(a, &[0, 0x10, 0x80, 0x28, 0x38, PLAYER_DATA_OFFSET, GEO_POOL_OFFSET]).ok()?;
    if g == GEO_POOL {
        Some(a)
    } else {
        None
    }
}

fn attempt_scan_geo_pool_roots(process: &Process) -> Option<Address> {
    let unity_player = get_unity_player_range(process)?;
    let (addr, len) = unity_player;
    for i in 0 .. (len / 8) {
        let a = addr.add(i * 8);
        if let Some(a) = attach_geo_pool_root(process, a) {
            let offset = a.value() - addr.value();
            asr::print_message(&format!("Found UnityPlayer + 0x{:X}", offset));
            return Some(a);
        }
    }
    None
}

async fn attempt_scan_geo_pool_leaves(process: &Process) -> Option<Address> {
    asr::print_message("Searching for geo pool...");
    let mut geo_pool_addrs: Vec<Address> = vec![];
    for geo_pool in [GEO_POOL].into_iter() {
        let needle = bytemuck::bytes_of(&geo_pool);
        geo_pool_addrs.extend(scan_unity_player_first(process, needle).await);
    }
    if geo_pool_addrs.is_empty() { return None; }
    geo_pool_addrs.sort();
    geo_pool_addrs.dedup();
    asr::print_message(&format!("Found geo pool: {}", geo_pool_addrs.len()));
    next_tick().await;
    asr::print_message("Searching for player data has geo pool pointers...");
    if geo_pool_addrs.iter().any(|a| a.to_string().ends_with("48")) {
        asr::print_message("Filtering by ends_with(\"48\") first...");
        geo_pool_addrs = geo_pool_addrs.into_iter().filter(|a| a.to_string().ends_with("48")).collect();
        asr::print_message(&geo_pool_addrs.len().to_string());
    }
    let mut player_data_addrs: Vec<Address> = vec![];
    for geo_pool_addr in geo_pool_addrs {
        let addr64 = Address64::new(geo_pool_addr.value() - GEO_POOL_OFFSET);
        let needle = bytemuck::bytes_of(&addr64);
        player_data_addrs.extend(scan_unity_player_first(process, needle).await);
    }
    if player_data_addrs.is_empty() { return None; }
    player_data_addrs.sort();
    player_data_addrs.dedup();
    asr::print_message(&format!("Found player data has geo pool pointer: {}", player_data_addrs.len()));
    next_tick().await;
    asr::print_message("Searching for has player data pointers...");
    if player_data_addrs.iter().any(|a| a.to_string().ends_with("c8")) {
        asr::print_message("Filtering by ends_with(\"c8\") first...");
        player_data_addrs = player_data_addrs.into_iter().filter(|a| a.to_string().ends_with("c8")).collect();
        asr::print_message(&player_data_addrs.len().to_string());
    }
    let mut has_player_data_addrs: Vec<Address> = vec![];
    for player_data_addr in player_data_addrs {
        let addr64 = Address64::new(player_data_addr.value() - PLAYER_DATA_OFFSET);
        let needle = bytemuck::bytes_of(&addr64);
        has_player_data_addrs.extend(scan_unity_player_first(process, needle).await);
    }
    if has_player_data_addrs.is_empty() { return None; }
    has_player_data_addrs.sort();
    has_player_data_addrs.dedup();
    asr::print_message(&format!("Found has player data pointer: {}", has_player_data_addrs.len()));
    next_tick().await;
    asr::print_message("Searching for has^2 player data pointers...");
    if has_player_data_addrs.iter().any(|a| a.to_string().ends_with("8")) {
        asr::print_message("Filtering by ends_with(\"8\") first...");
        has_player_data_addrs = has_player_data_addrs.into_iter().filter(|a| a.to_string().ends_with("8")).collect();
        asr::print_message(&has_player_data_addrs.len().to_string());
    }
    let mut has_2_player_data_addrs: Vec<Address> = vec![];
    for has_player_data_addr in has_player_data_addrs {
        let addr64 = Address64::new(has_player_data_addr.value() - 0x38);
        let needle = bytemuck::bytes_of(&addr64);
        has_2_player_data_addrs.extend(scan_unity_player_first(process, needle).await);
    }
    if has_2_player_data_addrs.is_empty() { return None; }
    has_2_player_data_addrs.sort();
    has_2_player_data_addrs.dedup();
    asr::print_message(&format!("Found has^2 player data pointer: {}", has_2_player_data_addrs.len()));
    next_tick().await;
    asr::print_message("Searching for has^3 player data pointers...");
    if has_2_player_data_addrs.iter().any(|a| a.to_string().ends_with("8")) {
        asr::print_message("Filtering by ends_with(\"8\") first...");
        has_2_player_data_addrs = has_2_player_data_addrs.into_iter().filter(|a| a.to_string().ends_with("8")).collect();
        asr::print_message(&has_2_player_data_addrs.len().to_string());
    }
    let mut has_3_player_data_addrs: Vec<Address> = vec![];
    for has_player_data_addr in has_2_player_data_addrs {
        let addr64 = Address64::new(has_player_data_addr.value() - 0x28);
        let needle = bytemuck::bytes_of(&addr64);
        has_3_player_data_addrs.extend(scan_unity_player_first(process, needle).await);
    }
    if has_3_player_data_addrs.is_empty() { return None; }
    has_3_player_data_addrs.sort();
    has_3_player_data_addrs.dedup();
    asr::print_message(&format!("Found has^3 player data pointer: {}", has_3_player_data_addrs.len()));
    next_tick().await;
    // 0x80
    asr::print_message("Searching for has^4 player data pointers...");
    let mut has_4_player_data_addrs: Vec<Address> = vec![];
    for has_player_data_addr in has_3_player_data_addrs {
        let addr64 = Address64::new(has_player_data_addr.value() - 0x80);
        let needle = bytemuck::bytes_of(&addr64);
        has_4_player_data_addrs.extend(scan_unity_player_first(process, needle).await);
    }
    if has_4_player_data_addrs.is_empty() { return None; }
    has_4_player_data_addrs.sort();
    has_4_player_data_addrs.dedup();
    asr::print_message(&format!("Found has^4 player data pointer: {}", has_4_player_data_addrs.len()));
    next_tick().await;
    
    // {0x10, 0x18}
    // "UnityPlayer.dll"+019D7CF0
    
    let the_addr = has_4_player_data_addrs[0];
    if let Some((addr, len)) = get_unity_player_range(process) {
        let offset = the_addr.value() - addr.value();
        if offset < len {
            asr::print_message(&format!("  {} = UnityPlayer + 0x{:X}", the_addr, offset));
        }
    }
    Some(the_addr)
}

