use rayon::prelude::*;
use regex::Regex;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::media_scanner;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SeriesMeta {
    pub season: u32,
    pub episode: u32,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VideoFileData {
    pub title: String,
    pub path: PathBuf,
    pub quality: Option<String>,
    pub has_hard_sub: bool,
    pub has_soft_sub: bool,
    pub is_dubbed: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VideoMetaData {
    pub name: String,
    pub subtitle_path: Option<PathBuf>,
    pub year: Option<u32>,
    pub files_data: Vec<VideoFileData>,
    pub series: Option<SeriesMeta>,
}

pub fn match_subtitles(found_files: media_scanner::FoundFiles) -> Vec<VideoMetaData> {
    // Build a map of subtitle stems by directory
    let subtitles_by_dir = found_files
        .subtitles
        .par_iter()
        .filter_map(|sub| {
            let dir = sub.parent().unwrap_or_else(|| Path::new(""));
            let stem = sub.file_stem()?.to_str()?.to_lowercase();
            Some((dir.to_path_buf(), (stem, sub)))
        })
        .fold(
            HashMap::<PathBuf, Vec<(String, &PathBuf)>>::new,
            |mut acc, (dir, entry)| {
                acc.entry(dir).or_default().push(entry);
                acc
            },
        )
        .reduce(HashMap::new, |mut a, b| {
            for (dir, entries) in b {
                a.entry(dir).or_default().extend(entries);
            }
            a
        });

    // Match subtitles to videos
    let meta_data = found_files
        .videos
        .into_par_iter()
        .map(|video| {
            let dir = video.parent().unwrap_or_else(|| Path::new(""));

            let mut meta = detect_metadata(video.clone());

            meta.subtitle_path = subtitles_by_dir.get(dir).and_then(|subs| {
                let name_lower = meta.name.to_lowercase();
                subs.par_iter()
                    .find_any(|(sub_stem, _)| sub_stem.contains(&name_lower))
                    .map(|(_, path)| path.to_path_buf())
            });

            meta
        })
        .collect();

    merge_metadata(meta_data)
}

fn merge_metadata(mut metas: Vec<VideoMetaData>) -> Vec<VideoMetaData> {
    let mut merged: Vec<VideoMetaData> = Vec::new();

    for mut new_meta in metas.drain(..) {
        if let Some(existing) = merged.iter_mut().find(|old| {
            old.name == new_meta.name
                && old.series == new_meta.series
                && match (old.year, new_meta.year) {
                    (Some(y1), Some(y2)) => y1 == y2,
                    _ => true,
                }
        }) {
            // Merge file data
            existing.files_data.append(&mut new_meta.files_data);

            // Fill year and subtitle path if missing
            existing
                .year
                .is_none()
                .then(|| existing.year = new_meta.year);
            existing
                .subtitle_path
                .is_none()
                .then(|| existing.subtitle_path = new_meta.subtitle_path);
        } else {
            merged.push(new_meta);
        }
    }

    merged
}

fn detect_metadata(path: PathBuf) -> VideoMetaData {
    let video_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    let normalized = video_stem.to_lowercase();

    VideoMetaData {
        name: detect_name(&normalized),
        subtitle_path: None,
        year: detect_year(&normalized),
        files_data: vec![VideoFileData {
            title: video_stem.into(),
            path,
            quality: detect_quality(&normalized),
            is_dubbed: detect_dubbed(&normalized),
            has_hard_sub: detect_hard_sub(&normalized),
            has_soft_sub: detect_soft_sub(&normalized),
        }],
        series: detect_series(&normalized),
    }
}

/// Assumes `input` is already lowercase for consistent matching.
fn detect_name(input: &str) -> String {
    #[cfg(debug_assertions)]
    {
        if input != input.to_lowercase() {
            eprintln!("Warning: input is not lowercase: '{input}'");
            return String::new();
        }
    }

    // List of junk tags to remove from the input string.
    // These represent common metadata noise in filenames.
    let junk_tags = [
        "farsi",
        "dubbed",
        "dub",
        "hardsub",
        "softsub",
        "bluray",
        "web-dl",
        "10bit",
        "x265",
        "x264",
        "6ch",
        "psa",
        "film2media",
        "digimoviez",
        "zardfilm.net",
        "mer30download.com",
        "extended",
        "hd720",
        "hd1080",
        "brrip",
        "anoxmous",
        "salamdl",
    ];

    // Step 1: Normalize separators by replacing '.', '(', ')' with spaces.
    let mut cleaned = input.replace(['.', '_', '-', '(', ')'], " ");

    // Step 2: Remove all junk tags to reduce noise.
    for tag in junk_tags.iter() {
        cleaned = cleaned.replace(tag, "");
    }

    // Step 3: Regex to find and truncate at metadata (year, quality, or season/episode).
    // Pattern explanation:
    //   - non-greedy capture of anything until a whitespace + (year|resolution|season-episode)
    //   - case-insensitive matching (?i)
    let re = Regex::new(
        r"(?i)(.*?)(?:\s+(19|20)\d{2}|\s+\d{3,4}p|\s+S\d{1,2}E\d{1,2}|\s+S\d{1,2}\s+E\d{1,2})",
    )
    .expect("Regex compilation failed");

    // Apply regex and extract the first capturing group or fallback to full cleaned string.
    let raw_name = re
        .captures(&cleaned)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .unwrap_or(&cleaned);

    // Step 4: Capitalize each word's first character, preserving rest lowercase.
    raw_name
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
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

        let before = input[..start].chars().next_back();
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
            "hd" | "hq" => "720p".into(),
            other => other.into(),
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
mod detect_sub_tests {
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
                "Expected detect_hard_sub to return true for {case:?}"
            );
        }

        let negatives = ["softsub", "subtitle", "audio.hardtrack"];
        for case in negatives {
            assert!(
                !detect_hard_sub(case),
                "Expected detect_hard_sub to return false for {case:?}"
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
                "Expected detect_soft_sub to return true for {case:?}"
            );
        }

        let negatives = ["hardsub", "subtitle", "audio.hardtrack"];
        for case in negatives {
            assert!(
                !detect_soft_sub(case),
                "Expected detect_hard_sub to return false for {case:?}"
            );
        }
    }
}

