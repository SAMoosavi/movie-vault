use crate::data_model::{Imdb, Media, Person};
use anyhow::{Result, anyhow};
use futures::future::join_all;
use futures::stream::{self, StreamExt};
use serde::{Deserialize, de::DeserializeOwned};
use tauri_plugin_http::reqwest::{Client, StatusCode};
use tauri_plugin_log::log::{error, info, warn};

const BASE_URL: &str = "https://www.omdbapi.com";
// ponytail: hardcoded shared OMDb key pool, rotated on 401/429; move to settings if the pool dies
const API_KEYS: &[&str] = &["e8f12113", "1e97e442"];
const CONCURRENCY: usize = 4;

async fn omdb_get<T: DeserializeOwned>(
    client: &Client,
    base_url: &str,
    query: &[(&str, String)],
) -> Result<T> {
    let url = format!("{}/", base_url);

    for key in API_KEYS {
        let mut params: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
        params.push(("apikey", key));

        let resp = client.get(&url).query(&params).send().await?;
        match resp.status() {
            s if s.is_success() => return Ok(resp.json::<T>().await?),
            StatusCode::UNAUTHORIZED | StatusCode::TOO_MANY_REQUESTS => {
                warn!("OMDb rejected key ({}), trying next key", resp.status());
            }
            s => return Err(anyhow!("OMDb request failed: {}", s)),
        }
    }

    Err(anyhow!("All OMDb API keys exhausted"))
}

fn field(value: Option<String>) -> String {
    value.filter(|s| s != "N/A").unwrap_or_default()
}

