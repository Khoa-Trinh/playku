pub struct YouTubeUrl {
    pub url_type: String,
    pub video_id: Option<String>,
    pub playlist_id: Option<String>,
    pub start_time_seconds: u32,
    pub index: Option<u32>,
    pub playback_action: String,
}

pub fn parse_youtube_url(url: &str) -> Option<YouTubeUrl> {
    if !url.contains("youtube.com") && !url.contains("youtu.be") {
        return None;
    }

    let mut video_id = None;
    let mut playlist_id = None;
    let mut start_time_seconds = 0;
    let mut index = None;

    // Inline parsing of query parameters to avoid closure lifetime/borrow checker issues
    let mut params = std::collections::HashMap::new();
    let parts: Vec<&str> = url.split(|c| c == '?' || c == '&').collect();
    for part in parts {
        if let Some(eq_idx) = part.find('=') {
            let key = &part[..eq_idx];
            let val = &part[eq_idx + 1..];
            params.insert(key, val);
        }
    }

    if let Some(&v) = params.get("v") {
        video_id = Some(v.to_string());
    }
    if let Some(&list) = params.get("list") {
        playlist_id = Some(list.to_string());
    }
    if let Some(&idx) = params.get("index") {
        index = idx.parse::<u32>().ok();
    }
    if let Some(&t) = params.get("t") {
        start_time_seconds = parse_timestamp(t);
    }

    // Handle youtu.be/VIDEO_ID format
    if video_id.is_none() && url.contains("youtu.be/") {
        if let Some(pos) = url.find("youtu.be/") {
            let path_part = &url[pos + 9..];
            let id_end = path_part.find(|c| c == '?' || c == '&' || c == '/').unwrap_or(path_part.len());
            let v_id = &path_part[..id_end];
            if !v_id.is_empty() {
                video_id = Some(v_id.to_string());
            }
        }
    }

    // Determine url_type and playback action
    if video_id.is_some() && playlist_id.is_some() {
        let vid = video_id.clone().unwrap();
        let list = playlist_id.clone().unwrap();
        Some(YouTubeUrl {
            url_type: "playlist_item".to_string(),
            video_id,
            playlist_id,
            start_time_seconds,
            index,
            playback_action: format!("Play video {} and load playlist queue {} starting at index {} so the user can navigate within that context.", vid, list, index.unwrap_or(1)),
        })
    } else if playlist_id.is_some() {
        let list = playlist_id.clone().unwrap();
        Some(YouTubeUrl {
            url_type: "pure_playlist".to_string(),
            video_id: None,
            playlist_id,
            start_time_seconds: 0,
            index: None,
            playback_action: format!("Initiate playback starting automatically with the first item (index 1) in the playlist queue {}.", list),
        })
    } else if video_id.is_some() {
        let vid = video_id.clone().unwrap();
        if start_time_seconds > 0 {
            Some(YouTubeUrl {
                url_type: "timestamped".to_string(),
                video_id,
                playlist_id: None,
                start_time_seconds,
                index: None,
                playback_action: format!("Play the video {} starting at exactly {} seconds.", vid, start_time_seconds),
            })
        } else {
            Some(YouTubeUrl {
                url_type: "standard".to_string(),
                video_id,
                playlist_id: None,
                start_time_seconds: 0,
                index: None,
                playback_action: format!("Play the video {} from the beginning (0:00).", vid),
            })
        }
    } else {
        None
    }
}

fn parse_timestamp(t: &str) -> u32 {
    let mut total_seconds = 0;
    let mut current_number = 0;

    for c in t.chars() {
        if c.is_ascii_digit() {
            current_number = current_number * 10 + c.to_digit(10).unwrap_or(0);
        } else if c == 'h' {
            total_seconds += current_number * 3600;
            current_number = 0;
        } else if c == 'm' {
            total_seconds += current_number * 60;
            current_number = 0;
        } else if c == 's' {
            total_seconds += current_number;
            current_number = 0;
        }
    }

    if current_number > 0 && total_seconds == 0 {
        total_seconds = current_number;
    }

    total_seconds
}
