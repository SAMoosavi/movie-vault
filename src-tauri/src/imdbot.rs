use anyhow::{Result, anyhow};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tauri_plugin_http::reqwest::Client;

use crate::{data_model::{Actor, Imdb, Media}, freeimdb::process_movies};

#[derive(Serialize, Deserialize)]
pub struct MovieSearchResult {
    pub ok: bool,
    pub description: Vec<SearchedMovie>,
    pub error_code: i32,
}

#[derive(Serialize, Deserialize)]
pub struct SearchedMovie {
    #[serde(rename = "#YEAR")]
    pub year: Option<i32>,
    #[serde(rename = "#IMDB_ID")]
    pub imdb_id: String,
}

#[derive(Deserialize, Debug)]
pub struct ImdbDataResponse {
    pub ok: bool,
    pub error_code: i32,
    pub description: String,
    pub short: ImdbShort,
    pub main: ImdbMain,
    #[serde(rename = "imdbId")]
    pub imdb_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImdbShort {
    #[serde(rename = "@type")]
    pub type_: String,
    pub name: String,
    pub image: String,
    pub description: String,
    pub aggregate_rating: AggregateRating,
    pub genre: Option<Vec<String>>,
    pub date_published: Option<String>,
    pub actor: Option<Vec<Person>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AggregateRating {
    pub rating_count: i32,
    pub rating_value: f64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub url: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImdbMain {
    countries_details: CountriesDetails,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CountriesDetails {
    countries: Vec<Country>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Country {
    text: String,
}

impl From<ImdbDataResponse> for Imdb {
    fn from(data: ImdbDataResponse) -> Self {
        Self {
            title: data.short.name,
            year: data
                .short
                .date_published
                .as_deref()
                .unwrap_or_default()
                .split('-')
                .next()
                .unwrap_or_default()
                .to_string(),
            released: data
                .short
                .date_published
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            genres: data.short.genre.unwrap_or_default(),
            actors: data
                .short
                .actor
                .unwrap_or_default()
                .into_iter()
                .map(|a| Actor {
                    id: String::new(),
                    name: a.name,
                    url: a.url,
                })
                .collect(),
            plot: data.short.description,
            countries: data
                .main
                .countries_details
                .countries
                .into_iter()
                .map(|c| c.text)
                .collect(),
            poster: data.short.image,
            imdb_rating: data.short.aggregate_rating.rating_value.to_string(),
            imdb_votes: data.short.aggregate_rating.rating_count.to_string(),
            imdb_id: data.imdb_id,
            r#type: data.short.type_,
        }
    }
}

async fn get_imdb_id(client: &Client, media: &Media) -> Result<String> {
    let response = client
        .get("https://imdb.iamidiotareyoutoo.com/search")
        .query(&[("q", &media.name)])
        .send()
        .await?;

    let result: MovieSearchResult = response.json().await?;

    if !result.ok {
        return Err(anyhow!("API error: {}", result.error_code));
    }

    let movies = &result.description;

    if movies.is_empty() {
        return Err(anyhow!("No movies found"));
    }

    let matched_movie = if let Some(year) = media.year {
        movies
            .iter()
            .filter(|m| m.year.is_some())
            .find(|m| m.year.unwrap() == year)
    } else {
        Some(&movies[0])
    };

    let imdb = match matched_movie {
        Some(movie) => movie.imdb_id.clone(),
        None => movies[0].imdb_id.clone(),
    };

    Ok(imdb)
}

async fn get_imdb_data(client: &Client, imdb_id: &str) -> Result<ImdbDataResponse> {
    let response = client
        .get("https://imdb.iamidiotareyoutoo.com/search")
        .query(&[("tt", imdb_id)])
        .send()
        .await?;

    let result: ImdbDataResponse = response.json().await?;

    if !result.ok {
        return Err(anyhow!(
            "API error: {}, {}",
            result.error_code,
            result.description
        ));
    }

    Ok(result)
}

pub async fn set_imdb_data(medias: &mut [Media]) {
    let client = Client::new();

    let futures = medias.iter_mut().map(|media| {
        let client = client.clone();
        async move {
            let imdb = get_imdb_id(&client, &*media).await?;
            anyhow::Ok((imdb, media))
        }
    });

    let result = join_all(futures).await;

    let mut pairs = Vec::new();
    for res in result {
        if let Ok((searched_movie, media)) = res {
            pairs.push((searched_movie, media));
        }
    }

    let ids: Vec<String> = pairs.iter().map(|(imdb_id, _)| imdb_id.clone()).collect();

    match process_movies(ids).await {
        Ok(imdbs) => {
            for imdb in imdbs {
                if let Some((_, media)) = pairs.iter_mut().find(|(imdb_id, _)| *imdb_id == imdb.imdb_id) {
                    media.imdb = Some(imdb);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch movies batch: {}", e);
        }
    }
}

pub async fn get_imdb_data_by_id(imdb_id: &str) -> Result<Imdb> {
    let client = Client::new();
    let imdb_data = get_imdb_data(&client, imdb_id).await?;

    Ok(imdb_data.into())
}

#[cfg(test)]
mod real_api_test {
    use super::*;
    use crate::data_model::Media;
    use std::path::PathBuf;
    use tauri_plugin_http::reqwest::Client;

    #[tokio::test]
    async fn fetch_movie_data() {
        let media = Media::from(PathBuf::from(
            "3.days.to.kill.2014.extended.720p.farsi.dubbed.film2media.mkv",
        ));
        let client = Client::new();

        let result = get_imdb_id(&client, &media).await;
        let imdb_id = result.unwrap();
        assert_eq!(imdb_id, "tt2172934");

        let result = get_imdb_data(&client, &imdb_id).await.unwrap();

        assert_eq!(result.short.type_, "Movie");
        assert_eq!(result.short.date_published.unwrap(), "2014-02-21");
        assert_eq!(
            result.main.countries_details.countries[0].text,
            "United States"
        );
    }

    #[tokio::test]
    async fn fetch_series_data() {
        let media = Media::from(PathBuf::from("black.mirror.s01.e01.480p.web-dl.x264.mkv"));
        let client = Client::new();

        let result = get_imdb_id(&client, &media).await;
        let imdb_id = result.unwrap();
        assert_eq!(imdb_id, "tt2085059");

        let result = get_imdb_data(&client, &imdb_id).await.unwrap();

        assert_eq!(result.short.type_, "tVSeries");
        assert_eq!(result.short.date_published.unwrap(), "2011-12-04");
        assert_eq!(
            result.main.countries_details.countries[0].text,
            "United Kingdom"
        );
    }

    #[tokio::test]
    async fn set_imdb_data() {
        let m1 = Media::from(PathBuf::from("black.mirror.s01.e01.480p.web-dl.x264.mkv"));
        let m2 = Media::from(PathBuf::from(
            "3.days.to.kill.2014.extended.720p.farsi.dubbed.film2media.mkv",
        ));

        let mut medias = vec![m1, m2];
        super::set_imdb_data(&mut medias).await;

        let new_m1 = &medias[0];
        assert_eq!(new_m1.name, "black mirror");

        let imdb = new_m1.imdb.as_ref().unwrap();
        assert_eq!(imdb.r#type, "tvSeries");
        assert_eq!(imdb.year, "2011");

        let new_m2 = &medias[1];
        assert_eq!(new_m2.name, "3 days to kill");

        let imdb = new_m2.imdb.as_ref().unwrap();
        assert_eq!(imdb.r#type, "movie");
        assert_eq!(imdb.year, "2014");
    }
}
