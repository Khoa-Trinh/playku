use anyhow::{Context, Result};
use libmpv2::Mpv;

pub struct MpvPlayer {
    mpv: Mpv,
}

impl MpvPlayer {
    /// Initialize a new mpv instance with optimized settings for streaming
    pub fn new(config_dir: &str) -> Result<Self> {
        // Use with_initializer to configure the engine BEFORE it boots!
        let mpv = Mpv::with_initializer(|builder| {
            // Force the engine to act like a full native player
            builder.set_property("config", "yes")?;
            builder.set_property("config-dir", config_dir)?;
            // Tell it to auto-load the scripts we placed in the folder
            builder.set_property("load-scripts", "yes")?;
            // Enable internal ytdl hook for DASH seeking and quality management
            builder.set_property("ytdl", "yes")?;
            Ok(())
        })
        .map_err(|e| anyhow::anyhow!("{:?}", e))
        .context("Failed to initialize libmpv context")?;

        // CRITICAL: hardware decoding and rendering optimization
        mpv.set_property("hwdec", "auto-safe")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set hwdec")?;
        
        // Enforce Zero-Copy direct rendering
        mpv.set_property("vd-lavc-dr", "yes")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set vd-lavc-dr")?;
        
        mpv.set_property("vo", "gpu")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set vo")?;

        // Caching for long videos
        mpv.set_property("cache", "yes")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set cache")?;
        
        // Set maximum demuxer cache to 75MB to avoid excessive RAM usage
        mpv.set_property("demuxer-max-bytes", "75M")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set demuxer-max-bytes")?;

        mpv.set_property("demuxer-max-back-bytes", "25M")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("Failed to set demuxer-max-back-bytes")?;

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

    /// Execute a command on the mpv instance
    pub fn command(&self, name: &str, args: &[&str]) -> Result<()> {
        self.mpv.command(name, args)
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context(format!("Failed to execute command {}", name))
    }

    /// Play the given URL
    pub fn play(&self, url: &str) -> Result<()> {
        self.play_urls(&[url.to_string()])
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
            // -1.0 blocks the thread natively (0% CPU)
            match self.mpv.wait_event(-1.0) {
                Some(Ok(libmpv2::events::Event::Shutdown)) => break,
                Some(Err(_)) | None => std::thread::sleep(std::time::Duration::from_millis(10)),
                _ => {}
            }
        }
    }
}
