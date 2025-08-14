use rayon::prelude::*;
use std::{
    hash::{Hash, Hasher},
    path::PathBuf,
};

mod episode;
mod imdb;
mod media;
mod media_file;
mod season;

pub use episode::Episode;
pub use imdb::Imdb;
pub use media::Media;
pub use media_file::MediaFile;
pub use season::Season;

#[cfg(test)]
pub use media_file::LanguageFormat;

pub fn get_metadata(videos: &[PathBuf]) -> Vec<Media> {
    let meta_data: Vec<_> = videos.par_iter().map(Media::from).collect();
    merge_media(&meta_data)
}

use std::collections::HashMap;

#[derive(Clone, Eq)]
struct MediaKey {
    name: String,
    is_series: bool,
    year: Option<u32>,
}

impl Hash for MediaKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.is_series.hash(state);
    }
}

impl PartialEq for MediaKey {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.is_series == other.is_series
            && match (self.year, other.year) {
                (Some(y1), Some(y2)) => y1 == y2,
                _ => true,
            }
    }
}

fn merge_media(metas: &[Media]) -> Vec<Media> {
    let mut grouped: HashMap<MediaKey, Media> = HashMap::new();

    for new_meta in metas {
        let key = MediaKey {
            name: new_meta.name.clone(),
            is_series: new_meta.is_series(),
            year: new_meta.year,
        };

        if let Some(existing) = grouped.get_mut(&key) {
            existing.merge(new_meta);
            continue;
        }

        grouped.insert(key, new_meta.clone());
    }

    let mut result = grouped.into_values().collect::<Vec<Media>>();
    result.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then(a.year.cmp(&b.year))
            .then(a.is_series().cmp(&b.is_series()))
    });

    result
}

#[cfg(test)]
mod tests_merge_media {
    use super::*;

    fn default_media(
        name: String,
        year: Option<u32>,
        season: Vec<Season>,
        files: Vec<MediaFile>,
    ) -> Media {
        Media {
            id: 0,
            name,
            year,
            watched: false,
            my_ranking: 0,
            seasons: season,
            files,
            imdb: None,
        }
    }

    fn default_season(number: i32, episodes: Vec<Episode>) -> Season {
        Season {
            id: 0,
            number,
            watched: false,
            episodes,
        }
    }

    fn default_episode(number: i32, files: Vec<MediaFile>) -> Episode {
        Episode {
            id: 0,
            number,
            watched: false,
            files,
        }
    }