#[cfg(test)]
mod detect_dubbed_tests {
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
                "Expected detect_dubbed to return true for {input:?}"
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
                "Expected detect_dubbed to return false for {input:?}"
            );
        }
    }
}

#[cfg(test)]
mod detect_quality_tests {
    use super::*;

    #[test]
    fn test_detect_quality_matches() {
        let cases = vec![
            ("Movie.1080p.mkv", Some("1080p".into())),
            ("Show.4K.UltraHD", Some("4k".into())),
            ("Clip.HD.version", Some("720p".into())),
            ("Video.hq.release", Some("720p".into())),
            ("OldMovie.480p.avi", Some("480p".into())),
            ("UnknownQuality.mkv", None),
        ];

        for (input, expected) in cases {
            assert_eq!(
                detect_quality(input),
                expected,
                "Failed on input: {input:?}"
            );
        }
    }
}

#[cfg(test)]
mod detect_year_tests {
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
            assert_eq!(detect_year(input), expected, "Failed on input: {input:?}");
        }
    }
}

#[cfg(test)]
mod detect_name_tests {
    use super::*;

    #[test]
    fn test_detect_name_examples() {
        let cases = [
            (
                "3.days.to.kill.2014.extended.720p.farsi.dubbed.film2media",
                "3 Days To Kill",
            ),
            ("coco.2017.720p.bluray.dubbed.digimoviez", "Coco"),
            ("in.time.2011.720p.film2media", "In Time"),
            ("who.am.i.2014.720p.bluray.hardsub.digimoviez", "Who Am I"),
            (
                "radhe.2021.hindi.720p.web-dl.x264.farsi.dubbed.zardfilm.net",
                "Radhe",
            ),
            ("tenet.dubbed", "Tenet"),
            ("civil.war.2024.720p.web-dl.softsub.digimoviez", "Civil War"),
            ("godfather_2022_dubbed_hd720", "Godfather"),
            (
                "freelance.2023.10bit.1080p.x265.web-dl.6ch.psa.farsi.sub.film2media",
                "Freelance",
            ),
            (
                "ralph.breaks.the.internet.2018.720p.farsi.dub",
                "Ralph Breaks The Internet",
            ),
            ("black.mirror.s01.e01.480p.web-dl.x264", "Black Mirror"),
            (
                "breaking.bad.s02e13.720p.bluray.farsi.dubbed",
                "Breaking Bad",
            ),
            (
                "emperor_of_the_sea_2004_s01e04_farsi_dubbed_(mer30download.com)",
                "Emperor Of The Sea",
            ),
            (
                "197863_harry_potter_and_the_halfblood_prince_2009_dubbed_1080p_brrip_anoxmous_salamdl",
                "197863 Harry Potter And The Halfblood Prince",
            ),
        ];

        for (input, expected) in cases {
            let result = detect_name(input);
            assert_eq!(result, expected, "Failed on: {input}");
        }
    }
}

#[cfg(test)]
mod detect_metadata_tests {
    use super::*;

