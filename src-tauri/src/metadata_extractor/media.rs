use regex::Regex;
use std::path::PathBuf;

use super::file::File;
use super::imdb::Imdb;
use super::season::Season;

#[derive(Debug, Clone)]
pub struct Media {
    pub id: i64,
    pub name: String,
    pub year: Option<u32>,
    pub watched: bool,
    pub my_ranking: u8,
    pub season: Vec<Season>,
    pub files: Vec<File>,
    pub imdb: Option<Imdb>,
}

impl From<PathBuf> for Media {
    fn from(path: PathBuf) -> Self {
        let video_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let (season, files) = match Season::try_from(path.clone()) {
            Ok(x) => (vec![x], vec![]),
            Err(_) => (vec![], vec![File::from(path)]),
        };

        Self {
            id: 0,
            name: Self::detect_name(&video_stem),
            year: Self::detect_year(&video_stem),
            files,
            season,
            imdb: None,
            watched: false,
            my_ranking: 0,
        }
    }
}

impl From<&PathBuf> for Media {
    fn from(path: &PathBuf) -> Self {
        Self::from(path.clone())
    }
}

impl Media {
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
            assert_eq!(
                Media::detect_year(input),
                expected,
                "Failed on input: {input:?}"
            );
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
            let result = Media::detect_name(input);
            assert_eq!(result, expected, "Failed on: {input}");
        }
    }
}

#[cfg(test)]
mod tests_media_from {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn as_movie() {
        let path = PathBuf::from("/path/to/movie.2020.mp4");
        let media = Media::from(path);

        assert_eq!(media.id, 0);
        assert_eq!(media.name, "Movie");
        assert_eq!(media.year, Some(2020));
        assert!(!media.watched);
        assert_eq!(media.my_ranking, 0);
        assert_eq!(media.season.len(), 0);
        assert_eq!(media.files.len(), 1);
        assert_eq!(media.imdb, None);
    }

    #[test]
    fn as_season() {
        let path = PathBuf::from("/path/to/movie.s1e1.mp4");
        let media = Media::from(path);

        assert_eq!(media.id, 0);
        assert_eq!(media.name, "Movie");
        assert_eq!(media.year, None);
        assert!(!media.watched);
        assert_eq!(media.my_ranking, 0);
        assert_eq!(media.season.len(), 1);
        assert_eq!(media.files.len(), 0);
        assert_eq!(media.imdb, None);
    }

    #[test]
    fn lowercase_conversion() {
        let path = PathBuf::from("/path/to/MOVIE.2021.MP4");
        let media = Media::from(path);

        assert_eq!(media.name, "Movie");
        assert_eq!(media.year, Some(2021));
    }

    #[test]
    fn test_from_ref_pathbuf() {
        let path = PathBuf::from("/path/to/movie.2022.mp4");
        let media_from_ref = Media::from(&path);
        let media_from_owned = Media::from(path.clone());

        assert_eq!(media_from_ref.id, media_from_owned.id);
        assert_eq!(media_from_ref.name, media_from_owned.name);
        assert_eq!(media_from_ref.year, media_from_owned.year);
        assert_eq!(media_from_ref.watched, media_from_owned.watched);
        assert_eq!(media_from_ref.my_ranking, media_from_owned.my_ranking);
        assert_eq!(media_from_ref.season.len(), media_from_owned.season.len());
        assert_eq!(media_from_ref.files.len(), media_from_owned.files.len());
        assert_eq!(media_from_ref.imdb, media_from_owned.imdb);
    }
}