// #![no_std]

use std::string::String;
use asr::future::{next_tick, retry};
use asr::Process;
use asr::game_engine::unity::{SceneManager, get_scene_name};
use asr::string::ArrayCString;

asr::async_main!(stable);
// asr::panic_handler!();

async fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        asr::print_message(&panic_info.to_string());
    }));
    // TODO: Set up some general state and settings.

    asr::print_message("Hello, World!");

    loop {
        let process = Process::wait_attach("hollow_knight.exe").await;
        process
            .until_closes(async {
                // TODO: Load some initial information from the process.
                let scene_manager = SceneManager::wait_attach(&process).await;
                let mut scene_name = get_scene_name_string(wait_get_current_scene_path::<32>(&process, &scene_manager).await);
                asr::print_message(&scene_name);
                loop {
                    // TODO: Do something on every tick.
                    let next_scene_name = get_scene_name_string(wait_get_current_scene_path::<32>(&process, &scene_manager).await);
                    if next_scene_name != scene_name {
                        scene_name = next_scene_name;
                        asr::print_message(&scene_name);
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

fn get_scene_name_string(scene_path: ArrayCString<32>) -> String {
    String::from_utf8(get_scene_name(&scene_path).to_vec()).unwrap()
}
