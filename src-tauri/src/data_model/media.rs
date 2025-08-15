use regex::Regex;
use std::path::PathBuf;

use super::imdb::Imdb;
use super::media_file::MediaFile;
use super::season::Season;

use itertools::Itertools;

#[derive(Debug, Clone, Default, Eq, serde::Serialize)]
pub struct Media {
    pub id: i64,
    pub name: String,
    pub year: Option<u32>,
    pub watched: bool,
    pub my_ranking: u8,
    pub watch_list: bool,
    pub seasons: Vec<Season>,
    pub files: Vec<MediaFile>,
    pub imdb: Option<Imdb>,
}

impl PartialEq for Media {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.year == other.year
            && self.watched == other.watched
            && self.my_ranking == other.my_ranking
            && self
                .seasons
                .iter()
                .sorted()
                .eq(other.seasons.iter().sorted())
            && self.files.iter().sorted().eq(other.files.iter().sorted())
            && self.imdb == other.imdb
            && self.watch_list == other.watch_list
    }
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
            Err(_) => (vec![], vec![MediaFile::from(path)]),
        };

        Self {
            id: 0,
            name: Self::detect_name(&video_stem),
            year: Self::detect_year(&video_stem),
            files,
            seasons: season,
            imdb: None,
            watched: false,
            my_ranking: 0,
            watch_list: false,
        }
    }
}

impl From<&PathBuf> for Media {
    fn from(path: &PathBuf) -> Self {
        Self::from(path.clone())
    }
}

impl Media {
    pub fn is_series(&self) -> bool {
        !self.seasons.is_empty()
    }

    pub fn merge(&mut self, other: &Self) {
        if self.year.is_none() {
            self.year = other.year;
        }

        if self.imdb.is_none() {
            self.imdb = other.imdb.clone();
        }

        self.files.extend(other.files.iter().cloned());

        for new_season in &other.seasons {
            if let Some(old_season) = self
                .seasons
                .iter_mut()
                .find(|s| s.number == new_season.number)
            {
                old_season.merge(new_season);
            } else {
                self.seasons.push(new_season.clone());
            }
        }

        self.seasons.sort_by_key(|s| s.number);
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
        re.captures(&cleaned)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str())
            .unwrap_or(&cleaned)
            .trim()
            .to_string()
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

            if !is_before_digit
                && !is_after_digit
                && let Ok(year) = s.parse::<u32>()
                && (1900..=2099).contains(&year)
            {
                last = Some(year);
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
                "3 days to kill",
            ),
            ("coco.2017.720p.bluray.dubbed.digimoviez", "coco"),
            ("in.time.2011.720p.film2media", "in time"),
            ("who.am.i.2014.720p.bluray.hardsub.digimoviez", "who am i"),
            (
                "radhe.2021.hindi.720p.web-dl.x264.farsi.dubbed.zardfilm.net",
                "radhe",
            ),
            ("tenet.dubbed", "tenet"),
            ("civil.war.2024.720p.web-dl.softsub.digimoviez", "civil war"),
            ("godfather_2022_dubbed_hd720", "godfather"),
            (
                "freelance.2023.10bit.1080p.x265.web-dl.6ch.psa.farsi.sub.film2media",
                "freelance",
            ),
            (
                "ralph.breaks.the.internet.2018.720p.farsi.dub",
                "ralph breaks the internet",
            ),
            ("black.mirror.s01.e01.480p.web-dl.x264", "black mirror"),
            (
                "breaking.bad.s02e13.720p.bluray.farsi.dubbed",
                "breaking bad",
            ),
            (
                "emperor_of_the_sea_2004_s01e04_farsi_dubbed_(mer30download.com)",
                "emperor of the sea",
            ),
            (
                "197863_harry_potter_and_the_halfblood_prince_2009_dubbed_1080p_brrip_anoxmous_salamdl",
                "197863 harry potter and the halfblood prince",
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
        assert_eq!(media.name, "movie");
        assert_eq!(media.year, Some(2020));
        assert!(!media.watched);
        assert_eq!(media.my_ranking, 0);
        assert_eq!(media.seasons.len(), 0);
        assert_eq!(media.files.len(), 1);
        assert_eq!(media.imdb, None);
    }

