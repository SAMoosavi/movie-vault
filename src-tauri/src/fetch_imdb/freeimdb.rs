use anyhow::{Result, anyhow};
use futures::stream::{self, StreamExt, TryStreamExt};
use serde::Deserialize;
use std::time::Duration;
use tauri_plugin_http::reqwest::{Client, StatusCode};
use tauri_plugin_log::log::{error, info, warn};
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
    start_year: Option<i32>,
    genres: Vec<String>,
    rating: Option<Rating>,
    plot: Option<String>,
    #[serde(default)]
    stars: Vec<Person>,
    #[serde(default)]
    origin_countries: Vec<Country>,
    #[serde(default)]
    directors: Vec<Person>,
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
    vote_count: i32,
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

impl From<&Person> for data_model::Person {
    fn from(value: &Person) -> Self {
        Self {
            id: value.id.clone(),
            name: value.display_name.clone(),
            url: value
                .primary_image
                .as_ref()
                .map(|img| img.url.clone())
                .unwrap_or_default(),
        }
    }
}

impl From<Title> for Imdb {
    fn from(value: Title) -> Self {
        Self {
            title: value.primary_title.unwrap_or_default(),
            year: value.start_year.unwrap_or_default(),
            genres: value.genres,
            actors: value.stars.iter().map(Into::into).collect(),
            writers: value.writers.iter().map(Into::into).collect(),
            directors: value.directors.iter().map(Into::into).collect(),
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
                .map(|r| r.vote_count)
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

async fn fetch_movies(client: &Client, ids: &[String], base_url: &str) -> Result<Response> {
    let url = format!("{}/titles:batchGet", base_url);
    let query: Vec<(&str, &str)> = ids.iter().map(|id| ("titleIds", id.as_str())).collect();

    info!("Fetching movies from {} with {} IDs", url, ids.len());

    for attempt in 1..=MAX_RETRIES {
        match client.get(&url).query(&query).send().await {
            Ok(resp) if resp.status().is_success() => {
                info!("Successfully fetched movies on attempt {}", attempt);
                return Ok(resp.json::<Response>().await?);
            }
            Ok(resp) if resp.status() == StatusCode::TOO_MANY_REQUESTS => {
                warn!(
                    "429 Too Many Requests — retrying after {}s (attempt {}/{})",
                    DELAY_S, attempt, MAX_RETRIES
                );
                if attempt == MAX_RETRIES {
                    error!("Max retries reached for 429 Too Many Requests");
                    return Err(anyhow!("429 Too Many Requests"));
                }
                sleep(Duration::from_secs(DELAY_S)).await;
            }
            Ok(resp) => {
                error!("Request failed with status {}", resp.status());
                if attempt == MAX_RETRIES {
                    return Err(anyhow!(
                        "request failed with status {} after {} attempts",
                        resp.status(),
                        MAX_RETRIES
                    ));
                }
            }
            Err(err) => {
                error!(
                    "Network error: {} (attempt {}/{})",
                    err, attempt, MAX_RETRIES
                );
                if attempt == MAX_RETRIES {
                    return Err(err.into());
                }
            }
        }

        warn!("Retry {}/{}", attempt, MAX_RETRIES);
    }

    unreachable!("Loop must return or error out before reaching here")
}

pub async fn process_movies(movie_ids: Vec<String>) -> Result<Vec<Imdb>> {
    info!("Processing movies for IDs: {:?}", movie_ids);
    let result = process_movies_inner(movie_ids, "https://api.imdbapi.dev").await;

    match &result {
        Ok(_) => info!("Successfully processed movies"),
        Err(e) => error!("Error processing movies: {}", e),
    }

    result
}

async fn process_movies_inner(movie_ids: Vec<String>, base_url: &str) -> Result<Vec<Imdb>> {
    let client = Client::builder().build()?;
    info!("Processing movies in batches for IDs: {:?}", movie_ids);

    let batches = movie_ids
        .chunks(BATCH_SIZE)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<_>>();

    let imdbs = stream::iter(batches)
        .map(|ids| {
            let client = client.clone();
            async move {
                info!("Fetching movies for batch: {:?}", ids);
                match fetch_movies(&client, &ids, base_url).await {
                    Ok(movies) => {
                        info!("Successfully fetched movies for batch: {:?}", ids);
                        Ok::<Vec<Imdb>, anyhow::Error>(
                            movies.titles.into_iter().map(Imdb::from).collect(),
                        )
                    }
                    Err(e) => {
                        error!("Error fetching movies for batch {:?}: {}", ids, e);
                        Err(e)
                    }
                }
            }
        })
        .buffer_unordered(CONCURRENCY)
        .try_collect::<Vec<_>>()
        .await?
        .into_iter()
        .flatten()
        .collect();

    info!("Successfully processed all movie batches");
    Ok(imdbs)
}

async fn get_imdb_data_by_id_inner(client: &Client, id: &str, base_url: &str) -> Result<Imdb> {
    let url = format!("{}/titles/{}", base_url, id);
    info!("Fetching IMDb data for ID: {}", id);

    for attempt in 1..=MAX_RETRIES {
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                info!("Successfully fetched IMDb data for ID: {}", id);
                return Ok(resp.json::<Title>().await?.into());
            }
            Ok(resp) if resp.status() == StatusCode::TOO_MANY_REQUESTS => {
                warn!(
                    "429 Too Many Requests for ID: {} — retrying after {}s",
                    id, DELAY_S
                );
                if attempt == MAX_RETRIES {
                    error!(
                        "Max retries reached for ID: {} with 429 Too Many Requests",
                        id
                    );
                    return Err(anyhow!("429 Too Many Requests"));
                }
                sleep(Duration::from_secs(DELAY_S)).await;
            }
            Ok(resp) => {
                error!(
                    "Request failed for ID: {} with status {}",
                    id,
                    resp.status()
                );
                if attempt == MAX_RETRIES {
                    return Err(anyhow!(
                        "request failed for id {} with status {} after {} attempts",
                        id,
                        resp.status(),
                        MAX_RETRIES
                    ));
                }
            }
            Err(err) => {
                error!("Network error for ID: {}: {}", id, err);
                if attempt == MAX_RETRIES {
                    return Err(err.into());
                }
            }
        }

        warn!("Retry {}/{} for ID: {}", attempt, MAX_RETRIES, id);
    }

    unreachable!("Loop must return or error out before reaching here")
}

pub async fn get_imdb_data_by_id(id: &str) -> Result<Imdb> {
    let client = Client::new();
    info!("Fetching IMDb data for ID: {}", id);
    match get_imdb_data_by_id_inner(&client, id, "https://api.imdbapi.dev").await {
        Ok(imdb) => {
            info!("Successfully fetched IMDb data for ID: {}", id);
            Ok(imdb)
        }
        Err(e) => {
            error!("Error fetching IMDb data for ID {}: {}", id, e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Matcher;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_fetch_movies_success() {
        let mut server = mockito::Server::new_async().await;
        let base_url = server.url();
        let json_response =
            r#"{"titles": [{"id": "tt0111161", "primaryTitle": "Test Movie", "genres": []}]}"#;
        server
            .mock("GET", "/titles:batchGet")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_response)
            .create();
        let client = Client::new();
        let ids = vec!["tt0111161".to_string()];
        let result = fetch_movies(&client, &ids, &base_url).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.titles.len(), 1);
        assert_eq!(response.titles[0].id, "tt0111161");
    }

    #[tokio::test]
    async fn test_get_imdb_data_by_id_success() {
        let mut server = mockito::Server::new_async().await;
        let base_url = server.url();
        let json_response = r#"{"id": "tt0111161", "primaryTitle": "Test Movie", "genres": []}"#;
        server
            .mock("GET", "/titles/tt0111161")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_response)
            .create();
        let client = Client::new();
        let result = get_imdb_data_by_id_inner(&client, "tt0111161", &base_url).await;
        assert!(result.is_ok());
        let imdb = result.unwrap();
        assert_eq!(imdb.imdb_id, "tt0111161");
    }

    #[tokio::test]
    async fn test_fetch_movies_retry_on_429() {
        let mut server = mockito::Server::new_async().await;
        let base_url = server.url();
        let json_response =
            r#"{"titles": [{"id": "tt0111161", "primaryTitle": "Test Movie", "genres": []}]}"#;
        server
            .mock("GET", "/titles:batchGet")
            .match_query(Matcher::Any)
            .with_status(429)
            .with_header("content-type", "application/json")
            .with_body("Too Many Requests")
            .expect(1)
            .create();
        server
            .mock("GET", "/titles:batchGet")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_response)
            .expect(1)
            .create();
        let client = Client::new();
        let ids = vec!["tt0111161".to_string()];
        let result = fetch_movies(&client, &ids, &base_url).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // to avoid hitting real API
    async fn test_real_api() {
        let result = get_imdb_data_by_id("tt0111161").await;
        assert!(result.is_ok());
        let imdb = result.unwrap();
        assert_eq!(imdb.imdb_id, "tt0111161");
    }
    #[tokio::test]
    async fn test_process_movies_batching() {
        let mut server = mockito::Server::new_async().await;
        let base_url = server.url();
        let movie_ids = vec![
            "tt1".to_string(),
            "tt2".to_string(),
            "tt3".to_string(),
            "tt4".to_string(),
            "tt5".to_string(),
            "tt6".to_string(),
        ];
        // Mock for 5 IDs
        let json5 = r#"{"titles": [{"id": "tt1", "primaryTitle": "Movie1", "genres": []}, {"id": "tt2", "primaryTitle": "Movie2", "genres": []}, {"id": "tt3", "primaryTitle": "Movie3", "genres": []}, {"id": "tt4", "primaryTitle": "Movie4", "genres": []}, {"id": "tt5", "primaryTitle": "Movie5", "genres": []}]}"#;
        server
            .mock("GET", "/titles:batchGet")
            .match_query(Matcher::Regex(
                r"(titleIds=[^&]+&){4}titleIds=[^&]+".to_string(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json5)
            .expect(1)
            .create();
        // Mock for 1 ID
        let json1 = r#"{"titles": [{"id": "tt6", "primaryTitle": "Movie6", "genres": []}]}"#;
        server
            .mock("GET", "/titles:batchGet")
            .match_query(Matcher::Regex(r"titleIds=[^&]+".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json1)
            .expect(1)
            .create();
        let result = process_movies_inner(movie_ids, &base_url).await;
        assert!(result.is_ok());
        let imdbs = result.unwrap();
        assert_eq!(imdbs.len(), 6);
        let ids: HashSet<String> = imdbs.iter().map(|i| i.imdb_id.clone()).collect();
        assert!(ids.contains("tt1"));
        assert!(ids.contains("tt6"));
    }
}
