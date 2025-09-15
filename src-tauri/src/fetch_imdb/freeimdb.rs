use anyhow::{Result, anyhow};
use futures::stream::{self, StreamExt, TryStreamExt};
use serde::Deserialize;
use std::time::Duration;
use tauri_plugin_http::reqwest::{Client, StatusCode};
use tokio::time::sleep;

use crate::data_model::{self, Imdb};

#[derive(Deserialize, Debug)]
struct Response {
    titles: Vec<Title>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Title {
    id: String,
    #[serde(rename = "type")]
    title_type: Option<String>,
    primary_title: Option<String>,
    #[serde(default)]
    primary_image: Option<Image>,
    start_year: Option<u16>,
    genres: Vec<String>,
    rating: Option<Rating>,
    plot: Option<String>,
    #[serde(default)]
    stars: Vec<Person>,
    #[serde(default)]
    origin_countries: Vec<Country>,
    #[allow(dead_code)]
    #[serde(default)]
    directors: Vec<Person>,
    #[allow(dead_code)]
    #[serde(default)]
    writers: Vec<Person>,
}

#[derive(Deserialize, Debug)]
struct Image {
    url: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Rating {
    aggregate_rating: f64,
    vote_count: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Person {
    id: String,
    display_name: String,
    #[serde(default)]
    primary_image: Option<Image>,
}

#[derive(Deserialize, Debug)]
struct Country {
    name: String,
}

impl From<Title> for Imdb {
    fn from(value: Title) -> Self {
        Self {
            title: value.primary_title.unwrap_or_default(),
            year: value.start_year.map(|y| y.to_string()).unwrap_or_default(),
            released: value.start_year.map(|y| y.to_string()).unwrap_or_default(),
            genres: value.genres,
            actors: value
                .stars
                .iter()
                .map(|f| data_model::Person {
                    id: f.id.clone(),
                    name: f.display_name.clone(),
                    url: f
                        .primary_image
                        .as_ref()
                        .map(|img| img.url.clone())
                        .unwrap_or_default(),
                })
                .collect(),
            plot: value.plot.unwrap_or_default(),
            countries: value
                .origin_countries
                .iter()
                .map(|c| c.name.clone())
                .collect(),
            poster: value.primary_image.map(|img| img.url).unwrap_or_default(),
            imdb_rating: value
                .rating
                .as_ref()
                .map(|r| r.aggregate_rating.to_string())
                .unwrap_or_default(),
            imdb_votes: value
                .rating
                .as_ref()
                .map(|r| r.vote_count.to_string())
                .unwrap_or_default(),
            imdb_id: value.id,
            r#type: value.title_type.unwrap_or_default(),
        }
    }
}

const BATCH_SIZE: usize = 5;
const MAX_RETRIES: u32 = 3;
const DELAY_S: u64 = 10;
const CONCURRENCY: usize = 4;

async fn fetch_movies(client: &Client, ids: &[String]) -> Result<Response> {
    let url = "https://api.imdbapi.dev/titles:batchGet";
    let query: Vec<(&str, &str)> = ids.iter().map(|id| ("titleIds", id.as_str())).collect();

    for attempt in 1..=MAX_RETRIES {
        match client.get(url).query(&query).send().await {
            Ok(resp) if resp.status().is_success() => {
                return Ok(resp.json::<Response>().await?);
            }
            Ok(resp) if resp.status() == StatusCode::TOO_MANY_REQUESTS => {
                eprintln!("429 Too Many Requests — retrying after {}s", DELAY_S);
                if attempt == MAX_RETRIES {
                    return Err(anyhow!("429 Too Many Requests"));
                }
                sleep(Duration::from_secs(DELAY_S)).await;
            }
            Ok(resp) => {
                eprintln!("Request failed: {}", resp.status());
                if attempt == MAX_RETRIES {
                    return Err(resp.error_for_status().unwrap_err().into());
                }
            }
            Err(err) => {
                eprintln!("Network error: {}", err);
                if attempt == MAX_RETRIES {
                    return Err(err.into());
                }
            }
        }

        eprintln!("Retry {}/{}", attempt, MAX_RETRIES);
    }

    unreachable!("Loop must return or error out before reaching here")
}

pub async fn process_movies(movie_ids: Vec<String>) -> Result<Vec<Imdb>> {
    let client = Client::builder().build()?;

    let batches = movie_ids
        .chunks(BATCH_SIZE)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<_>>();

    let imdbs = stream::iter(batches)
        .map(|ids| {
            let client = client.clone();
            async move {
                let movies = fetch_movies(&client, &ids).await?;
                Ok::<Vec<Imdb>, anyhow::Error>(movies.titles.into_iter().map(Imdb::from).collect())
            }
        })
        .buffer_unordered(CONCURRENCY)
        .try_collect::<Vec<_>>()
        .await?
        .into_iter()
        .flatten()
        .collect();

    Ok(imdbs)
}

pub async fn get_imdb_data_by_id(id: &str) -> Result<Imdb> {
    let client = Client::new();

    let url = format!("https://api.imdbapi.dev/titles/{id}");

    for attempt in 1..=MAX_RETRIES {
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                return Ok(resp.json::<Title>().await?.into());
            }
            Ok(resp) if resp.status() == StatusCode::TOO_MANY_REQUESTS => {
                eprintln!("429 Too Many Requests — retrying after {}s", DELAY_S);
                if attempt == MAX_RETRIES {
                    return Err(anyhow!("429 Too Many Requests"));
                }
                sleep(Duration::from_secs(DELAY_S)).await;
            }
            Ok(resp) => {
                eprintln!("Request failed: {}", resp.status());
                if attempt == MAX_RETRIES {
                    return Err(resp.error_for_status().unwrap_err().into());
                }
            }
            Err(err) => {
                eprintln!("Network error: {}", err);
                if attempt == MAX_RETRIES {
                    return Err(err.into());
                }
            }
        }

        eprintln!("Retry {}/{}", attempt, MAX_RETRIES);
    }

    unreachable!("Loop must return or error out before reaching here")
}
