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
    let junk_tags = [
        "Farsi",
        "Dubbed",
        "Dub",
        "HardSub",
        "SoftSub",
        "BluRay",
        "WEB-DL",
        "10bit",
        "x265",
        "x264",
        "6CH",
        "PSA",
        "Film2Media",
        "DigiMoviez",
        "Zardfilm.Net",
        "mer30download.com",
        "EXTENDED",
        "HD720",
        "HD1080",
        "BrRip",
        "anoXmous",
        "SalamDL",
    ];

    // Normalize input
    let mut cleaned = input.replace(['.', '_', '(', ')'], " ");

    // Remove known junk tags
    for tag in junk_tags {
        cleaned = cleaned.replace(tag, "");
    }

    // Regex to stop title at first metadata marker (year, resolution, SxxExx)
    let re = Regex::new(
        r"(?i)(.*?)(?:\s+(19|20)\d{2}|\s+\d{3,4}p|\s+S\d{1,2}E\d{1,2}|\s+S\d{1,2}\s+E\d{1,2})",
    )
    .unwrap();
    let raw_name = re
        .captures(&cleaned)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .unwrap_or(&cleaned);

    raw_name
        .trim()
        .split_whitespace()
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn detect_year(input: &str) -> Option<u32> {
    let re = Regex::new(r"(19|20)\d{2}").unwrap();
    let mut last = None;

    for m in re.find_iter(input) {
        let s = m.as_str();
        let start = m.start();
        let end = m.end();

        let before = input[..start].chars().rev().next();
        let after = input[end..].chars().next();

        let is_before_digit = before.map(|c| c.is_ascii_digit()).unwrap_or(false);
        let is_after_digit = after.map(|c| c.is_ascii_digit()).unwrap_or(false);

        if !is_before_digit && !is_after_digit {
            if let Ok(year) = s.parse::<u32>() {
                if (1900..=2099).contains(&year) {
                    last = Some(year); // keep updating to get the last
                }
            }
        }
    }

    last
}

fn detect_quality(input: &str) -> Option<String> {
    // Case-insensitive regex for common quality tags
    let re = Regex::new(r"(?i)\b(4k|2160p|1080p|720p|480p|hd|hq)\b").unwrap();

    re.find(input)
        .map(|m| match m.as_str().to_lowercase().as_str() {
            "hd" | "hq" => "720p".to_string(),
            other => other.to_string(),
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
        let input = "_S03_E10_";
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

#[cfg(test)]
mod detect_quality {
    use super::*;

    #[test]
    fn test_detect_quality_matches() {
        let cases = vec![
            ("Movie.1080p.mkv", Some("1080p".to_string())),
            ("Show.4K.UltraHD", Some("4k".to_string())),
            ("Clip.HD.version", Some("720p".to_string())),
            ("Video.hq.release", Some("720p".to_string())),
            ("OldMovie.480p.avi", Some("480p".to_string())),
            ("UnknownQuality.mkv", None),
        ];

        for (input, expected) in cases {
            assert_eq!(
                detect_quality(input),
                expected,
                "Failed on input: {:?}",
                input
            );
        }
    }
}

#[cfg(test)]
mod detect_year {
    use super::*;

    #[test]
    fn test_detect_year_found() {
        let cases = [
            ("Movie.1999.1080p.mkv", Some(1999)),
            ("Film.2015.Release", Some(2015)),
            ("NoYearHere", None),
            ("Year200", None),
            ("OldMovie.1899", None),
            ("Future.2099", Some(2099)),
            ("Future_2099", Some(2099)),
            ("Future_2099_test", Some(2099)),
            ("Future_20993", None),
            ("2015.year.2020", Some(2020)),
            ("2015.year.2020.1080p", Some(2020)),
        ];

        for (input, expected) in cases {
            assert_eq!(detect_year(input), expected, "Failed on input: {:?}", input);
        }
    }
}

#[cfg(test)]
mod extract_name_tests {
    use super::*;

    #[test]
    fn test_extract_name_examples() {
        let cases = [
            (
                "3.Days.to.Kill.2014.EXTENDED.720p.Farsi.Dubbed.Film2Media",
                "3 Days To Kill",
            ),
            ("Coco.2017.720p.BluRay.Dubbed.DigiMoviez", "Coco"),
            ("In.Time.2011.720p.Film2Media", "In Time"),
            ("Who.Am.I.2014.720p.BluRay.HardSub.DigiMoviez", "Who Am I"),
            (
                "Radhe.2021.Hindi.720p.WEB-DL.x264.Farsi.Dubbed.Zardfilm.Net",
                "Radhe",
            ),
            ("tenet.Dubbed", "Tenet"),
            ("Civil.War.2024.720p.WEB-DL.SoftSub.DigiMoviez", "Civil War"),
            ("GodFather_2022_Dubbed_HD720", "GodFather"),
            (
                "Freelance.2023.10bit.1080p.x265.WEB-DL.6CH.PSA.Farsi.Sub.Film2Media",
                "Freelance",
            ),
            (
                "Ralph.Breaks.the.Internet.2018.720p.Farsi.Dub",
                "Ralph Breaks The Internet",
            ),
            ("Black.Mirror.S01.E01.480p.WEB-DL.x264", "Black Mirror"),
            (
                "Breaking.Bad.S02E13.720p.BluRay.Farsi.Dubbed",
                "Breaking Bad",
            ),
            (
                "Emperor_of_the_Sea_2004_s01e04_Farsi_Dubbed_(mer30download.com)",
                "Emperor Of The Sea",
            ),
            (
                "197863_Harry_Potter_and_the_HalfBlood_Prince_2009_DUBBED_1080p_BrRip_anoXmous_SalamDL",
                "197863 Harry Potter And The HalfBlood Prince",
            ),
        ];

        for (input, expected) in cases {
            let result = extract_name(input);
            assert_eq!(result, expected, "Failed on: {input}");
        }
    }
}
