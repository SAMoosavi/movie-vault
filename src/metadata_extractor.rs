use rayon::prelude::*;
use regex::Regex;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::media_scanner;

#[derive(Debug)]
pub struct SeriesMeta {
    pub season: u32,
    pub episode: u32,
}

#[derive(Debug)]
pub struct VideoMeta {
    pub name: String,
    pub title: String,
    pub path: PathBuf,
    pub subtitle_path: Option<PathBuf>,
    pub year: Option<u32>,
    pub quality: Option<String>,
    pub is_dubbed: bool,
    pub has_hard_sub: bool,
    pub has_soft_sub: bool,
    pub series: Option<SeriesMeta>,
}

pub fn match_subtitles(found_files: media_scanner::FoundFiles) -> Vec<VideoMeta> {
    // Pre-process subtitles into a more searchable structure
    let subtitles_by_dir: HashMap<PathBuf, Vec<(String, &PathBuf)>> = found_files
        .subtitles
        .par_iter()
        .filter_map(|sub| {
            let dir = sub.parent().unwrap_or_else(|| Path::new(""));
            let stem = sub
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase())?;
            Some((dir.to_path_buf(), (stem, sub)))
        })
        .fold(
            || HashMap::new(),
            |mut map: HashMap<PathBuf, Vec<(String, &PathBuf)>>, (dir, entry)| {
                map.entry(dir).or_default().push(entry);
                map
            },
        )
        .reduce(
            || HashMap::new(),
            |mut a, b| {
                for (k, v) in b {
                    a.entry(k).or_default().extend(v);
                }
                a
            },
        );

    found_files
        .videos
        .into_par_iter() // Process videos in parallel too
        .map(|video| {
            let video_dir = video.parent().unwrap_or_else(|| Path::new(""));
            let video_stem = video.file_stem().and_then(|s| s.to_str()).unwrap_or("");

            let mut meta_data = get_meta_data(video_stem, video.clone());

            meta_data.subtitle_path = subtitles_by_dir
                .get(video_dir)
                .and_then(|subs| {
                    subs.par_iter()  // Parallel search within directory
                        .find_any(|(sub_stem, _)| sub_stem.contains(&meta_data.name.to_lowercase()))
                        .map(|(_, sub_path)| sub_path.to_owned())
                })
                .cloned();

            meta_data
        })
        .collect()
}

fn get_meta_data(video_stem: &str, path: PathBuf) -> VideoMeta {
    let stem_lower = video_stem.to_lowercase().replace("_", ".");
    let name = extract_name(video_stem);

    VideoMeta {
        name,
        title: video_stem.to_string(),
        path,
        subtitle_path: None,
        year: detect_year(&stem_lower),
        quality: detect_quality(&stem_lower),
        is_dubbed: detect_dubbed(&stem_lower),
        has_hard_sub: detect_hard_sub(&stem_lower),
        has_soft_sub: detect_soft_sub(&stem_lower),
        series: detect_series(video_stem),
    }
}

fn extract_name(input: &str) -> String {
    // Remove common patterns first
    let cleaned = input
        .replace(['.', '_'], " ")
        .replace("Farsi", "")
        .replace("Dubbed", "")
        .replace("Dub", "")
        .replace("HardSub", "")
        .replace("SoftSub", "")
        .replace("BluRay", "")
        .replace("WEB-DL", "")
        .replace("10bit", "")
        .replace("x265", "")
        .replace("x264", "")
        .replace("6CH", "")
        .replace("PSA", "")
        .replace("Film2Media", "")
        .replace("DigiMoviez", "")
        .replace("Zardfilm.Net", "")
        .replace("mer30download.com", "")
        .replace("EXTENDED", "")
        .replace("(", "")
        .replace(")", "");

    // Extract the main title part (before year or quality)
    let re = Regex::new(r"(.*?)(\d{4}|[0-9]{3,4}p|s\d{1,2}e\d{1,2}|$)").unwrap();
    re.captures(&cleaned)
        .and_then(|caps| caps.get(1))
        .map(|m| {
            m.as_str()
                .trim()
                .split_whitespace()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_else(|| cleaned.trim().to_string())
}

fn detect_year(input: &str) -> Option<u32> {
    Regex::new(r"\b(19|20)\d{2}\b")
        .unwrap()
        .find(input)
        .and_then(|m| m.as_str().parse().ok())
}

fn detect_quality(input: &str) -> Option<String> {
    let re = Regex::new(r"(4k|2160p|1080p|720p|480p|hd|hq)").unwrap();
    re.find(input).map(|m| match m.as_str() {
        "hd" | "hq" => "720p".to_string(),
        s => s.to_string(),
    })
}

fn detect_dubbed(input: &str) -> bool {
    input.contains("dub") || input.contains("farsi") || input.contains("dubbed")
}

fn detect_hard_sub(input: &str) -> bool {
    input.contains("hardsub") || input.contains("hard sub")
}

fn detect_soft_sub(input: &str) -> bool {
    input.contains("softsub") || input.contains("soft sub") || input.contains("sub")
}

fn detect_series(input: &str) -> Option<SeriesMeta> {
    // Try S01E01 pattern first
    if let Some(caps) = Regex::new(r"[Ss](\d{1,2})[Ee](\d{1,2})")
        .unwrap()
        .captures(input)
    {
        let season = caps
            .get(1)
            .and_then(|s| s.as_str().parse().ok())
            .unwrap_or(0);
        let episode = caps
            .get(2)
            .and_then(|s| s.as_str().parse().ok())
            .unwrap_or(0);
        return Some(SeriesMeta { season, episode });
    }

    // Try _S01_E04 pattern
    if let Some(caps) = Regex::new(r"_S(\d{1,2})_E(\d{1,2})_")
        .unwrap()
        .captures(input)
    {
        let season = caps
            .get(1)
            .and_then(|s| s.as_str().parse().ok())
            .unwrap_or(0);
        let episode = caps
            .get(2)
            .and_then(|s| s.as_str().parse().ok())
            .unwrap_or(0);
        return Some(SeriesMeta { season, episode });
    }

    None
}
