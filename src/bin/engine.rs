#[path = "../player.rs"]
mod player;

use anyhow::{Context, Result};
use std::env;

fn main() -> Result<()> {
    // 1. Read and parse arguments robustly
    let args: Vec<String> = env::args().collect();
    let always_on_top = args.contains(&String::from("--ontop"));
    // Find the first argument that isn't the executable name or the flag
    let url = args.into_iter().skip(1).find(|arg| arg != "--ontop").expect("No URL provided");

    // 2. Locate the native config folder (mpv_config is automatically placed alongside the exe by build.rs)
    let mut exe_path = env::current_exe().expect("Failed to get executable path");
    exe_path.pop(); // Remove playku-engine.exe from the path
    
    let config_dir = exe_path.join("mpv_config");
    let safe_config_path = config_dir.to_str().unwrap().replace("\\", "/");

    println!("Initializing media player engine...");
    
    // 3. Initialize player with native config
    let mut player = player::MpvPlayer::new(&safe_config_path).context("Failed to initialize player")?;
    
    // Apply always on top property dynamically (overrides any default/OS behavior)
    player.set_property("ontop", always_on_top).context("Failed to set ontop property")?;

    // 4. Play the URL directly
    println!("Playing: {}", url);
    player.play(&url)?;
    
    // 5. Natively block the thread (0% CPU) until the video window is closed
    player.wait();
    
    println!("Playback closed. Spawning playku-ui and shutting down engine...");

    // 6. Spawn the UI launcher as a detached process
    let mut exe_path = env::current_exe().expect("Failed to get executable path");
    exe_path.pop();
    
    let ui_binary = if cfg!(target_os = "windows") {
        "playku-ui.exe"
    } else {
        "playku-ui"
    };
    let ui_path = exe_path.join(ui_binary);

    std::process::Command::new(&ui_path)
        .spawn()
        .expect("Failed to restart UI");

    Ok(())
}