    #[test]
    fn empty_input() {
        let result = merge_media(&[]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn single_media() {
        let m1 = default_media("Movie1".to_string(), None, vec![], vec![]);
        let result = merge_media(&[m1.clone()]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], m1);
    }

    #[test]
    fn two_movies_same_name_same_year() {
        let f1 = MediaFile::generate_random_file(1);
        let f2 = MediaFile::generate_random_file(1);

        let m1 = default_media("Movie".to_string(), Some(2020), vec![], vec![f1.clone()]);
        let m2 = default_media("Movie".to_string(), Some(2020), vec![], vec![f2.clone()]);

        let mut expected = m1.clone();
        expected.files.push(f2.clone());

        let result = merge_media(&[m1, m2]);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Movie");
        assert_eq!(result[0].year, Some(2020));
        assert_eq!(result[0].files.len(), 2);
        assert!(result[0].files.contains(&f1));
        assert!(result[0].files.contains(&f2));
    }

    #[test]
    fn two_movies_same_name_different_years() {
        let f1 = MediaFile::generate_random_file(1);
        let f2 = MediaFile::generate_random_file(2);

        let m1 = default_media("Movie".to_string(), Some(2020), vec![], vec![f1.clone()]);
        let m2 = default_media("Movie".to_string(), Some(2010), vec![], vec![f2.clone()]);

        let result = merge_media(&[m1, m2]);

        // Since years differ, replaces with the last one
        assert_eq!(result.len(), 2);
        assert!(result[1].files.contains(&f1));
        assert_eq!(result[1].name, "Movie");
        assert_eq!(result[1].year, Some(2020));
        assert_eq!(result[1].files.len(), 1);

        assert_eq!(result[0].name, "Movie");
        assert_eq!(result[0].year, Some(2010));
        assert_eq!(result[0].files.len(), 1);
        assert!(result[0].files.contains(&f2));
    }

    #[test]
    fn two_movies_same_name_one_year_none() {
        let f1 = MediaFile::generate_random_file(1);
        let f2 = MediaFile::generate_random_file(2);

        let m1 = default_media("Movie".to_string(), None, vec![], vec![f1.clone()]);
        let m2 = default_media("Movie".to_string(), Some(2020), vec![], vec![f2.clone()]);

        let mut expected = m1.clone();
        expected.year = Some(2020); // Since merge sets it
        expected.files.push(f2.clone());

        let mut result = merge_media(&[m1, m2]);
        result.sort_by_key(|m| m.name.clone());

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Movie");
        assert_eq!(result[0].year, Some(2020));
        assert_eq!(result[0].files.len(), 2);
        assert!(result[0].files.contains(&f1));
        assert!(result[0].files.contains(&f2));
    }

    #[test]
    fn movie_and_series_same_name() {
        let f1 = MediaFile::generate_random_file(1);
        let s1 = default_season(1, vec![]);

        let m_movie = default_media("SameName".to_string(), Some(2020), vec![], vec![f1.clone()]);
        let m_series = default_media("SameName".to_string(), Some(2020), vec![s1.clone()], vec![]);

        let mut result = merge_media(&[m_movie.clone(), m_series.clone()]);
        result.sort_by_key(|m| (m.name.clone(), m.is_series()));

        assert_eq!(result.len(), 2);
        // Movie
        assert_eq!(result[0].name, "SameName");
        assert!(!result[0].is_series());
        assert_eq!(result[0].files, vec![f1]);
        // Series
        assert_eq!(result[1].name, "SameName");
        assert!(result[1].is_series());
        assert_eq!(result[1].seasons, vec![s1]);
    }

    #[test]
    fn two_series_same_name_same_year_different_seasons() {
        let e1 = default_episode(1, vec![]);
        let e2 = default_episode(1, vec![]);

        let s1 = default_season(1, vec![e1.clone()]);
        let s2 = default_season(2, vec![e2.clone()]);

        let m1 = default_media("Series".to_string(), Some(2020), vec![s1.clone()], vec![]);
        let m2 = default_media("Series".to_string(), Some(2020), vec![s2.clone()], vec![]);

        let mut expected = m1.clone();
        expected.seasons.push(s2.clone());

        let mut result = merge_media(&[m1, m2]);
        result.sort_by_key(|m| m.name.clone());

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Series");
        assert_eq!(result[0].year, Some(2020));
        assert_eq!(result[0].seasons.len(), 2);
        assert!(result[0].seasons.contains(&s1));
        assert!(result[0].seasons.contains(&s2));
    }

    #[test]
    fn multiple_with_conflicting_years_order_matters() {
        let f1 = MediaFile::generate_random_file(1);
        let f2 = MediaFile::generate_random_file(2);
        let f3 = MediaFile::generate_random_file(3);

        let m2000_1 = default_media("Movie".to_string(), Some(2000), vec![], vec![f1.clone()]);
        let m2000_2 = default_media("Movie".to_string(), Some(2000), vec![], vec![f3.clone()]);
        let m2010 = default_media("Movie".to_string(), Some(2010), vec![], vec![f2.clone()]);

        let result = merge_media(&[m2000_1, m2010, m2000_2]);

        // Due to replacement logic: ends up with last (m2000_2)
        assert_eq!(result.len(), 2);
        assert!(result.contains(&default_media(
            "Movie".to_string(),
            Some(2000),
            vec![],
            vec![f1, f3]
        )));
        assert!(result.contains(&default_media(
            "Movie".to_string(),
            Some(2010),
            vec![],
            vec![f2]
        )));
    }

    #[test]
    fn different_names() {
        let m1 = default_media("Movie1".to_string(), Some(2020), vec![], vec![]);
        let m2 = default_media("Movie2".to_string(), Some(2020), vec![], vec![]);

        let mut result = merge_media(&[m1.clone(), m2.clone()]);
        result.sort_by_key(|m| m.name.clone());

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], m1);
        assert_eq!(result[1], m2);
    }
}