    #[test]
    fn as_season() {
        let path = PathBuf::from("/path/to/movie.s1e1.mp4");
        let media = Media::from(path);

        assert_eq!(media.id, 0);
        assert_eq!(media.name, "movie");
        assert_eq!(media.year, None);
        assert!(!media.watched);
        assert_eq!(media.my_ranking, 0);
        assert_eq!(media.seasons.len(), 1);
        assert_eq!(media.files.len(), 0);
        assert_eq!(media.imdb, None);
    }

    #[test]
    fn lowercase_conversion() {
        let path = PathBuf::from("/path/to/MOVIE.2021.MP4");
        let media = Media::from(path);

        assert_eq!(media.name, "movie");
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
        assert_eq!(media_from_ref.seasons.len(), media_from_owned.seasons.len());
        assert_eq!(media_from_ref.files.len(), media_from_owned.files.len());
        assert_eq!(media_from_ref.imdb, media_from_owned.imdb);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_media() -> Media {
        Media {
            name: "test".into(),
            ..Default::default()
        }
    }

    #[test]
    fn test_merge_year_none_to_some() {
        let mut media1 = default_media();
        let media2 = Media {
            year: Some(2020),
            ..default_media()
        };

        media1.merge(&media2);
        assert_eq!(media1.year, Some(2020));
    }

    #[test]
    fn test_merge_year_other_none() {
        let mut media1 = Media {
            year: Some(2019),
            ..default_media()
        };
        let media2 = default_media();

        media1.merge(&media2);
        assert_eq!(media1.year, Some(2019));
    }

    #[test]
    fn test_merge_files() {
        let file1 = MediaFile::generate_random_file(1);
        let file2 = MediaFile::generate_random_file(2);
        let file3 = MediaFile::generate_random_file(3);

        let mut media1 = Media {
            files: vec![file1.clone()],
            ..default_media()
        };
        let media2 = Media {
            files: vec![file2.clone(), file3.clone()],
            ..default_media()
        };

        media1.merge(&media2);
        assert_eq!(media1.files.len(), 3);
        assert_eq!(media1.files[0], file1);
        assert_eq!(media1.files[1], file2);
        assert_eq!(media1.files[2], file3);
    }

    #[test]
    fn test_merge_seasons_no_overlap() {
        let mut media1 = Media {
            seasons: vec![Season {
                id: 0,
                watched: false,
                number: 1,
                episodes: vec![],
            }],
            ..default_media()
        };
        let media2 = Media {
            seasons: vec![Season {
                number: 2,
                id: 0,
                watched: false,
                episodes: vec![],
            }],
            ..default_media()
        };

        media1.merge(&media2);
        assert_eq!(media1.seasons.len(), 2);
        assert_eq!(media1.seasons[0].number, 1);
        assert_eq!(media1.seasons[1].number, 2);
    }

    #[test]
    fn test_merge_seasons_with_overlap_calls_merge() {
        let mut media1 = Media {
            seasons: vec![Season {
                id: 0,
                watched: false,
                number: 1,
                episodes: vec![],
            }],
            ..default_media()
        };
        let media2 = Media {
            seasons: vec![Season {
                id: 0,
                watched: false,
                number: 1,
                episodes: vec![],
            }],
            ..default_media()
        };

        media1.merge(&media2);
        assert_eq!(media1.seasons.len(), 1);
    }

    #[test]
    fn test_merge_seasons_adds_new_and_sorts() {
        let mut media1 = Media {
            seasons: vec![
                Season {
                    id: 0,
                    watched: false,
                    number: 3,
                    episodes: vec![],
                },
                Season {
                    id: 0,
                    watched: false,
                    number: 1,
                    episodes: vec![],
                },
            ],
            ..default_media()
        };
        let media2 = Media {
            seasons: vec![Season {
                id: 0,
                watched: false,
                number: 2,
                episodes: vec![],
            }],
            ..default_media()
        };

        media1.merge(&media2);
        assert_eq!(media1.seasons.len(), 3);
        assert_eq!(media1.seasons[0].number, 1);
        assert_eq!(media1.seasons[1].number, 2);
        assert_eq!(media1.seasons[2].number, 3);
    }

    #[test]
    fn test_merge_empty_seasons() {
        let mut media1 = default_media();
        let media2 = default_media();

        media1.merge(&media2);
        assert_eq!(media1.seasons.len(), 0);
    }
}
