// #![no_std]

mod splits;

use std::collections::BTreeMap;
use std::mem::MaybeUninit;
use std::string::String;
use asr::future::{next_tick, retry};
use asr::{Process, Address, MemoryRange, Address64};
use asr::game_engine::unity::{SceneManager, get_scene_name};
use asr::string::ArrayCString;
// use asr::time::Duration;
// use asr::timer::TimerState;
use asr::watcher::Pair;
use memchr;
use memchr::memmem::Finder;
use serde::{Deserialize, Serialize};

asr::async_main!(stable);
// asr::panic_handler!();

const SCENE_PATH_SIZE: usize = 64;

const HOLLOW_KNIGHT_NAMES: [&str; 2] = [
    "hollow_knight.exe", // Windows
    "Hollow Knight", // Mac
];

const SCENE_ASSET_PATH_OFFSET: u64 = 0x10;
const ACTIVE_SCENE_OFFSET: u64 = 0x48;

#[derive(Deserialize, Serialize)]
struct SceneInfo {
    name: String,
    path: String
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
                let scene_manager = SceneManager::wait_attach(&process).await;
                let mut scene_name = get_scene_name_string(wait_get_current_scene_path::<SCENE_PATH_SIZE>(&process, &scene_manager).await);
                let mut scene_table: BTreeMap<i32, SceneInfo> = serde_json::from_str(include_str!("scene_table.json")).unwrap_or_default();
                let si = on_scene(&process, &scene_manager, &mut scene_table);
                on_scan(&process, &scene_table, si).await;

                let splits = splits::default_splits();
                let mut i = 0;
                loop {
                    let current_split = &splits[i];
                    if let Ok(next_scene_name) = scene_manager.get_current_scene_path::<SCENE_PATH_SIZE>(&process).map(get_scene_name_string) {
                        if next_scene_name != scene_name {
                            let si = on_scene(&process, &scene_manager, &mut scene_table);
                            // asr::print_message(&next_scene_name);
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
                            on_scan(&process, &scene_table, si).await;
                        }
                    }
                    next_tick().await;
                }
            })
            .await;
    }
}

