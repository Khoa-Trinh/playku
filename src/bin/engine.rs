#![windows_subsystem = "windows"]

use playku::player::MpvPlayer;
use anyhow::{Context, Result};
use std::env;

fn main() -> Result<()> {
    // 1. Read and parse arguments robustly
    let args: Vec<String> = env::args().collect();
    
    // Check CLI flags
    let mut always_on_top = args.contains(&String::from("--ontop")) || args.contains(&String::from("-o"));
    let mut audio_only = args.contains(&String::from("--audio")) || args.contains(&String::from("--audio-only")) || args.contains(&String::from("-a"));

    // Find the first argument that is the URL (not one of our flags or exe name)
    let mut raw_url = args.into_iter().skip(1).find(|arg| {
        arg != "--ontop" && arg != "-o" && arg != "--audio" && arg != "--audio-only" && arg != "-a"
    }).expect("No URL provided");

    // 1.5 STRIP CUSTOM PROTOCOL PREFIX
    // Browsers often append a trailing slash to custom protocols, so we trim both the prefix and any trailing slashes
    if let Some(stripped) = raw_url.strip_prefix("playku://") {
        raw_url = stripped.to_string();
    } else if let Some(stripped) = raw_url.strip_prefix("playku:") {
        raw_url = stripped.to_string();
    }



    // Parse modern query parameter indicators (injected by extension)
    if raw_url.contains("playku_ontop=true") {
        always_on_top = true;
    }
    if raw_url.contains("playku_audio=true") {
        audio_only = true;
    }

    // Ensure it has a valid HTTP/HTTPS scheme for mpv/yt-dlp
    // Browsers/OS protocols often alter or strip parts of nested URLs (e.g., changing "https://" to "https//")
    let mut url = raw_url;
    if url.starts_with("https//") {
        url = format!("https://{}", &url[7..]);
    } else if url.starts_with("http//") {
        url = format!("http://{}", &url[6..]);
    } else if url.starts_with("https://https//") {
        url = format!("https://{}", &url[15..]);
    } else if url.starts_with("https://http//") {
        url = format!("http://{}", &url[14..]);
    } else if !url.starts_with("http://") && !url.starts_with("https://") {
        url = format!("https://{}", url);
    }

    // 2. Locate the native config folder (mpv_config is automatically placed alongside the exe by build.rs)
    let mut exe_path = env::current_exe().expect("Failed to get executable path");
    exe_path.pop(); // Remove playku-engine.exe from the path
    
    let config_dir = exe_path.join("mpv_config");
    let safe_config_path = config_dir.to_str().unwrap().replace("\\", "/");

    println!("Initializing media player engine...");
    
    // 3. Initialize player with native config and audio-only setting
    let mut player = MpvPlayer::new(&safe_config_path, audio_only).context("Failed to initialize player")?;
    
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
