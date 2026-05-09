use anyhow::{Context, Result};
use libmpv2::Mpv;
use crate::youtube::parse_youtube_url;

pub struct MpvPlayer {
    mpv: Mpv,
}

impl MpvPlayer {
    /// Initialize a new mpv instance with optimized settings for streaming
    pub fn new(config_dir: &str, audio_only: bool) -> Result<Self> {
        // Use with_initializer to configure the engine BEFORE it boots!
        let mpv = Mpv::with_initializer(move |builder| {
            // Force the engine to act like a full native player
            builder.set_property("config", "yes")?;
            builder.set_property("config-dir", config_dir)?;
            // Tell it to auto-load the scripts we placed in the folder
            builder.set_property("load-scripts", "yes")?;
            // Enable internal ytdl hook for DASH seeking and quality management
            builder.set_property("ytdl", "yes")?;
            
            if audio_only {
                // Disable video decoding entirely
                builder.set_property("vid", "no")?;
                builder.set_property("force-window", "yes")?;
            }
            
            Ok(())
        })
        .map_err(|e| anyhow::anyhow!("{:?}", e))
        .context("Failed to initialize libmpv context")?;

        // 1. Force the low-power hardware profile before setting other video options
        mpv.set_property("profile", "fast")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set profile=fast")?;

        // 2. Try modern, more efficient APIs. gpu-next uses the newer libplacebo renderer (Vulkan/D3D11)
        mpv.set_property("vo", "gpu-next,gpu")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set vo")?;

        // 3. More aggressive hardware decoding
        mpv.set_property("hwdec", "auto")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set hwdec")?;

        // 4. Force cheap scalers and disable dithering for maximum power savings
        let _ = mpv.set_property("scale", "bilinear");
        let _ = mpv.set_property("cscale", "bilinear");
        let _ = mpv.set_property("dscale", "bilinear");
        let _ = mpv.set_property("dither-depth", "no");

        // Enforce Zero-Copy direct rendering
        mpv.set_property("vd-lavc-dr", "yes")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set vd-lavc-dr")?;

        // Caching for long videos
        mpv.set_property("cache", "yes")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set cache")?;

        // 5. Smooth Out Network CPU Spikes by increasing read-ahead buffer time
        mpv.set_property("demuxer-readahead-secs", "60")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set demuxer-readahead-secs")?;

        // Set maximum demuxer cache to 25MB to avoid excessive RAM usage
        mpv.set_property("demuxer-max-bytes", "25M")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set demuxer-max-bytes")?;

        mpv.set_property("demuxer-max-back-bytes", "12M")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set demuxer-max-back-bytes")?;

        // Limit yt-dlp subtitle download to English only (prevents fetching dozens of auto-langs)
        mpv.set_property("ytdl-raw-options", "sub-langs=en")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set default ytdl-raw-options")?;

        // Window Management
        mpv.set_property("border", "no")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set border=no")?;
        
        mpv.set_property("window-dragging", "yes")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set window-dragging=yes")?;
        
        mpv.set_property("ontop", "yes")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set ontop=yes")?;

        // UI Customization (uosc)
        // Disable default bloat
        mpv.set_property("osc", "no")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set osc=no")?;
        
        mpv.set_property("osd-bar", "no")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set osd-bar=no")?;

        Ok(Self { mpv })
    }

    /// Set a property on the mpv instance
    pub fn set_property<T: libmpv2::SetData>(&self, name: &str, value: T) -> Result<()> {
        self.mpv.set_property(name, value)
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context(format!("Failed to set property {}", name))
    }

    /// Get a property from the mpv instance
    pub fn get_property<T: libmpv2::GetData>(&self, name: &str) -> Result<T> {
        self.mpv.get_property(name)
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context(format!("Failed to get property {}", name))
    }

    /// Execute a command on the mpv instance
    pub fn command(&self, name: &str, args: &[&str]) -> Result<()> {
        self.mpv.command(name, args)
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context(format!("Failed to execute command {}", name))
    }

