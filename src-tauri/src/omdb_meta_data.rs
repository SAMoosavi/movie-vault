use crate::data_model::{Imdb, Media};

use anyhow::Result;
use futures::future::join_all;
use serde::Deserialize;
use tauri_plugin_http::reqwest::Client;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
struct OmdbMovie {
    title: String,
    year: String,
    rated: String,
    released: String,
    runtime: String,
    genre: String,
    director: String,
    writer: String,
    actors: String,
    plot: String,
    language: String,
    country: String,
    awards: String,
    poster: String,
    imdb_rating: String,
    imdb_votes: String,
    imdb_id: String,
    box_office: Option<String>,
    total_seasons: Option<String>,
    #[serde(rename = "type")]
    r#type: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "PascalCase")]
enum OmdbResponse {
    Movie(Box<OmdbMovie>),
    Error { response: String, error: String },
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
            title: raw.title,
            year: raw.year,
            rated: raw.rated,
            released: raw.released,
            runtime: raw.runtime,
            genres: split_csv_field(&raw.genre),
            directors: split_csv_field(&raw.director),
            writers: split_csv_field(&raw.writer),
            actors: split_csv_field(&raw.actors),
            plot: raw.plot,
            languages: split_csv_field(&raw.language),
            countries: split_csv_field(&raw.country),
            awards: raw.awards,
            poster: raw.poster,
            imdb_rating: raw.imdb_rating,
            imdb_votes: raw.imdb_votes,
            imdb_id: raw.imdb_id,
            box_office: raw.box_office,
            total_seasons: raw.total_seasons,
            r#type: raw.r#type,
        }
    }
}

pub async fn get_omdb_of_medias(medias: &[Media], api_key: &str) -> Result<Vec<Media>> {
    let client = Client::new();
    let api_key = api_key.to_string();

    let tasks = medias.iter().map(|media| {
        let client = client.clone();
        let mut media = media.clone();
        let api_key = api_key.clone();

        async move {
            let mut builder = client
                .get("https://www.omdbapi.com/")
                .query(&[("apikey", &api_key), ("t", &media.name)]);

            if let Some(year) = media.year {
                builder = builder.query(&[("y", &year.to_string())]);
            }

            match builder.send().await {
                Ok(resp) => match resp.json::<OmdbResponse>().await {
                    Ok(OmdbResponse::Movie(parsed)) => {
                        media.imdb = Some((*parsed).into());
                        Some(media)
                    }
                    Ok(OmdbResponse::Error { error, response }) => {
                        eprintln!("OMDb error:{response} -> {error}");
                        Some(media)
                    }
                    Err(e) => {
                        eprintln!("Parse error: {}", e);
                        Some(media)
                    }
                },
                Err(e) => {
                    eprintln!("❌ Failed to fetch OMDb for {}: {}", media.name, e);
                    Some(media)
                }
            }
        }
    });

    let medias_out: Vec<Media> = join_all(tasks).await.into_iter().flatten().collect();

    Ok(medias_out)
}

pub async fn get_omdb_by_id(imdb_id: &str, api_key: &str) -> Result<Imdb> {
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
            name: "3 Days To Kill".into(),
            ..Media::default()
        };

        let result = get_omdb_of_medias(std::slice::from_ref(&test_video), "4c602a26")
            .await
            .unwrap();

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
            name: "Breaking Bad".into(),
            ..Media::default()
        };

        let result = get_omdb_of_medias(std::slice::from_ref(&test_video), "4c602a26")
            .await
            .unwrap();

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
