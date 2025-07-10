use rayon::prelude::*;
use regex::Regex;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::media_scanner;

#[derive(Debug, PartialEq, Eq)]
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
    // Matches "dub", "dubbed", or "farsi" as whole words, case-insensitive
    let re = Regex::new(r"(?i)\b(dub|dubbed|farsi)\b").unwrap();
    re.is_match(input)
}

fn detect_hard_sub(input: &str) -> bool {
    let re = Regex::new(r"(?i)\bhard[\s._-]?(sub|subtitle)\b").unwrap();
    re.is_match(input)
}

fn detect_soft_sub(input: &str) -> bool {
    let re = Regex::new(r"(?i)\bsoft[\s._-]?(sub|subtitle)\b").unwrap();
    re.is_match(input)
}

fn detect_series(input: &str) -> Option<SeriesMeta> {
    let re = Regex::new(r"(?i)s(\d{1,2})[\s._-]?e(\d{1,2})").ok()?;

    re.captures(input).and_then(|caps| {
        let season = caps.get(1)?.as_str().parse().ok()?;
        let episode = caps.get(2)?.as_str().parse().ok()?;
        Some(SeriesMeta { season, episode })
    })
}

#[cfg(test)]
mod detect_series_tests {
    use super::*;

    #[test]
    fn detects_standard_sxxexx_format() {
        let input = "Breaking.Bad.S02E05.720p.mkv";
        let expected = Some(SeriesMeta {
            season: 2,
            episode: 5,
        });
        assert_eq!(detect_series(input), expected);
    }

    #[test]
    fn detects_underscored_format() {
        let input = "_S03_E10_.mp4";
        let expected = Some(SeriesMeta {
            season: 3,
            episode: 10,
        });
        assert_eq!(detect_series(input), expected);
    }

    #[test]
    fn detects_mixed_case_and_separator() {
        let input = "s04-e11.avi";
        let expected = Some(SeriesMeta {
            season: 4,
            episode: 11,
        });
        assert_eq!(detect_series(input), expected);
    }

    #[test]
    fn handles_lowercase_with_dot_separator() {
        let input = "showname.s01.e09.mkv";
        let expected = Some(SeriesMeta {
            season: 1,
            episode: 9,
        });
        assert_eq!(detect_series(input), expected);
    }

    #[test]
    fn returns_none_if_no_match() {
        let input = "Inception.2010.1080p.mkv";
        assert_eq!(detect_series(input), None);
    }

    #[test]
    fn handles_partial_match_but_incorrect_format() {
        let input = "some_show_S05E.avi";
        assert_eq!(detect_series(input), None);
    }

    #[test]
    fn accepts_input_with_leading_or_trailing_underscores() {
        let input = "_S2_E8_.Something.Else.mp4";
        let expected = Some(SeriesMeta {
            season: 2,
            episode: 8,
        });
        assert_eq!(detect_series(input), expected);
    }
}

#[cfg(test)]
mod detect_sub {
    use super::*;

    #[test]
    fn test_detect_hard_sub_cases() {
        let positives = [
            "movie.hardsub.mkv",
            "hard sub release",
            "hard.sub.version",
            "this-is-hardsub",
        ];

        for case in positives {
            assert!(
                detect_hard_sub(case),
                "Expected detect_hard_sub to return true for {:?}",
                case
            );
        }

        let negatives = ["softsub", "subtitle", "audio.hardtrack"];
        for case in negatives {
            assert!(
                !detect_hard_sub(case),
                "Expected detect_hard_sub to return false for {:?}",
                case
            );
        }
    }
    #[test]
    fn test_detect_soft_sub_cases() {
        let positives = [
            "movie.softsub.mkv",
            "soft sub release",
            "soft.sub.version",
            "this-is-softsub",
            "soft.subtitle",
        ];

        for case in positives {
            assert!(
                detect_soft_sub(case),
                "Expected detect_soft_sub to return true for {:?}",
                case
            );
        }

        let negatives = ["hardsub", "subtitle", "audio.hardtrack"];
        for case in negatives {
            assert!(
                !detect_hard_sub(case),
                "Expected detect_hard_sub to return false for {:?}",
                case
            );
        }
    }
}

#[cfg(test)]
mod detect_dubbed {
    use super::*;

    #[test]
    fn test_detect_dubbed_positive() {
        let positives = [
            "movie.dub.mkv",
            "Farsi dubbed version",
            "farsi audio",
            "official DUB release",
            "dubbed film",
        ];

        for input in positives {
            assert!(
                detect_dubbed(input),
                "Expected detect_dubbed to return true for {:?}",
                input
            );
        }
    }

    #[test]
    fn test_detect_dubbed_negative() {
        let negatives = [
            "dubious story",
            "redubbed version",
            "no subtitles",
            "audio track",
            "farsight analysis",
        ];

        for input in negatives {
            assert!(
                !detect_dubbed(input),
                "Expected detect_dubbed to return false for {:?}",
                input
            );
        }
    }
}