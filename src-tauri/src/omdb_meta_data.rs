use crate::metadata_extractor::{ImdbMetaData, VideoMetaData};

use futures::future::join_all;
use rayon::prelude::*;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Clone)]
struct OmdbMovie {
    Title: String,
    Year: String,
    Rated: String,
    Released: String,
    Runtime: String,
    Genre: String,
    Director: String,
    Writer: String,
    Actors: String,
    Plot: String,
    Language: String,
    Country: String,
    Awards: String,
    Poster: String,
    imdbRating: String,
    imdbVotes: String,
    imdbID: String,
    BoxOffice: Option<String>,    // Present only for movies
    totalSeasons: Option<String>, // Present only for series
}

impl From<OmdbMovie> for ImdbMetaData {
    fn from(raw: OmdbMovie) -> Self {
        fn split_csv_field(field: &str) -> Vec<String> {
            field
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }

        ImdbMetaData {
            title: raw.Title,
            year: raw.Year,
            rated: raw.Rated,
            released: raw.Released,
            runtime: raw.Runtime,
            genre: split_csv_field(&raw.Genre),
            directors: split_csv_field(&raw.Director),
            writers: split_csv_field(&raw.Writer),
            actors: split_csv_field(&raw.Actors),
            plot: raw.Plot,
            languages: split_csv_field(&raw.Language),
            country: split_csv_field(&raw.Country),
            awards: raw.Awards,
            poster: raw.Poster,
            imdb_rating: raw.imdbRating,
            imdb_votes: raw.imdbVotes,
            imdb_id: raw.imdbID,
            box_office: raw.BoxOffice,
            total_seasons: raw.totalSeasons,
        }
    }
}

async fn fetch_omdb_metadata(
    videos_metadata: &[VideoMetaData],
    api_key: &str,
) -> Vec<(VideoMetaData, Option<OmdbMovie>)> {
    let client = Client::new();
    let mut seen_titles = HashSet::new();

    let tasks = videos_metadata
        .iter()
        .filter(|meta| seen_titles.insert((meta.name.clone(), meta.year)))
        .cloned()
        .map(|meta| {
            let client = client.clone();
            let api_key = api_key.to_string();
            let name = meta.name.replace(' ', "+");
            let year = meta.year;

            tokio::spawn(async move {
                let base_url = std::env::var("OMDB_API_URL")
                    .unwrap_or_else(|_| "https://www.omdbapi.com".into());

                let url = match year {
                    Some(year) => format!("{base_url}/?apikey={api_key}&t={name}&y={year}"),
                    None => format!("{base_url}/?apikey={api_key}&t={name}"),
                };

                // println!("{url}");
                let result = match client.get(&url).send().await {
                    Ok(resp) => match resp.text().await {
                        Ok(body) => {
                            // println!("raw JSON: {body}");
                            serde_json::from_str::<OmdbMovie>(&body).ok()
                        }
                        Err(e) => {
                            // println!("error reading response body: {e:?}");
                            None
                        }
                    },
                    Err(e) => {
                        // println!("request error: {e:?}");
                        None
                    }
                };

                (meta, result)
            })
        });

    join_all(tasks)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect()
}

pub async fn get_omdb_metadata(
    videos_metadata: &[VideoMetaData],
    api_key: &str,
) -> Vec<VideoMetaData> {
    let fetched_map = fetch_omdb_metadata(videos_metadata, api_key).await;

    videos_metadata
        .par_iter()
        .cloned()
        .map(|mut meta| {
            let matched = fetched_map
                .iter()
                .find(|(f_meta, _)| f_meta.name == meta.name && f_meta.year == meta.year)
                .and_then(|(_, omdb)| omdb.clone().map(ImdbMetaData::from));

            meta.imdb_metadata = matched;
            meta
        })
        .collect()
}

#[cfg(test)]
mod test_fetch_omdb_metadata {
    use super::*;
    use crate::metadata_extractor::{SeriesMeta, VideoMetaData};

    #[tokio::test]
    async fn test_get_omdb_metadata() {
        // Mock server for OMDb
        let test_video = VideoMetaData {
            name: "3 Days To Kill".into(),
            subtitle_path: None,
            year: None,
            files_data: vec![],
            series: None,
            imdb_metadata: None,
        };

        // Override the API URL for testing
        let result = fetch_omdb_metadata(&[test_video.clone()], "4c602a26").await;

        assert_eq!(result.len(), 1);
        let (input, movie) = &result[0];
        let movie = movie.as_ref().unwrap();
        assert_eq!(*input, test_video);

        assert_eq!(movie.Title, "3 Days to Kill");
        assert_eq!(movie.Year, "2014");
        assert_eq!(movie.imdbID, "tt2172934");
        assert_eq!(movie.BoxOffice.as_ref().unwrap(), "$30,697,999");
        assert!(movie.totalSeasons.is_none());
    }

