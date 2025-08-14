use crate::metadata_extractor::{Imdb, Media};

use futures::future::join_all;
use serde::Deserialize;

use tauri_plugin_http::reqwest::{self, Client};

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
    BoxOffice: Option<String>,
    totalSeasons: Option<String>,
    r#Type: String,
}

impl From<OmdbMovie> for Imdb {
    fn from(raw: OmdbMovie) -> Self {
        fn split_csv_field(field: &str) -> Vec<String> {
            field
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }

        Imdb {
            title: raw.Title,
            year: raw.Year,
            rated: raw.Rated,
            released: raw.Released,
            runtime: raw.Runtime,
            genres: split_csv_field(&raw.Genre),
            directors: split_csv_field(&raw.Director),
            writers: split_csv_field(&raw.Writer),
            actors: split_csv_field(&raw.Actors),
            plot: raw.Plot,
            languages: split_csv_field(&raw.Language),
            countries: split_csv_field(&raw.Country),
            awards: raw.Awards,
            poster: raw.Poster,
            imdb_rating: raw.imdbRating,
            imdb_votes: raw.imdbVotes,
            imdb_id: raw.imdbID,
            box_office: raw.BoxOffice,
            total_seasons: raw.totalSeasons,
            r#type: raw.Type,
        }
    }
}

pub async fn get_omdb_of_medias(medias: &[Media], api_key: &str) -> Vec<Media> {
    let client = Client::new();

    let tasks = medias.iter().map(|media| {
        let client = client.clone();
        let mut media = media.clone();
        let api_key = api_key.to_string();

        tokio::spawn(async move {
            let mut builder = client
                .get("https://www.omdbapi.com/")
                .query(&[("apikey", &api_key), ("t", &media.name)]);

            if let Some(year) = media.year {
                builder = builder.query(&[("y", &year.to_string())]);
            }

            let parsed = builder.send().await?.json::<OmdbMovie>().await?.into();

            media.imdb = Some(parsed);

            Ok::<Media, reqwest::Error>(media)
        })
    });

    join_all(tasks)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(Result::ok)
        .collect()
}

pub async fn get_omdb_by_id(imdb_id: &str, api_key: &str) -> reqwest::Result<Imdb> {
    let client = Client::new();
    let builder = client
        .get("https://www.omdbapi.com/")
        .query(&[("apikey", &api_key), ("i", &imdb_id)]);

    let imdb = builder.send().await?.json::<OmdbMovie>().await?.into();

    Ok(imdb)
}

#[cfg(test)]
mod test_get_omdb_of_medias {
    use super::*;

    #[tokio::test]
    async fn test_get_omdb_metadata() {
        let test_video = Media {
            id: 0,
            name: "3 Days To Kill".into(),
            year: None,
            files: vec![],
            seasons: vec![],
            imdb: None,
            watched: false,
            my_ranking: 0,
        };

        let result = get_omdb_of_medias(&[test_video.clone()], "4c602a26").await;

        assert_eq!(result.len(), 1);
        let imdb = result[0].imdb.clone().unwrap();

        assert_eq!(imdb.title, "3 Days to Kill");
        assert_eq!(imdb.year, "2014");
        assert_eq!(imdb.imdb_id, "tt2172934");
        assert_eq!(imdb.box_office.unwrap(), "$30,697,999");
    }

    #[tokio::test]
    async fn test_get_omdb_metadata_of_serial() {
        let test_video = Media {
            id: 0,
            name: "Breaking Bad".into(),
            year: None,
            files: vec![],
            seasons: vec![],
            imdb: None,
            watched: false,
            my_ranking: 0,
        };

        let result = get_omdb_of_medias(&[test_video.clone()], "4c602a26").await;

        assert_eq!(result.len(), 1);
        let imdb = result[0].imdb.clone().unwrap();

        assert_eq!(imdb.title, "Breaking Bad");
        assert_eq!(imdb.year, "2008–2013");
        assert_eq!(imdb.imdb_id, "tt0903747");
        assert_eq!(imdb.total_seasons.unwrap(), "5");
    }
}

#[cfg(test)]
mod test_get_omdb_with_id {
    use super::*;

    #[tokio::test]
    async fn test_get_omdb_with_id() {
        let result = get_omdb_by_id("tt0381849", "4c602a26").await;
        let ans =  Imdb {
            title: "3:10 to Yuma".into(),
            year: "2007".into(),
            rated: "R".into(),
            released: "07 Sep 2007".into(),
            runtime: "122 min".into(),
            genres: vec!["Action".into(),"Crime".into(),"Drama".into()],
            directors: vec!["James Mangold".into()],
            writers: vec!["Halsted Welles".into(),"Michael Brandt".into(),"Derek Haas".into()],
            actors: vec!["Russell Crowe".into(),"Christian Bale".into(),"Ben Foster".into()],
            plot: "A small-time rancher agrees to hold a captured outlaw who's awaiting a train to go to court in Yuma. A battle of wills ensues as the outlaw tries to psych out the rancher.".into(),
            languages: vec!["English".into(),"Chinese".into()],
            countries: vec!["United States".into()],
            awards: "Nominated for 2 Oscars. 3 wins & 32 nominations total".into(),
            poster: "https://m.media-amazon.com/images/M/MV5BODE0NTcxNTQzNF5BMl5BanBnXkFtZTcwMzczOTIzMw@@._V1_SX300.jpg".into(),
            imdb_rating: "7.6".into(),
            imdb_votes: "342,944".into(),
            imdb_id: "tt0381849".into(),
            box_office: Some("$53,606,916".into()),
            total_seasons: None,
            r#type: "movie".into()
        };

        assert_eq!(ans, result.unwrap());
    }
}
