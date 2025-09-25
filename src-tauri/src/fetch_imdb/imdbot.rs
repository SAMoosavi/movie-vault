use anyhow::{Result, anyhow};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tauri_plugin_http::reqwest::Client;

use super::freeimdb;
use crate::data_model::Media;

#[derive(Debug, Serialize, Deserialize)]
struct MovieSearchResult {
    ok: bool,
    description: Vec<SearchedMovie>,
    error_code: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchedMovie {
    #[serde(rename = "#YEAR")]
    year: Option<i32>,
    #[serde(rename = "#IMDB_ID")]
    imdb_id: String,
}

async fn get_imdb_id(client: &Client, media: &Media) -> Result<String> {
    let result: MovieSearchResult = client
        .get("https://imdb.iamidiotareyoutoo.com/search")
        .query(&[("q", &media.name)])
        .send()
        .await?
        .json()
        .await?;

    if !result.ok {
        return Err(anyhow!("API error: {}", result.error_code));
    }

    let movies = result.description;
    let matched = media
        .year
        .and_then(|year| movies.iter().find(|m| m.year == Some(year)))
        .or_else(|| movies.first());

    matched
        .map(|m| m.imdb_id.clone())
        .ok_or_else(|| anyhow!("No movies found"))
}

pub async fn set_imdb_data(medias: &mut [Media]) {
    let client = Client::new();

    let results = join_all(medias.iter_mut().map(|media| {
        let client = client.clone();
        async move {
            get_imdb_id(&client, media)
                .await
                .map(|imdb_id| (imdb_id, media))
        }
    }))
    .await;

    let mut pairs: Vec<_> = results.into_iter().filter_map(Result::ok).collect();

    let ids: Vec<_> = pairs.iter().map(|(id, _)| id.clone()).collect();

    match freeimdb::process_movies(ids).await {
        Ok(imdbs) => {
            for imdb in imdbs {
                if let Some((_, media)) = pairs.iter_mut().find(|(id, _)| id == &imdb.imdb_id) {
                    media.imdb = Some(imdb);
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to fetch movies batch: {err}");
        }
    }
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
    }

    #[tokio::test]
    async fn fetch_series_data() {
        let media = Media::from(PathBuf::from("black.mirror.s01.e01.480p.web-dl.x264.mkv"));
        let client = Client::new();

        let result = get_imdb_id(&client, &media).await;
        let imdb_id = result.unwrap();
        assert_eq!(imdb_id, "tt2085059");
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
        assert_eq!(imdb.year, 2011);

        let new_m2 = &medias[1];
        assert_eq!(new_m2.name, "3 days to kill");

        let imdb = new_m2.imdb.as_ref().unwrap();
        assert_eq!(imdb.r#type, "movie");
        assert_eq!(imdb.year, 2014);
    }
}
