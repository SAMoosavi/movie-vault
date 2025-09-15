use anyhow::{Result, anyhow};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tauri_plugin_http::reqwest::Client;

use super::freeimdb;
use crate::data_model::Media;

#[derive(Serialize, Deserialize)]
struct MovieSearchResult {
    ok: bool,
    description: Vec<SearchedMovie>,
    error_code: i32,
}

#[derive(Serialize, Deserialize)]
struct SearchedMovie {
    #[serde(rename = "#YEAR")]
    year: Option<i32>,
    #[serde(rename = "#IMDB_ID")]
    imdb_id: String,
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
    for (searched_movie, media) in result.into_iter().flatten() {
        pairs.push((searched_movie, media));
    }

    let ids: Vec<String> = pairs.iter().map(|(imdb_id, _)| imdb_id.clone()).collect();

    match freeimdb::process_movies(ids).await {
        Ok(imdbs) => {
            for imdb in imdbs {
                if let Some((_, media)) = pairs
                    .iter_mut()
                    .find(|(imdb_id, _)| *imdb_id == imdb.imdb_id)
                {
                    media.imdb = Some(imdb);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch movies batch: {}", e);
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
        assert_eq!(imdb.year, "2011");

        let new_m2 = &medias[1];
        assert_eq!(new_m2.name, "3 days to kill");

        let imdb = new_m2.imdb.as_ref().unwrap();
        assert_eq!(imdb.r#type, "movie");
        assert_eq!(imdb.year, "2014");
    }
}