fn list(value: Option<String>) -> Vec<String> {
    field(value)
        .split(", ")
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn people(value: Option<String>) -> Vec<Person> {
    // ponytail: OMDb exposes no person IDs/photos; id=name, url empty
    list(value)
        .into_iter()
        .map(|name| Person {
            id: name.clone(),
            name,
            url: String::new(),
        })
        .collect()
}

fn parse_year(value: Option<&str>) -> i32 {
    value
        .unwrap_or_default()
        .split(['-', '\u{2013}'])
        .next()
        .and_then(|y| y.trim().parse().ok())
        .unwrap_or_default()
}

fn parse_votes(value: Option<String>) -> i32 {
    field(value).replace(',', "").parse().unwrap_or_default()
}

#[derive(Deserialize, Debug)]
struct SearchResponse {
    #[serde(rename = "Search", default)]
    search: Vec<SearchItem>,
}

#[derive(Deserialize, Debug)]
struct SearchItem {
    #[serde(rename = "imdbID")]
    imdb_id: String,
}

async fn get_imdb_id_inner(client: &Client, media: &Media, base_url: &str) -> Result<String> {
    info!("Searching OMDb for: {}", media.name);

    let mut query = vec![("s", media.name.clone()), ("r", "json".to_string())];
    if let Some(year) = media.year {
        query.push(("y", year.to_string()));
    }

    let mut result: SearchResponse = omdb_get(client, base_url, &query).await?;

    if result.search.is_empty() && media.year.is_some() {
        query.pop();
        result = omdb_get(client, base_url, &query).await?;
    }

    result
        .search
        .into_iter()
        .next()
        .map(|m| m.imdb_id)
        .ok_or_else(|| anyhow!("No results found for {}", media.name))
}

#[derive(Deserialize, Debug)]
struct TitleResponse {
    #[serde(rename = "Response")]
    ok: Option<String>,
    #[serde(rename = "Error")]
    error: Option<String>,
    #[serde(rename = "Title")]
    title: Option<String>,
    #[serde(rename = "Year")]
    year: Option<String>,
    #[serde(rename = "Plot")]
    plot: Option<String>,
    #[serde(rename = "Genre")]
    genre: Option<String>,
    #[serde(rename = "Country")]
    country: Option<String>,
    #[serde(rename = "Director")]
    director: Option<String>,
    #[serde(rename = "Writer")]
    writer: Option<String>,
    #[serde(rename = "Actors")]
    actors: Option<String>,
    #[serde(rename = "Poster")]
    poster: Option<String>,
    #[serde(rename = "imdbRating")]
    rating: Option<String>,
    #[serde(rename = "imdbVotes")]
    votes: Option<String>,
    #[serde(rename = "imdbID")]
    imdb_id: Option<String>,
    #[serde(rename = "Type")]
    r#type: Option<String>,
}

fn parse_title(resp: TitleResponse) -> Result<Imdb> {
    if resp.ok.as_deref() != Some("True") {
        return Err(anyhow!(
            "OMDb error: {}",
            resp.error.unwrap_or_else(|| "unknown".to_string())
        ));
    }

    Ok(Imdb {
        imdb_id: resp.imdb_id.unwrap_or_default(),
        r#type: resp.r#type.unwrap_or_default(),
        title: resp.title.unwrap_or_default(),
        year: parse_year(resp.year.as_deref()),
        plot: field(resp.plot),
        genres: list(resp.genre),
        countries: list(resp.country),
        poster: field(resp.poster),
        imdb_rating: field(resp.rating),
        imdb_votes: parse_votes(resp.votes),
        actors: people(resp.actors),
        writers: people(resp.writer),
        directors: people(resp.director),
    })
}

async fn get_imdb_data_by_id_inner(client: &Client, id: &str, base_url: &str) -> Result<Imdb> {
    info!("Fetching OMDb data for ID: {}", id);
    let query = [
        ("i", id.to_string()),
        ("plot", "full".to_string()),
        ("r", "json".to_string()),
    ];
    let resp: TitleResponse = omdb_get(client, base_url, &query).await?;
    parse_title(resp)
}

pub async fn get_imdb_data_by_id(id: &str) -> Result<Imdb> {
    get_imdb_data_by_id_inner(&Client::new(), id, BASE_URL).await
}

pub async fn set_imdb_data(medias: &mut [Media]) {
    set_imdb_data_inner(medias, BASE_URL).await;
}

async fn set_imdb_data_inner(medias: &mut [Media], base_url: &str) {
    info!("Setting IMDb data for {} media items", medias.len());
    let client = Client::new();

    let results = join_all(medias.iter_mut().map(|media| {
        let client = client.clone();
        async move { (get_imdb_id_inner(&client, media, base_url).await, media) }
    }))
    .await;

    let mut found: Vec<(String, &mut Media)> = results
        .into_iter()
        .filter_map(|(res, media)| match res {
            Ok(id) => Some((id, media)),
            Err(e) => {
                warn!("Failed to resolve IMDb ID for '{}': {}", media.name, e);
                None
            }
        })
        .collect();

    if found.is_empty() {
        warn!("No IMDb IDs found for any media items");
        return;
    }

    let ids: Vec<String> = found.iter().map(|(id, _)| id.clone()).collect();
    let imdbs: Vec<Imdb> = stream::iter(ids)
        .map(|id| {
            let client = client.clone();
            async move { get_imdb_data_by_id_inner(&client, &id, base_url).await }
        })
        .buffered(CONCURRENCY)
        .filter_map(|res| async move {
            match res {
                Ok(imdb) => Some(imdb),
                Err(e) => {
                    error!("Failed to fetch IMDb data: {}", e);
                    None
                }
            }
        })
        .collect()
        .await;

    for imdb in imdbs {
        if let Some((_, media)) = found.iter_mut().find(|(id, _)| *id == imdb.imdb_id) {
            media.imdb = Some(imdb);
        } else {
            error!(
                "Received IMDb data for id {} but no matching media was found",
                imdb.imdb_id
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_maps_first_result() {
        let mut server = mockito::Server::new_async().await;
        let base_url = server.url();
        server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"Search":[{"Title":"Black Mirror","Year":"2011\u2013","imdbID":"tt2085059","Type":"series"}],"totalResults":"25","Response":"True"}"#,
            )
            .create();
        let media = Media::from(std::path::PathBuf::from("black.mirror.s01.e01.mkv"));
        let result = get_imdb_id_inner(&Client::new(), &media, &base_url).await;
        assert_eq!(result.unwrap(), "tt2085059");
    }

    #[tokio::test]
    async fn test_search_not_found_is_error() {
        let mut server = mockito::Server::new_async().await;
        let base_url = server.url();
        server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"Response":"False","Error":"Movie not found!"}"#)
            .create();
        let media = Media::from(std::path::PathBuf::from("nonexistent.movie.2020.mkv"));
        assert!(
            get_imdb_id_inner(&Client::new(), &media, &base_url)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_title_response_parses_into_imdb() {
        let resp: TitleResponse = serde_json::from_str(
            r#"{
                "Title": "The Shawshank Redemption",
                "Year": "1994",
                "Plot": "Two imprisoned men bond.",
                "Genre": "Drama",
                "Country": "United States",
                "Director": "Frank Darabont",
                "Writer": "Stephen King, Frank Darabont",
                "Actors": "Tim Robbins, Morgan Freeman",
                "Poster": "https://example.com/poster.jpg",
                "imdbRating": "9.3",
                "imdbVotes": "2,800,000",
                "imdbID": "tt0111161",
                "Type": "movie",
                "Response": "True"
            }"#,
        )
        .unwrap();
        let imdb = parse_title(resp).unwrap();
        assert_eq!(imdb.imdb_id, "tt0111161");
        assert_eq!(imdb.title, "The Shawshank Redemption");
        assert_eq!(imdb.year, 1994);
        assert_eq!(imdb.genres, vec!["Drama".to_string()]);
        assert_eq!(imdb.countries, vec!["United States".to_string()]);
        assert_eq!(imdb.imdb_rating, "9.3");
        assert_eq!(imdb.imdb_votes, 2800000);
        assert_eq!(imdb.actors.len(), 2);
        assert_eq!(imdb.actors[0].name, "Tim Robbins");
        assert_eq!(imdb.writers.len(), 2);
        assert_eq!(imdb.directors.len(), 1);
        assert_eq!(imdb.poster, "https://example.com/poster.jpg");
    }

    #[tokio::test]
    async fn test_na_fields_become_empty() {
        let resp: TitleResponse = serde_json::from_str(
            r#"{
                "Title": "X", "Year": "N/A", "Genre": "N/A", "Country": "N/A",
                "Director": "N/A", "Writer": "N/A", "Actors": "N/A", "Poster": "N/A",
                "imdbRating": "N/A", "imdbVotes": "N/A", "imdbID": "tt1234567",
                "Type": "movie", "Response": "True"
            }"#,
        )
        .unwrap();
        let imdb = parse_title(resp).unwrap();
        assert_eq!(imdb.year, 0);
        assert!(imdb.genres.is_empty());
        assert!(imdb.actors.is_empty());
        assert!(imdb.poster.is_empty());
        assert_eq!(imdb.imdb_votes, 0);
    }

    #[tokio::test]
    async fn test_rotates_key_on_401() {
        let mut server = mockito::Server::new_async().await;
        let base_url = server.url();
        server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(r#"{"Response":"False","Error":"Invalid API key!"}"#)
            .expect(1)
            .create();
        server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"Search":[{"Title":"Test","Year":"2014","imdbID":"tt2172934","Type":"movie"}],"Response":"True"}"#,
            )
            .expect(1)
            .create();
        let media = Media::from(std::path::PathBuf::from("3.days.to.kill.2014.mkv"));
        let result = get_imdb_id_inner(&Client::new(), &media, &base_url).await;
        assert_eq!(result.unwrap(), "tt2172934");
    }

    #[tokio::test]
    async fn test_real_api_by_id() {
        let imdb = get_imdb_data_by_id("tt0111161").await.unwrap();
        assert_eq!(imdb.imdb_id, "tt0111161");
        assert_eq!(imdb.title, "The Shawshank Redemption");
    }

    #[tokio::test]
    async fn test_real_api_set_imdb_data() {
        let m1 = Media::from(std::path::PathBuf::from(
            "3.days.to.kill.2014.extended.720p.farsi.dubbed.film2media.mkv",
        ));
        let m2 = Media::from(std::path::PathBuf::from(
            "black.mirror.s01.e01.480p.web-dl.x264.mkv",
        ));
        let mut medias = vec![m1, m2];
        set_imdb_data(&mut medias).await;

        let movie = &medias[0];
        assert_eq!(movie.name, "3 days to kill");
        let imdb = movie.imdb.as_ref().unwrap();
        assert_eq!(imdb.r#type, "movie");
        assert_eq!(imdb.year, 2014);

        let series = &medias[1];
        assert_eq!(series.name, "black mirror");
        let imdb = series.imdb.as_ref().unwrap();
        assert_eq!(imdb.r#type, "series");
        assert_eq!(imdb.year, 2011);
    }
}
