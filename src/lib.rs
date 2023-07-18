// #![no_std]

use std::string::String;
use asr::future::{next_tick, retry};
use asr::Process;
use asr::game_engine::unity::{SceneManager, get_scene_name};
use asr::string::ArrayCString;
use asr::timer::TimerState;
// use asr::time::Duration;

asr::async_main!(stable);
// asr::panic_handler!();

const SCENE_PATH_SIZE: usize = 64;

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
                let mut scene_name = get_scene_name_string(wait_get_current_scene_path::<SCENE_PATH_SIZE>(&process, &scene_manager).await);
                on_new_scene(&scene_name);
                loop {
                    // TODO: Do something on every tick.
                    if let Ok(next_scene_name) = scene_manager.get_current_scene_path::<SCENE_PATH_SIZE>(&process).map(get_scene_name_string) {
                        if next_scene_name != scene_name {
                            scene_name = next_scene_name;
                            on_new_scene(&scene_name);
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

fn on_new_scene(scene_name: &str) {
    asr::print_message(scene_name);
    match asr::timer::state() {
        TimerState::NotRunning => {
            if scene_name == "Tutorial_01" {
                asr::timer::start();
            }
        }
        TimerState::Running => {
            if scene_name.starts_with("Cinematic_Ending") {
                asr::timer::split();
            }
        }
        _ => ()
    }
}
