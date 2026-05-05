mod player;

use anyhow::{Context, Result};
use std::env;
use std::io;

fn main() -> Result<()> {
    // Basic argument parsing: expect a YouTube URL as the first argument
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("\n=======================================================");
        println!("❌ ERROR: Missing YouTube URL argument!");
        println!("=======================================================");
        println!("To run this application, you must provide a video URL.");
        println!("Example usage via cargo:");
        println!("    cargo run -- \"https://www.youtube.com/watch?v=dQw4w9WgXcQ\"");
        println!("=======================================================\n");
        return Ok(());
    }
    
    let video_url = &args[1];
    
    // --- 1. LOCATE THE NATIVE CONFIG FOLDER ---
    let mut exe_path = env::current_exe().expect("Failed to get executable path");
    exe_path.pop(); // Remove playku.exe from the path
    
    let config_dir = exe_path.join("mpv_config");
    // Convert backslashes for the C-API just to be safe
    let safe_config_path = config_dir.to_str().unwrap().replace("\\", "/");

    println!("Initializing media player...");
    
    // --- 2. BOOT THE ENGINE WITH NATIVE CONFIG ---
    // Passing the path here ensures the font cache is built perfectly
    let player = player::MpvPlayer::new(&safe_config_path).context("Failed to initialize player")?;
    
    // 3. Play the URL directly (letting mpv's internal ytdl hook handle it)
    println!("Playing: {}", video_url);
    player.play(video_url)?;
    
    println!("Playback started. Press Enter to exit...");
    
    // Simple way to keep the application running until the user presses Enter
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    // 4. Graceful shutdown (the libmpv context is destroyed when `player` goes out of scope)
    println!("Shutting down...");
    
    Ok(())
}
