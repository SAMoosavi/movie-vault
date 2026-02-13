use super::{ImdbSyncStats, freeimdb};
use crate::data_model::Media;
use anyhow::{Result, anyhow};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri_plugin_http::reqwest::Client;
use tauri_plugin_log::log::{error, info, warn};

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
    info!("Fetching IMDB ID for: {}", media.name);

    let result: MovieSearchResult = client
        .get("https://imdb.iamidiotareyoutoo.com/search")
        .query(&[("q", &media.name)])
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send request for {}: {}", media.name, e);
            e
        })?
        .json()
        .await
        .map_err(|e| {
            error!("Failed to parse JSON response for {}: {}", media.name, e);
            e
        })?;

    if !result.ok {
        let err_msg = format!("API error: {}", result.error_code);
        error!("{}", err_msg);
        return Err(anyhow!(err_msg));
    }

    let movies = result.description;
    let matched = media
        .year
        .and_then(|year| movies.iter().find(|m| m.year == Some(year)))
        .or_else(|| movies.first());

    matched
        .map(|m| {
            info!("Found IMDB ID: {} for {}", m.imdb_id, media.name);
            m.imdb_id.clone()
        })
        .ok_or_else(|| {
            let err = anyhow!("No movies found for {}", media.name);
            error!("{}", err);
            err
        })
}

pub async fn set_imdb_data(medias: &mut [Media]) -> Result<ImdbSyncStats> {
    info!("Starting set_imdb_data for {} media items", medias.len());
    let client = Client::new();
    let mut stats = ImdbSyncStats {
        requested: medias.len(),
        ..ImdbSyncStats::default()
    };

    let results = join_all(medias.iter_mut().map(|media| {
        let client = client.clone();
        async move {
            info!("Searching IMDB ID for '{}'", media.name);
            match get_imdb_id(&client, media).await {
                Ok(id) => {
                    info!("Found IMDB ID {} for '{}'", id, media.name);
                    Ok((id, media))
                }
                Err(e) => {
                    warn!("Failed to get IMDB ID for '{}': {}", media.name, e);
                    Err(e)
                }
            }
        }
    }))
    .await;

    let mut pairs = Vec::new();
    for result in results {
        match result {
            Ok(pair) => pairs.push(pair),
            Err(err) => {
                stats.id_lookup_failed += 1;
                warn!("IMDB ID lookup failed: {}", err);
            }
        }
    }

    if pairs.is_empty() {
        warn!("No IMDB IDs were found for any media items");
        return Ok(stats);
    }

    let ids: Vec<_> = pairs.iter().map(|(id, _)| id.clone()).collect();
    info!("Processing {} IMDB IDs in batch", ids.len());

    match freeimdb::process_movies(ids).await {
        Ok(imdbs) => {
            info!("Successfully fetched {} IMDB records", imdbs.len());
            let imdb_by_id: HashMap<String, crate::data_model::Imdb> = imdbs
                .into_iter()
                .map(|imdb| (imdb.imdb_id.clone(), imdb))
                .collect();

            for (id, media) in pairs {
                if let Some(imdb) = imdb_by_id.get(&id) {
                    info!("Attaching IMDB data (id {}) to media '{}'", imdb.imdb_id, media.name);
                    media.imdb = Some(imdb.clone());
                    stats.enriched += 1;
                } else {
                    stats.batch_missing += 1;
                    warn!(
                        "No IMDB payload returned for id {} (media '{}')",
                        id, media.name
                    );
                }
            }
        }
        Err(err) => {
            stats.batch_fetch_failed += pairs.len();
            warn!(
                "Failed to fetch IMDB batch data for {} items: {}",
                pairs.len(),
                err
            );
        }
    }

    if stats.failed_total() > 0 {
        warn!(
            "IMDB enrichment completed with failures: requested={}, enriched={}, id_lookup_failed={}, batch_missing={}, batch_fetch_failed={}",
            stats.requested,
            stats.enriched,
            stats.id_lookup_failed,
            stats.batch_missing,
            stats.batch_fetch_failed
        );
    }

    Ok(stats)
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
        let stats = super::set_imdb_data(&mut medias).await.unwrap();
        assert_eq!(stats.requested, 2);
        assert_eq!(stats.enriched, 2);
        assert_eq!(stats.failed_total(), 0);

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