#[cfg(test)]
mod tests_get_metadata {

    use super::*;

    #[test]
    fn get_metadata_of_series() {
        let ans = vec![Media {
            id: 0,
            name: "loki".into(),
            year: None,
            files: vec![],
            seasons: vec![Season {
                id: 0,
                number: 1,
                watched: false,
                episodes: vec![Episode {
                    id: 0,
                    number: 2,
                    watched: false,
                    files: vec![MediaFile {
                        path: "/marvel/loki/S1/Loki.S01E02.720p.WEB.DL.Dubbed.ZarFilm.mkv".into(),
                        file_name: "Loki.S01E02.720p.WEB.DL.Dubbed.ZarFilm".into(),
                        quality: Some("720p".into()),
                        id: 0,
                        language_format: LanguageFormat::Dubbed,
                    }],
                }],
            }],
            imdb: None,
            watched: false,
            my_ranking: 0,
        }];
        let result =
            get_metadata(&["/marvel/loki/S1/Loki.S01E02.720p.WEB.DL.Dubbed.ZarFilm.mkv".into()]);

        assert_eq!(result, ans);
    }

    #[test]
    fn get_metadata_of_movie() {
        let ans = vec![Media {
            id: 0,
            name: "who am i".into(),
            year: Some(2014),
            files: vec![MediaFile {
                id: 0,
                file_name: "Who.Am.I.2014.720p.BluRay.HardSub.DigiMoviez".into(),
                path: "/film/Who.Am.I.2014.720p.BluRay.HardSub.DigiMoviez.mp4".into(),
                quality: Some("720p".into()),
                language_format: LanguageFormat::HardSub,
            }],
            imdb: None,
            watched: false,
            my_ranking: 0,
            seasons: vec![],
        }];

        let result =
            get_metadata(&["/film/Who.Am.I.2014.720p.BluRay.HardSub.DigiMoviez.mp4".into()]);

        assert_eq!(result, ans);
    }

