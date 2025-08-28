use super::{IdType, episode::Episode};
use itertools::Itertools;
use regex::Regex;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Season {
    pub id: IdType,
    pub number: i32,
    pub watched: bool,
    pub episodes: Vec<Episode>,
}

impl Ord for Season {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number
            .cmp(&other.number)
            .then_with(|| self.watched.cmp(&other.watched))
            .then_with(|| self.episodes.cmp(&other.episodes))
    }
}

impl PartialOrd for Season {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Season {
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
            && self.watched == other.watched
            && self
                .episodes
                .iter()
                .sorted()
                .eq(other.episodes.iter().sorted())
    }
}

impl Eq for Season {}

impl TryFrom<PathBuf> for Season {
    type Error = Box<dyn std::error::Error>;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let video_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let series = Self::detect_series(&video_stem)?;

        Ok(Self {
            id: 0,
            number: series.0,
            watched: false,
            episodes: vec![Episode::new(path, series.1)],
        })
    }
}

impl Season {
    pub fn merge(&mut self, other: &Self) {
        for new_episodes in &other.episodes {
            if let Some(old_episodes) = self
                .episodes
                .iter_mut()
                .find(|s| s.number == new_episodes.number)
            {
                old_episodes.merge(new_episodes);
            } else {
                self.episodes.push(new_episodes.clone());
            }
        }

        self.episodes.sort_by_key(|s| s.number);
    }
}

impl Season {
    fn detect_series(input: &str) -> Result<(i32, i32), Box<dyn std::error::Error>> {
        let re = Regex::new(r"(?i)s(\d{1,2})[\s._-]?e(\d{1,2})")?;

        re.captures(input)
            .and_then(|caps| {
                let season = caps.get(1)?.as_str().parse().ok()?;
                let episode = caps.get(2)?.as_str().parse().ok()?;
                Some(Ok((season, episode)))
            })
            .unwrap_or(Err("it's not series".into()))
    }
}

#[cfg(test)]
mod detect_series_tests {
    use super::*;

    #[test]
    fn detects_standard_sxxexx_format() {
        let input = "Breaking.Bad.S02E05.720p.mkv";
        let expected = (2, 5);
        assert_eq!(Season::detect_series(input).unwrap(), expected);
    }

    #[test]
    fn detects_underscored_format() {
        let input = "_S03_E10_";
        let expected = (3, 10);
        assert_eq!(Season::detect_series(input).unwrap(), expected);
    }

    #[test]
    fn detects_mixed_case_and_separator() {
        let input = "s04-e11.avi";
        let expected = (4, 11);
        assert_eq!(Season::detect_series(input).unwrap(), expected);
    }

    #[test]
    fn handles_lowercase_with_dot_separator() {
        let input = "showname.s01.e09.mkv";
        let expected = (1, 9);
        assert_eq!(Season::detect_series(input).unwrap(), expected);
    }

    #[test]
    fn returns_none_if_no_match() {
        let input = "Inception.2010.1080p.mkv";
        assert!(Season::detect_series(input).is_err());
    }

    #[test]
    fn handles_partial_match_but_incorrect_format() {
        let input = "some_show_S05E.avi";
        assert!(Season::detect_series(input).is_err());
    }

    #[test]
    fn accepts_input_with_leading_or_trailing_underscores() {
        let input = "_S2_E8_.Something.Else.mp4";
        let expected = (2, 8);
        assert_eq!(Season::detect_series(input).unwrap(), expected);
    }
}

#[cfg(test)]
mod test_try_from_season {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn valid_path() {
        let path = PathBuf::from("show.name.s01e01.mkv");
        let season = Season::try_from(path.clone()).unwrap();

        assert_eq!(season.id, 0);
        assert_eq!(season.number, 1);
        assert!(!season.watched);
        assert_eq!(season.episodes.len(), 1);
    }

    #[test]
    fn invalid_filename() {
        let path = PathBuf::from("invalid_file.mkv");
        let result = Season::try_from(path);

        assert!(result.is_err());
    }

    #[test]
    fn no_file_stem() {
        let path = PathBuf::from("/");
        let result = Season::try_from(path);

        assert!(result.is_err());
    }

    #[test]
    fn non_unicode_stem() {
        let path = PathBuf::from("invalid_stem");
        let result = Season::try_from(path);

        assert!(result.is_err());
    }

    #[test]
    fn lowercase_conversion() {
        let path = PathBuf::from("Show.Name.S02E03.MKV");
        let season = Season::try_from(path.clone()).unwrap();

        assert_eq!(season.number, 2);
    }
}

#[cfg(test)]
mod tests_merge {
    use super::*;

    #[test]
    fn no_overlap() {
        let mut season1 = Season {
            id: 0,
            watched: false,
            number: 1,
            episodes: vec![
                Episode {
                    id: 1,
                    number: 1,
                    watched: false,
                    files: vec![],
                },
                Episode {
                    id: 2,
                    number: 3,
                    watched: false,
                    files: vec![],
                },
            ],
        };

        let season2 = Season {
            id: 0,
            number: 1,
            watched: false,
            episodes: vec![
                Episode {
                    id: 3,
                    number: 2,
                    watched: false,
                    files: vec![],
                },
                Episode {
                    id: 4,
                    number: 4,
                    watched: false,
                    files: vec![],
                },
            ],
        };

        season1.merge(&season2);

        assert_eq!(season1.episodes.len(), 4);
        assert_eq!(season1.episodes[0].number, 1);
        assert_eq!(season1.episodes[1].number, 2);
        assert_eq!(season1.episodes[2].number, 3);
        assert_eq!(season1.episodes[3].number, 4);
    }

    #[test]
    fn with_overlap() {
        let mut season1 = Season {
            id: 0,
            watched: false,
            number: 1,
            episodes: vec![
                Episode {
                    id: 1,
                    number: 1,
                    watched: false,
                    files: vec![],
                },
                Episode {
                    id: 2,
                    number: 2,
                    watched: false,
                    files: vec![],
                },
            ],
        };

        let season2 = Season {
            id: 0,
            watched: false,
            number: 1,
            episodes: vec![
                Episode {
                    id: 3,
                    number: 2,
                    watched: false,
                    files: vec![],
                },
                Episode {
                    id: 4,
                    number: 3,
                    watched: false,
                    files: vec![],
                },
            ],
        };

        season1.merge(&season2);

        assert_eq!(season1.episodes.len(), 3);
        assert_eq!(season1.episodes[0].number, 1);
        assert_eq!(season1.episodes[1].number, 2);
        assert_eq!(season1.episodes[2].number, 3);
    }
}