    #[tokio::test]
    async fn test_get_omdb_metadata_of_serial() {
        let test_video = VideoMetaData {
            name: "Breaking Bad".into(),
            subtitle_path: None,
            year: None,
            files_data: vec![],
            series: Some(SeriesMeta {
                season: 1,
                episode: 1,
            }),
            imdb_metadata: None,
        };

        // Override the API URL for testing
        let result = fetch_omdb_metadata(&[test_video.clone()], "4c602a26").await;

        assert_eq!(result.len(), 1);
        let (input, movie) = &result[0];
        let movie = movie.as_ref().unwrap();
        assert_eq!(*input, test_video);

        assert_eq!(movie.Title, "Breaking Bad");
        assert_eq!(movie.Year, "2008–2013");
        assert_eq!(movie.imdbID, "tt0903747");
        assert_eq!(movie.totalSeasons.as_ref().unwrap(), "5");
    }

    #[tokio::test]
    async fn test_get_omdb_metadata_with_mock() {
        /*  use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        // Start a local mock server
        let mock_server = MockServer::start().await;

        // Set up a mock response
        let omdb_response = serde_json::json!({
            "Title": "3 Days to Kill",
            "Year": "2014",
            "Rated": "PG-13",
            "Released": "2014-02-21",
            "Runtime": "117 min",
            "Genre": "Action, Drama, Thriller",
            "Director": "McG",
            "Writer": "Adi Hasak, Luc Besson",
            "Actors": "Kevin Costner, Hailee Steinfeld, Connie Nielsen",
            "Plot": "A dying CIA agent trying to reconnect with his estranged daughter is offered an experimental drug that could save his life in exchange for one last assignment.",
            "Language": "English",
            "Country": "United States",
            "Awards": "1 win & 1 nomination",
            "Poster": "https://example.com/poster.jpg",
            "imdbRating": "6.2",
            "imdbVotes": "83,000",
            "imdbID": "tt2172934",
            "Type": "movie",
            "BoxOffice": "$30,697,999",
            "Response": "True"
        });

        // Count how many times the endpoint is called
        let response = ResponseTemplate::new(200).set_body_json(omdb_response);
        Mock::given(method("GET"))
            .and(path("/"))
            .and(query_param("t", "3+Days+to+Kill"))
            .and(query_param("apikey", "testkey"))
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Provide test input
        let test_video = VideoMetaData {
            name: "3 Days to Kill".into(),
            subtitle_path: None,
            year: None,
            files_data: vec![],
            series: None,
            imdb_metadata: None,
        };

        // Use the mock server URL instead of real OMDb
        let api_key = "testkey";
        unsafe {
            std::env::set_var("OMDB_API_URL", mock_server.uri());
        }

        // Call the function
        let result = fetch_omdb_metadata(&[test_video.clone()], api_key).await;

        assert_eq!(result.len(), 1);
        let (input, movie) = &result[0];
        let movie = movie.as_ref().unwrap();
        assert_eq!(*input, test_video);
        assert_eq!(movie.Title, "3 Days to Kill");
        assert_eq!(movie.Year, "2014");
        assert_eq!(movie.imdbID, "tt2172934");
        assert_eq!(movie.BoxOffice.as_ref().unwrap(), "$30,697,999");
        assert!(movie.totalSeasons.is_none());
        */
    }
}

#[cfg(test)]
mod test_get_omdb_metadata {

    use super::*;
    use crate::metadata_extractor::VideoMetaData;

    #[tokio::test]
    async fn test_get_omdb_metadata() {
        let test_video = VideoMetaData {
            name: "Ghost Rider".into(),
            subtitle_path: None,
            year: Some(2007),
            files_data: vec![],
            series: None,
            imdb_metadata: None,
        };

        // Override the API URL for testing
        let result = get_omdb_metadata(&[test_video.clone()], "4c602a26").await;

        let result_video = VideoMetaData {
            name: "Ghost Rider".into(),
            subtitle_path: None,
            year: Some(2007),
            files_data: vec![],
            series: None,
            imdb_metadata: Some(ImdbMetaData {
                title: "Ghost Rider".into(),
                year: "2007".into(),
                rated: "PG-13".into(),
                released: "16 Feb 2007".into(),
                runtime: "110 min".into(),
                genre: vec!["Action".into(), "Fantasy".into(), "Thriller".into()],
                directors: vec!["Mark Steven Johnson".into()],
                writers: vec!["Mark Steven Johnson".into()],
                actors: vec!["Nicolas Cage".into(), "Eva Mendes".into(), "Sam Elliott".into()],
                plot: "A motorcycle stuntman who sold his soul becomes a supernatural agent of vengeance.".into(),
                languages: vec!["English".into()],
                country: vec!["United States".into(), "Australia".into()],
                awards: "1 win & 11 nominations total".into(),
                poster: "https://m.media-amazon.com/images/M/MV5BMzIyNDE5ODI1OV5BMl5BanBnXkFtZTcwNTIyNDE0MQ@@._V1_SX300.jpg".into(),
                imdb_rating: "5.3".into(),
                imdb_votes: "261,090".into(),
                imdb_id: "tt0259324".into(),
                box_office: Some("$115,802,596".into()),
                total_seasons: None,
            }),
        };

        assert_eq!(vec![result_video], result);
    }
}