    #[test]
    fn test_get_meta_data_full() {
        assert_eq!(
            detect_metadata("/marvel/loki/S1/Loki.S01E02.720p.WEB.DL.Dubbed.ZarFilm.mkv".into()),
            VideoMetaData {
                name: "Loki".into(),
                subtitle_path: None,
                year: None,
                files_data: vec![VideoFileData {
                    path: "/marvel/loki/S1/Loki.S01E02.720p.WEB.DL.Dubbed.ZarFilm.mkv".into(),
                    title: "Loki.S01E02.720p.WEB.DL.Dubbed.ZarFilm".into(),
                    quality: Some("720p".into()),
                    is_dubbed: true,
                    has_hard_sub: false,
                    has_soft_sub: false,
                }],
                series: Some(SeriesMeta {
                    season: 1,
                    episode: 2,
                }),
            }
        );

        assert_eq!(
            detect_metadata("/film/Who.Am.I.2014.720p.BluRay.HardSub.DigiMoviez.mp4".into()),
            VideoMetaData {
                name: "Who Am I".into(),
                subtitle_path: None,
                year: Some(2014,),
                files_data: vec![VideoFileData {
                    title: "Who.Am.I.2014.720p.BluRay.HardSub.DigiMoviez".into(),
                    path: "/film/Who.Am.I.2014.720p.BluRay.HardSub.DigiMoviez.mp4".into(),
                    quality: Some("720p".into()),
                    is_dubbed: false,
                    has_hard_sub: true,
                    has_soft_sub: false
                }],
                series: None,
            }
        );

        assert_eq!(
            detect_metadata("/marvel/avengers/Avengers.2012.720p.Farsi.Dubbed.Film9.mkv".into()),
            VideoMetaData {
                name: "Avengers".into(),
                subtitle_path: None,
                year: Some(2012),
                files_data: vec![VideoFileData {
                    title: "Avengers.2012.720p.Farsi.Dubbed.Film9".into(),
                    path: "/marvel/avengers/Avengers.2012.720p.Farsi.Dubbed.Film9.mkv".into(),
                    quality: Some("720p".into()),
                    is_dubbed: true,
                    has_hard_sub: false,
                    has_soft_sub: false
                }],
                series: None,
            }
        );
    }
}

#[cfg(test)]
mod merge_metadata_tests {
    use super::*;

    fn dummy_file(title: &str, path: &str) -> VideoFileData {
        VideoFileData {
            title: title.into(),
            path: path.into(),
            quality: None,
            has_hard_sub: false,
            has_soft_sub: false,
            is_dubbed: false,
        }
    }

    #[test]
    fn test_merge_metadata_basic() {
        let meta1 = VideoMetaData {
            name: "Sample Movie".into(),
            subtitle_path: Some("subs/sample.srt".into()),
            year: Some(2023),
            files_data: vec![dummy_file("Sample Movie 1080p", "movies/sample_1080p.mkv")],
            series: None,
        };

        let meta2 = VideoMetaData {
            name: "Sample Movie".into(),
            subtitle_path: None,
            year: Some(2023),
            files_data: vec![dummy_file("Sample Movie 720p", "movies/sample_720p.mkv")],
            series: None,
        };

        let result = merge_metadata(vec![meta1, meta2]);

        assert_eq!(result.len(), 1);
        let merged = &result[0];

        assert_eq!(merged.name, "Sample Movie");
        assert_eq!(merged.year, Some(2023));
        assert_eq!(merged.subtitle_path, Some("subs/sample.srt".into()));
        assert_eq!(merged.files_data.len(), 2);
    }

    #[test]
    fn test_should_not_merge_metadata_different_year() {
        let meta1 = VideoMetaData {
            name: "Sample Movie".into(),
            subtitle_path: Some("subs/sample.srt".into()),
            year: Some(2024),
            files_data: vec![dummy_file("Sample Movie 1080p", "movies/sample_1080p.mkv")],
            series: None,
        };

        let meta2 = VideoMetaData {
            name: "Sample Movie".into(),
            subtitle_path: None,
            year: Some(2023),
            files_data: vec![dummy_file("Sample Movie 720p", "movies/sample_720p.mkv")],
            series: None,
        };

        let result = merge_metadata(vec![meta1, meta2]);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_with_missing_year() {
        let meta1 = VideoMetaData {
            name: "Show".into(),
            subtitle_path: None,
            year: None,
            files_data: vec![dummy_file("Show S01E01", "shows/show_s01e01.mkv")],
            series: Some(SeriesMeta {
                season: 1,
                episode: 1,
            }),
        };

        let meta2 = VideoMetaData {
            name: "Show".into(),
            subtitle_path: Some("subs/show_s01e01.srt".into()),
            year: Some(2021),
            files_data: vec![dummy_file("Show S01E01 720p", "shows/show_s01e01_720p.mkv")],
            series: Some(SeriesMeta {
                season: 1,
                episode: 1,
            }),
        };

        let result = merge_metadata(vec![meta1, meta2]);

        assert_eq!(result.len(), 1);
        let merged = &result[0];
        assert_eq!(merged.year, Some(2021));
        assert_eq!(merged.subtitle_path, Some("subs/show_s01e01.srt".into()));
        assert_eq!(merged.files_data.len(), 2);
    }

    #[test]
    fn test_merge_different_series_not_combined() {
        let meta1 = VideoMetaData {
            name: "Show".into(),
            subtitle_path: None,
            year: None,
            files_data: vec![dummy_file("Show S01E01", "shows/show_s01e01.mkv")],
            series: Some(SeriesMeta {
                season: 1,
                episode: 1,
            }),
        };

        let meta2 = VideoMetaData {
            name: "Show".into(),
            subtitle_path: None,
            year: None,
            files_data: vec![dummy_file("Show S01E02", "shows/show_s01e02.mkv")],
            series: Some(SeriesMeta {
                season: 1,
                episode: 2,
            }),
        };

        let result = merge_metadata(vec![meta1, meta2]);
        assert_eq!(result.len(), 2);
    }
}