async fn wait_get_current_scene_path<const N: usize>(process: &Process, scene_manager: &SceneManager) -> ArrayCString<N> {
    retry(|| scene_manager.get_current_scene_path(&process)).await
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

fn on_scene(process: &Process, scene_manager: &SceneManager, scene_table: &mut BTreeMap<i32, SceneInfo>) -> i32 {
    let si = scene_manager.get_current_scene_index(&process).unwrap_or(-1);
    let sp: ArrayCString<SCENE_PATH_SIZE> = scene_manager.get_current_scene_path(&process).unwrap_or_default();
    let sn = get_scene_name_string(sp);
    scene_table.insert(si, SceneInfo{name: sn.clone(), path: String::from_utf8(sp.to_vec()).unwrap()});
    if sn == "Menu_Title" {
        log_scene_table(scene_table);
    }
    si
}

async fn on_scan(process: &Process, scene_table: &BTreeMap<i32, SceneInfo>, si: i32) {
    asr::print_message("Attempting scan...");
    let scan_result = attempt_scan_scene_path(process, scene_table, si).await;
    asr::print_message("Scan finished.");
    if scan_result == None {
        asr::print_message("None");
    } else {
        asr::print_message("Some");
    }
}

async fn attempt_scan_scene_path(process: &Process, scene_table: &BTreeMap<i32, SceneInfo>, si: i32) -> Option<Address> {
    let SceneInfo { name: _, path } = scene_table.get(&si).unwrap();
    let needle = path.as_bytes();
    asr::print_message("Searching for scene path contents...");
    let scene_path_contents_addrs = scan_unity_player_first(process, needle).await;
    if scene_path_contents_addrs.is_empty() { return None; }
    asr::print_message(&format!("Found scene path contents: {}", scene_path_contents_addrs.len()));
    next_tick().await;
    asr::print_message("Searching for scene path pointers...");
    let mut scene_path_pointer_addrs: Vec<Address> = vec![];
    for scene_path_contents_addr in scene_path_contents_addrs.into_iter() {
        let scene_path_addr64 = Address64::new(scene_path_contents_addr.value());
        let needle = bytemuck::bytes_of(&scene_path_addr64);
        scene_path_pointer_addrs.extend(scan_unity_player_first(process, needle).await);
    }
    if scene_path_pointer_addrs.is_empty() { return None; }
    asr::print_message(&format!("Found scene path pointer: {}", scene_path_pointer_addrs.len()));
    next_tick().await;
    asr::print_message("Searching for scene has asset path pointers...");
    let mut scene_has_asset_path_addrs: Vec<Address> = vec![];
    for scene_path_pointer_addr in scene_path_pointer_addrs {
        let scene_addr64 = Address64::new(scene_path_pointer_addr.value() - SCENE_ASSET_PATH_OFFSET);
        let needle = bytemuck::bytes_of(&scene_addr64);
        scene_has_asset_path_addrs.extend(scan_unity_player_first(process, needle).await);
    }
    if scene_has_asset_path_addrs.is_empty() { return None; }
    asr::print_message(&format!("Found scene has asset path pointer: {}", scene_has_asset_path_addrs.len()));
    next_tick().await;
    asr::print_message("Searching UnityPlayer for has active scene pointer...");
    let mut has_active_scene_addrs: Vec<Address> = vec![];
    for scene_has_asset_path_addr in scene_has_asset_path_addrs.into_iter() {
        let addr64 = Address64::new(scene_has_asset_path_addr.value() - ACTIVE_SCENE_OFFSET);
        let needle = bytemuck::bytes_of(&addr64);
        let finder = Finder::new(needle);
        has_active_scene_addrs.extend(scan_unity_player(process, &finder));
    }
    if has_active_scene_addrs.is_empty() { return None; }
    asr::print_message(&format!("Found has active scene pointer: {}", has_active_scene_addrs.len()));
    let the_addr = has_active_scene_addrs[0];
    if let Ok((addr, len)) = process.get_module_range("UnityPlayer.dll") {
        let offset = the_addr.value() - addr.value();
        if offset < len {
            asr::print_message(&format!("  {} = UnityPlayer + {}", the_addr, offset));
        }
    }
    Some(the_addr)
    /*
    next_tick().await;
    asr::print_message("Searching UnityPlayer for SceneManager address...");
    let mut scene_manager_addrs: Vec<Address> = vec![];
    for has_active_scene_addr in has_active_scene_addrs {
        let addr64 = Address64::new(has_active_scene_addr.value());
        let needle = bytemuck::bytes_of(&addr64);
        let finder = Finder::new(needle);
        scene_manager_addrs.extend(scan_unity_player(process, &finder));
    }
    if scene_manager_addrs.is_empty() { return None; }
    asr::print_message(&format!("Found scene manager address: {}", scene_manager_addrs.len()));
    Some(scene_manager_addrs[0])
    */
}

async fn scan_unity_player_first(process: &Process, needle: &[u8]) -> Vec<Address> {
    let mut rs: Vec<Address> = vec![];
    let finder = Finder::new(needle);
    let addrs = scan_unity_player(&process, &finder);
    if !addrs.is_empty() {
        asr::print_message("Found in UnityPlayer");
        rs.extend(addrs);
        return rs;
    }
    next_tick().await;
    for (_mr, addrs) in scan_all_memory_ranges(process, &finder).await {
        // print_memory_range_info(mr).unwrap_or_default();
        rs.extend(addrs);
    }
    rs
}

fn scan_unity_player(process: &Process, finder: &Finder<'_>) -> Vec<Address> {
    if let Ok(unity_player) = process.get_module_range("UnityPlayer.dll") {
        scan_memory_range(process, unity_player, &finder)
    } else {
        vec![]
    }
}

async fn scan_all_memory_ranges<'a>(process: &'a Process, finder: &Finder<'_>) -> Vec<(MemoryRange<'a>, Vec<Address>)> {
    let mut rs: Vec<(MemoryRange<'a>, Vec<Address>)> = vec![];
    for mr in process.memory_ranges() {
        if let Ok(r) = mr.range() {
            let addrs = scan_memory_range(process, r, finder);
            next_tick().await;
            if !addrs.is_empty() {
                rs.push((mr, addrs));
            }
        }
    }
    rs
}

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

fn _print_memory_range_info(mr: MemoryRange<'_>) -> Result<(), asr::Error> {
    asr::print_message(&format!("memory range:\n  size: {:?}\n  flags: {:?}\n  {:?}\n  {:?}", 
        mr.size()?,
        mr.flags()?,
        mr.address()?,
        mr.range().map(|(a, l)| Address::new(a.value() + l))?));
    Ok(())
}