    /// Play the given URL
    pub fn play(&self, url: &str) -> Result<()> {
        if let Some(parsed) = parse_youtube_url(url) {
            let video_id_str = match &parsed.video_id {
                Some(id) => format!("\"{}\"", id),
                None => "null".to_string(),
            };
            let playlist_id_str = match &parsed.playlist_id {
                Some(id) => format!("\"{}\"", id),
                None => "null".to_string(),
            };
            let json = format!(
                "{{\n  \"url_type\": \"{}\",\n  \"video_id\": {},\n  \"playlist_id\": {},\n  \"start_time_seconds\": {},\n  \"playback_action\": \"{}\"\n}}",
                parsed.url_type,
                video_id_str,
                playlist_id_str,
                parsed.start_time_seconds,
                parsed.playback_action
            );
            println!("YouTube URL Handler Action:\n{}", json);

            // Execute the playback action based on the parsed results
            match parsed.url_type.as_str() {
                "timestamped" => {
                    let start_str = parsed.start_time_seconds.to_string();
                    self.mpv.set_property("start", start_str.as_str())
                        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
                    
                    self.mpv.command("loadfile", &[url])
                        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
                    
                    // Reset start property to avoid affecting subsequent files
                    let _ = self.mpv.set_property("start", "none");
                }
                "playlist_item" => {
                    if let Some(idx) = parsed.index {
                        let start_idx = idx.saturating_sub(1);
                        let start_idx_str = start_idx.to_string();
                        self.mpv.set_property("playlist-start", start_idx_str.as_str())
                            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
                    }
                    // Enable playlist loading with limited sub-langs
                    self.mpv.set_property("ytdl-raw-options", "yes-playlist=,sub-langs=en")
                        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

                    self.mpv.command("loadfile", &[url])
                        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
                }
                "pure_playlist" => {
                    // Enable playlist loading with limited sub-langs
                    self.mpv.set_property("ytdl-raw-options", "yes-playlist=,sub-langs=en")
                        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

                    self.mpv.command("loadfile", &[url])
                        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
                }
                _ => {
                    self.mpv.command("loadfile", &[url])
                        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
                }
            }
            Ok(())
        } else {
            self.play_urls(&[url.to_string()])
        }
    }

    /// Play multiple URLs (e.g., video + audio)
    pub fn play_urls(&self, urls: &[String]) -> Result<()> {
        if urls.len() >= 2 {
            let video_url = &urls[0];
            let audio_url = &urls[1];

            // 1. Load the main video URL
            self.mpv.command("loadfile", &[video_url])
                .map_err(|e| anyhow::anyhow!("{:?}", e))
                .context("Failed to load video file")?;
            
            // 2. Wait a bit for mpv to initialize the session
            std::thread::sleep(std::time::Duration::from_millis(500));

            // 3. Add the audio file and select it
            self.mpv.command("audio-add", &[audio_url, "select"])
                .map_err(|e| anyhow::anyhow!("{:?}", e))
                .context("Failed to add audio file")?;
            
        } else if !urls.is_empty() {
            // Fallback: only one URL provided
            self.mpv.command("loadfile", &[&urls[0]])
                .map_err(|e| anyhow::anyhow!("{:?}", e))
                .context("Failed to load combined file")?;
        }
        Ok(())
    }

    /// Pause or unpause playback
    pub fn pause(&self, pause: bool) -> Result<()> {
        self.mpv
            .set_property("pause", pause)
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to toggle pause")?;
        Ok(())
    }

    /// Seek relative to current position (in seconds)
    pub fn seek(&self, seconds: f64) -> Result<()> {
        self.mpv
            .command("seek", &[&seconds.to_string()])
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to seek")?;
        Ok(())
    }

    /// Set volume (0 to 100)
    pub fn set_volume(&self, level: f64) -> Result<()> {
        self.mpv
            .set_property("volume", level)
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set volume")?;
        Ok(())
    }

    /// Block natively on the mpv event loop until shutdown/close
    pub fn wait(&mut self) {
        loop {
            // -1.0 blocks the thread natively (0% CPU) until a new event arrives
            match self.mpv.wait_event(-1.0) {
                Some(Ok(libmpv2::events::Event::Shutdown)) => break,
                
                // Catch the reason the file ended
                Some(Ok(libmpv2::events::Event::EndFile(reason))) => {
                    // In libmpv, the reason is returned as a raw u32 integer.
                    // 0 = EOF (Naturally finished playing)
                    // 2 = Stop (User skipped the track or changed the URL)
                    // 3 = Quit
                    
                    if reason == 0 { // Check for EOF
                        let current_pos = self.get_property::<i64>("playlist-pos-1").unwrap_or(1);
                        let total_count = self.get_property::<i64>("playlist-count").unwrap_or(1);

                        // If it naturally finished AND it is the last video, shut down.
                        if current_pos >= total_count {
                            break;
                        }
                    }
                }
                // Just loop back and block again immediately
                _ => continue,
            }
        }
    }
}