    #[test]
    fn get_metadata_complex() {
        let ans = vec![
            Media {
                id: 0,
                name: "loki".into(),
                year: None,
                files: vec![],
                seasons: vec![
                    Season {
                        id: 0,
                        number: 1,
                        watched: false,
                        episodes: vec![
                            Episode {
                                id: 0,
                                number: 2,
                                watched: false,
                                files: vec![
                                    MediaFile {
                                        id: 0,
                                        path: "/marvel/loki/S1/Loki.S01E02.720p.WEB.DL.Dubbed.ZarFilm.mkv".into(),
                                        file_name: "Loki.S01E02.720p.WEB.DL.Dubbed.ZarFilm".into(),
                                        quality: Some("720p".into()),
                                        language_format: LanguageFormat::Dubbed,
                                    },
                                    MediaFile {
                                        id: 0,
                                        path: "/marvel/loki/S1/Loki.S01E02.720p.WEB.DL.Dubbed.mkv".into(),
                                        file_name: "Loki.S01E02.720p.WEB.DL.Dubbed".into(),
                                        quality: Some("720p".into()),
                                        language_format: LanguageFormat::Dubbed,
                                    },
                                ],
                            },
                            Episode {
                                id: 0,
                                number: 3,
                                watched: false,
                                files: vec![
                                    MediaFile {
                                        id: 0,
                                        path: "/marvel/loki/S1/Loki.S01E03.720p.WEB.DL.Dubbed.mkv".into(),
                                        file_name: "Loki.S01E03.720p.WEB.DL.Dubbed".into(),
                                        quality: Some("720p".into()),
                                        language_format: LanguageFormat::Dubbed,
                                    },
                                    MediaFile {
                                        id: 0,
                                        path: "/marvel/loki/S1/Loki.S01E03.720p.WEB.DL.Dubbed.ZarFilm.mkv".into(),
                                        file_name: "Loki.S01E03.720p.WEB.DL.Dubbed.ZarFilm".into(),
                                        quality: Some("720p".into()),
                                        language_format: LanguageFormat::Dubbed,
                                    },
                                ],
                            },
                        ],
                    },
                    Season {
                        id: 0,
                        number: 2,
                        watched: false,
                        episodes: vec![
                            Episode {
                                id: 0,
                                number: 2,
                                watched: false,
                                files: vec![
                                    MediaFile {
                                        id: 0,
                                        path: "/marvel/loki/S2/Loki.S02E02.720p.WEB.DL.Dubbed.ZarFilm.mkv".into(),
                                        file_name: "Loki.S02E02.720p.WEB.DL.Dubbed.ZarFilm".into(),
                                        quality: Some("720p".into()),
                                        language_format: LanguageFormat::Dubbed,
                                    },
                                    MediaFile {
                                        id: 0,
                                        path: "/marvel/loki/S2/Loki.S02E02.720p.WEB.DL.Dubbed.mkv".into(),
                                        file_name: "Loki.S02E02.720p.WEB.DL.Dubbed".into(),
                                        quality: Some("720p".into()),
                                        language_format: LanguageFormat::Dubbed,
                                    },
                                ],
                            },
                            Episode {
                                id: 0,
                                number: 3,
                                watched: false,
                                files: vec![
                                    MediaFile {
                                        id: 0,
                                        path: "/marvel/loki/S2/Loki.S02E03.720p.WEB.DL.Dubbed.ZarFilm.mkv".into(),
                                        file_name: "Loki.S02E03.720p.WEB.DL.Dubbed.ZarFilm".into(),
                                        quality: Some("720p".into()),
                                        language_format: LanguageFormat::Dubbed,
                                    },
                                    MediaFile {
                                        id: 0,
                                        path: "/marvel/loki/S2/Loki.S02E03.720p.WEB.DL.Dubbed.mkv".into(),
                                        file_name: "Loki.S02E03.720p.WEB.DL.Dubbed".into(),
                                        quality: Some("720p".into()),
                                        language_format: LanguageFormat::Dubbed,
                                    },
                                ],
                            },
                        ],
                    },
                ],
                imdb: None,
                watched: false,
                my_ranking: 0,
            },
            Media {
                id: 0,
                name: "who am i".into(),
                year: Some(2014),
                files: vec![
                    MediaFile {
                        id: 0,
                        path: "/film/Who.Am.I.2014.720p.BluRay.HardSub.DigiMoviez.mp4".into(),
                        file_name: "Who.Am.I.2014.720p.BluRay.HardSub.DigiMoviez".into(),
                        quality: Some("720p".into()),
                        language_format: LanguageFormat::HardSub,
                    },
                    MediaFile {
                        id: 0,
                        path: "/film/Who.Am.I.2014.720p.BluRay.HardSub.F2M.mp4".into(),
                        file_name: "Who.Am.I.2014.720p.BluRay.HardSub.F2M".into(),
                        quality: Some("720p".into()),
                        language_format: LanguageFormat::HardSub,
                    },
                ],
                imdb: None,
                watched: false,
                my_ranking: 0,
                seasons: vec![],
            },
        ];

        let result = get_metadata(&[
            "/marvel/loki/S1/Loki.S01E02.720p.WEB.DL.Dubbed.ZarFilm.mkv".into(),
            "/marvel/loki/S2/Loki.S02E02.720p.WEB.DL.Dubbed.ZarFilm.mkv".into(),
            "/marvel/loki/S2/Loki.S02E02.720p.WEB.DL.Dubbed.mkv".into(),
            "/marvel/loki/S1/Loki.S01E02.720p.WEB.DL.Dubbed.mkv".into(),
            "/marvel/loki/S1/Loki.S01E03.720p.WEB.DL.Dubbed.mkv".into(),
            "/film/Who.Am.I.2014.720p.BluRay.HardSub.DigiMoviez.mp4".into(),
            "/marvel/loki/S2/Loki.S02E03.720p.WEB.DL.Dubbed.ZarFilm.mkv".into(),
            "/marvel/loki/S1/Loki.S01E03.720p.WEB.DL.Dubbed.ZarFilm.mkv".into(),
            "/film/Who.Am.I.2014.720p.BluRay.HardSub.F2M.mp4".into(),
            "/marvel/loki/S2/Loki.S02E03.720p.WEB.DL.Dubbed.mkv".into(),
        ]);

        assert_eq!(result, ans);
    }
}
