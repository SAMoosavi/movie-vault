use futures::stream::{self, StreamExt};
use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
struct VideoMeta {
    name: String,
    title: String,
    path: PathBuf,
    subtitle_path: Option<PathBuf>,
    has_hard_sub: bool,
    is_dubbed: bool,
    quality: Option<String>,
    series: Option<SeriesMeta>,
}

#[derive(Debug, Clone)]
struct SeriesMeta {
    season: u32,
    part: u32,
}

fn extract_name(title: &str) -> String {
    let parts: Vec<&str> = title.split('.').collect();
    if !parts.is_empty() {
        parts[0].to_string()
    } else {
        title.to_string()
    }
}

fn match_subtitles(videos: &[PathBuf], subs: &[PathBuf]) -> Vec<VideoMeta> {
    let mut results = vec![];

    for video in videos {
        let video_stem = video.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let video_dir = video.parent().unwrap_or(Path::new(""));

        let matching_sub = subs.iter().find(|s| {
            let sub_stem = s.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let sub_dir = s.parent().unwrap_or(Path::new(""));
            sub_stem.contains(video_stem) && sub_dir == video_dir
        });

        let title = video_stem.to_string();
        let name = extract_name(video_stem);
        let is_dubbed = video_stem.to_lowercase().contains("dub")
            || video_stem.to_lowercase().contains("farsi");
        let has_hard_sub = video_stem.to_lowercase().contains("sub");
        let quality = detect_quality(video_stem);
        let series = detect_series(video_stem);

        results.push(VideoMeta {
            name,
            title,
            path: video.clone(),
            subtitle_path: matching_sub.cloned(),
            has_hard_sub,
            is_dubbed,
            quality,
            series,
        });
    }

    results
}

fn detect_series(name: &str) -> Option<SeriesMeta> {
    let re = Regex::new(r"[Ss](\d{1,2})[Ee](\d{1,2})").unwrap();
    if let Some(caps) = re.captures(name) {
        let season = caps
            .get(1)
            .and_then(|s| s.as_str().parse().ok())
            .unwrap_or(0);
        let part = caps
            .get(2)
            .and_then(|s| s.as_str().parse().ok())
            .unwrap_or(0);
        return Some(SeriesMeta { season, part });
    }
    None
}

fn detect_quality(name: &str) -> Option<String> {
    let qualities = ["4k", "1080p", "720p", "480p"];
    for &q in &qualities {
        if name.to_lowercase().contains(q) {
            return Some(q.to_string());
        }
    }
    None
}

#[tokio::main]
async fn main() {
    let root = "/media/moosavi/film/marvel";

    // Collect file paths
    let entries: Vec<PathBuf> = WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    // Process files concurrently (limit = 50 at a time)
    let results = stream::iter(entries)
        .map(|path| async move {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if ["mp4", "mkv", "avi"].contains(&ext.as_str()) {
                Some(("video", path))
            } else if ext == "srt" {
                Some(("subtitle", path))
            } else {
                println!("what is it?{}", ext);
                None
            }
        })
        .buffer_unordered(50)
        .filter_map(async move |res| res)
        .collect::<Vec<_>>()
        .await;

    // Separate files
    let mut video_files = vec![];
    let mut subtitle_files = vec![];

    for (kind, path) in results {
        match kind {
            "video" => video_files.push(path),
            "subtitle" => subtitle_files.push(path),
            _ => {}
        }
    }

    println!("\n🎬 Found {} video files", video_files.len());
    println!("📝 Found {} subtitle files", subtitle_files.len());

    let meta_datas = match_subtitles(&video_files, &subtitle_files);

    for meta_data in &meta_datas {
        println!("{:#?}", meta_data);
    }
}
