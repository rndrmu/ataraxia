use std::time::Duration;
use serde_json::Value;
use crate::protocol::TrackInfo;

#[derive(Debug, Clone)]
pub struct ResolvedTrack {
    pub info: TrackInfo,
    pub stream_url: String,
    pub http_headers: String,
}

pub async fn resolve(identifier: &str, ytdlp: &str) -> Result<ResolvedTrack, String> {
    let output = tokio::time::timeout(
        Duration::from_secs(30),
        tokio::process::Command::new(ytdlp)
            .args([
                "-f", "bestaudio[ext=webm]/bestaudio[acodec=opus]/bestaudio/best",
                "--dump-json", "--no-playlist", "--quiet", identifier,
            ])
            .output(),
    )
    .await
    .map_err(|_| "yt-dlp timed out".to_string())?
    .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!("yt-dlp exited with {}", output.status));
    }

    let info: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| e.to_string())?;

    let stream_url = info["url"]
        .as_str()
        .ok_or("no url in yt-dlp output")?
        .to_string();

    let mut http_headers = String::new();
    if let Some(hdrs) = info["http_headers"].as_object() {
        for (k, v) in hdrs {
            if let Some(v) = v.as_str() {
                use std::fmt::Write as _;
                let _ = write!(http_headers, "{}: {}\r\n", k, v);
            }
        }
    }

    let track_info = TrackInfo {
        identifier: info["webpage_url"]
            .as_str()
            .unwrap_or(identifier)
            .to_string(),
        title: info["title"].as_str().unwrap_or("Unknown").to_string(),
        author: info["uploader"].as_str().unwrap_or("Unknown").to_string(),
        length: info["duration"].as_f64().map(|d| (d * 1000.0) as u64).unwrap_or(0),
        uri: info["webpage_url"].as_str().unwrap_or(identifier).to_string(),
        artwork_url: info["thumbnail"].as_str().map(|s| s.to_string()),
        is_stream: info["is_live"].as_bool().unwrap_or(false),
        source_name: info["extractor_key"].as_str().unwrap_or("unknown").to_lowercase(),
    };

    Ok(ResolvedTrack { info: track_info, stream_url, http_headers })
}
