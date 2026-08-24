use std::path::PathBuf;

use anyhow::{Context, anyhow};
use serde::Serialize;
use tauri::{Emitter, Manager};
use tauri_plugin_log::{
    RotationStrategy, Target, TargetKind, TimezoneStrategy,
    log::{LevelFilter, error, info, warn},
};

use crate::data_model::{IdType, Media};
use crate::db::{NumericalString, Sqlite};
use crate::{
    data_model::Tag,
    db::{DB, FilterValues},
};

#[derive(serde::Serialize, serde::Deserialize)]
struct ExportedData {
    medias: Vec<Media>,
    tags: Vec<Tag>,
}

mod data_model;
mod db;
mod fetch_imdb;
mod media_scanner;
mod metadata_extractor;

struct AppState {
    db: Sqlite,
    // serializes sync_files so startup sync, mount-poll and watcher events can't interleave
    sync_lock: tokio::sync::Mutex<()>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncFileProgressBare {
    processed: usize,
    total: usize,
    inserted_new: usize,
    merged_existing: usize,
    imdb_enriched: usize,
    imdb_failures: usize,
}

fn to_frontend_error(context: &str, err: anyhow::Error) -> String {
    let msg = format!("{context}: {err:#}");
    error!("{msg}");
    msg
}

// ponytail: heals medias inserted while the old IMDb API was dead; cheap SELECT once healed
// ponytail: treats entries with people missing photos/IDs as stale (pre-OMDb rows); drop once all users are migrated
fn needs_imdb_refresh(media: &Media) -> bool {
    match &media.imdb {
        None => true,
        Some(imdb) => [&imdb.actors, &imdb.writers, &imdb.directors]
            .iter()
            .any(|people| !people.is_empty() && people.iter().any(|p| p.url.is_empty())),
    }
}

async fn backfill_missing_imdb(db: &Sqlite, api_keys: &[String]) -> Result<usize, String> {
    let missing: Vec<Media> = db
        .get_all_medias()
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(needs_imdb_refresh)
        .collect();

    if missing.is_empty() {
        return Ok(0);
    }
    info!("Refreshing IMDb data for {} medias", missing.len());

    let mut updated = 0usize;
    for chunk in missing.chunks(50) {
        let mut chunk = chunk.to_vec();
        // refetch by existing ID when known so titles resolve even with odd names
        fetch_imdb::refresh_by_ids(&mut chunk, api_keys).await;

        for media in &chunk {
            if let Some(imdb) = &media.imdb {
                if imdb.imdb_id.is_empty() {
                    continue;
                }
                db.insert_imdb(imdb).map_err(|e| e.to_string())?;
                db.update_media_imdb(media.id, &imdb.imdb_id)
                    .map_err(|e| e.to_string())?;
                updated += 1;
            }
        }
        info!("Backfilled {}/{} medias", updated, missing.len());
    }

    Ok(updated)
}

#[tauri::command]
async fn sync_files(
    root: String,
    api_keys: Vec<String>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<usize, String> {
    let db = &state.db;
    let _sync_guard = state.sync_lock.lock().await;

    info!("Starting sync_files for root: {root}");

    if !PathBuf::from(&root).exists() {
        // ponytail: unmounted drive is not a deletion; keep rows so remount recovers without a full rescan
        warn!("Root {} does not exist (unmounted?); skipping sync", root);
        return Ok(0);
    }

    let result: anyhow::Result<usize> = async {
        media_scanner::sync_files(db)
            .await
            .context("failed to sync files with DB")?;
        info!("media_scanner::sync_files completed");

        if let Err(e) = backfill_missing_imdb(db, &api_keys).await {
            error!("IMDb backfill failed: {}", e);
        }

        media_scanner::sync_files(db)
            .await
            .context("failed to sync files with DB")?;
        info!("media_scanner::sync_files completed");

        let found_files = media_scanner::find_movies(db, PathBuf::from(&root))
            .await
            .with_context(|| format!("failed to find movie files under root={root}"))?;

        info!("Found {} candidate files under {}", found_files.len(), root);
        if found_files.is_empty() {
            warn!("No candidate files found under {root}");
        }

        let metadata = metadata_extractor::get_metadata(&found_files);
        let total = metadata.len();
        info!("Extracted metadata for {total} items");

        if total == 0 {
            warn!("No metadata extracted from candidate files");
        }

        let chunk_size = 50;
        let mut processed = 0usize;
        let mut inserted_new = 0usize;
        let mut merged_existing = 0usize;
        let mut imdb_enriched = 0usize;
        let mut imdb_failures = 0usize;

        for (i, chunk_slice) in metadata.chunks(chunk_size).enumerate() {
            let mut chunk = chunk_slice.to_vec();
            info!(
                "Processing chunk {} (items {}..{})",
                i + 1,
                processed + 1,
                processed + chunk.len()
            );

            let imdb_stats = fetch_imdb::set_imdb_data(&mut chunk, &api_keys)
                .await
                .with_context(|| format!("failed to enrich imdb data for chunk {}", i + 1))?;
            imdb_enriched += imdb_stats.enriched;
            imdb_failures += imdb_stats.failed_total();

            if imdb_stats.failed_total() > 0 {
                warn!(
                    "Chunk {} IMDB enrichment partial failure: requested={}, enriched={}, id_lookup_failed={}, batch_missing={}, batch_fetch_failed={}",
                    i + 1,
                    imdb_stats.requested,
                    imdb_stats.enriched,
                    imdb_stats.id_lookup_failed,
                    imdb_stats.batch_missing,
                    imdb_stats.batch_fetch_failed
                );
            } else {
                info!("Fetched IMDB data for chunk {}", i + 1);
            }

            let db_stats = db
                .insert_medias(&chunk)
                .with_context(|| format!("failed to insert chunk {} into DB", i + 1))?;

            processed += chunk.len();
            inserted_new += db_stats.inserted_new;
            merged_existing += db_stats.merged_existing;
            info!(
                "Chunk {} DB upsert stats: inserted_new={}, merged_existing={} (processed {} / {})",
                i + 1,
                db_stats.inserted_new,
                db_stats.merged_existing,
                processed,
                total,
            );

            let progress = SyncFileProgressBare {
                processed,
                total,
                inserted_new,
                merged_existing,
                imdb_enriched,
                imdb_failures,
            };
            if let Err(e) = app_handle.emit("sync-progress", progress) {
                error!("Failed to emit sync-progress for chunk {}: {}", i + 1, e);
            } else {
                info!(
                    "Emitted sync-progress: processed={}/{}, inserted_new={}, merged_existing={}, imdb_enriched={}, imdb_failures={}",
                    processed, total, inserted_new, merged_existing, imdb_enriched, imdb_failures
                );
            }
        }

        info!(
            "sync_files completed successfully: processed={}, inserted_new={}, merged_existing={}, imdb_enriched={}, imdb_failures={}",
            processed, inserted_new, merged_existing, imdb_enriched, imdb_failures
        );
        Ok(inserted_new)
    }
    .await;

    result.map_err(|e| to_frontend_error("sync_files failed", e))
}

#[tauri::command]
fn get_countries(state: tauri::State<'_, AppState>) -> Result<Vec<NumericalString>, String> {
    info!("get_countries called");
    state
        .db
        .get_countries()
        .inspect(|res| info!("get_countries returned {} entries", res.len()))
        .map_err(|e| to_frontend_error("get_countries failed", e))
}

#[tauri::command]
fn get_genres(state: tauri::State<'_, AppState>) -> Result<Vec<NumericalString>, String> {
    info!("get_genres called");
    state
        .db
        .get_genres()
        .inspect(|res| info!("get_genres returned {} entries", res.len()))
        .map_err(|e| to_frontend_error("get_genres failed", e))
}

#[tauri::command]
fn get_people(state: tauri::State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    info!("get_people called");
    state
        .db
        .get_people()
        .inspect(|res| info!("get_people returned {} entries", res.len()))
        .map_err(|e| to_frontend_error("get_people failed", e))
}

#[tauri::command]
fn filter_medias(
    filters: FilterValues,
    page: u32,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<data_model::Media>, String> {
    info!(
        "filter_medias called with page {} and filters: {:?}",
        page, filters
    );
    state
        .db
        .filter_medias(&filters, page)
        .inspect(|res| {
            info!(
                "filter_medias returned {} items for page {}",
                res.len(),
                page
            )
        })
        .map_err(|e| to_frontend_error("filter_medias failed", e))
}

#[tauri::command]
fn get_media_by_id(
    media_id: IdType,
    state: tauri::State<'_, AppState>,
) -> Result<data_model::Media, String> {
    info!("get_media_by_id called with media_id={media_id}");

    let result: anyhow::Result<data_model::Media> = (|| {
        let media = state
            .db
            .get_media_by_id(media_id)
            .with_context(|| format!("failed to load media_id={media_id}"))?
            .ok_or_else(|| anyhow!("media_id={media_id} not found"))?;

        info!("Media found for media_id={media_id}");
        Ok(media)
    })();

    result.map_err(|e| to_frontend_error("get_media_by_id failed", e))
}

#[tauri::command]
async fn search_imdb(
    query: String,
    api_keys: Vec<String>,
) -> Result<Vec<fetch_imdb::SearchResult>, String> {
    fetch_imdb::search_imdb(&query, &api_keys)
        .await
        .map_err(|e| {
            error!("IMDb search failed for '{}': {}", query, e);
            e.to_string()
        })
}

#[tauri::command]
async fn update_media_imdb(
    media_id: IdType,
    imdb_id: &str,
    api_keys: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<IdType, String> {
    info!("update_media_imdb called with media_id={media_id}, imdb_id={imdb_id}");

    let result: anyhow::Result<IdType> = async {
        let imdb = fetch_imdb::get_imdb_data_by_id(imdb_id, &api_keys)
            .await
            .with_context(|| format!("failed to fetch imdb payload for imdb_id={imdb_id}"))?;

        state
            .db
            .insert_imdb(&imdb)
            .with_context(|| format!("failed to persist imdb_id={} in DB", imdb.imdb_id))?;

        let updated_id = state
            .db
            .update_media_imdb(media_id, imdb_id)
            .with_context(|| {
                format!("failed to attach imdb_id={imdb_id} to media_id={media_id}")
            })?;

        info!("Successfully updated media imdb for media_id={media_id}");
        Ok(updated_id)
    }
    .await;

    result.map_err(|e| to_frontend_error("update_media_imdb failed", e))
}

#[tauri::command]
async fn create_media_from_imdb(
    imdb_id: &str,
    api_keys: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<IdType, String> {
    info!("create_media_from_imdb called with imdb_id={imdb_id}");

    let result: anyhow::Result<IdType> = async {
        let imdb = fetch_imdb::get_imdb_data_by_id(imdb_id, &api_keys)
            .await
            .with_context(|| format!("failed to fetch imdb payload for imdb_id={imdb_id}"))?;

        let media = Media {
            name: imdb.title.clone(),
            year: Some(imdb.year),
            imdb: Some(imdb.clone()),
            ..Media::default()
        };

        let new_media_id = state
            .db
            .insert_media(&media)
            .with_context(|| format!("failed to insert media created from imdb_id={imdb_id}"))?;

        info!("Successfully created media from IMDb ID: {imdb_id}, new media_id={new_media_id}");
        Ok(new_media_id)
    }
    .await;

    result.map_err(|e| to_frontend_error("create_media_from_imdb failed", e))
}

#[tauri::command]
fn update_watch_list(
    media_id: IdType,
    watch_list: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("update_watch_list called with media_id={media_id}, watch_list={watch_list}");
    state
        .db
        .update_watch_list(media_id, watch_list)
        .with_context(|| format!("failed to update watch_list for media_id={media_id}"))
        .map_err(|e| to_frontend_error("update_watch_list failed", e))?;

    info!("Successfully updated watch_list for media_id={media_id}");
    Ok(())
}

#[tauri::command]
fn update_media_watched(
    media_id: IdType,
    watched: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("update_media_watched called with media_id={media_id}, watched={watched}");
    state
        .db
        .update_media_watched(media_id, watched)
        .with_context(|| format!("failed to update watched status for media_id={media_id}"))
        .map_err(|e| to_frontend_error("update_media_watched failed", e))?;

    info!("Successfully updated watched status for media_id={media_id}");
    Ok(())
}

#[tauri::command]
fn update_season_watched(
    season_id: IdType,
    watched: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("update_season_watched called with season_id={season_id}, watched={watched}");
    state
        .db
        .update_season_watched(season_id, watched)
        .with_context(|| format!("failed to update watched status for season_id={season_id}"))
        .map_err(|e| to_frontend_error("update_season_watched failed", e))?;

    info!("Successfully updated watched status for season_id={season_id}");
    Ok(())
}

#[tauri::command]
fn update_episode_watched(
    episode_id: IdType,
    watched: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("update_episode_watched called with episode_id={episode_id}, watched={watched}");
    state
        .db
        .update_episode_watched(episode_id, watched)
        .with_context(|| format!("failed to update watched status for episode_id={episode_id}"))
        .map_err(|e| to_frontend_error("update_episode_watched failed", e))?;

    info!("Successfully updated watched status for episode_id={episode_id}");
    Ok(())
}

#[tauri::command]
fn update_media_my_ranking(
    media_id: IdType,
    my_ranking: u8,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("update_media_my_ranking called with media_id={media_id}, my_ranking={my_ranking}");
    if my_ranking > 10 {
        return Err(to_frontend_error(
            "update_media_my_ranking failed",
            anyhow!("invalid my_ranking={my_ranking}; expected range 0..=10"),
        ));
    }

    state
        .db
        .update_media_my_ranking(media_id, my_ranking)
        .with_context(|| format!("failed to update my_ranking for media_id={media_id}"))
        .map_err(|e| to_frontend_error("update_media_my_ranking failed", e))?;

    info!("Successfully updated my_ranking for media_id={media_id}");
    Ok(())
}

#[tauri::command]
fn get_tags(state: tauri::State<'_, AppState>) -> Result<Vec<Tag>, String> {
    info!("get_tags called");
    state
        .db
        .get_tags()
        .map_err(|e| to_frontend_error("get_tags failed", e))
}

#[tauri::command]
fn remove_tag(tag_id: IdType, state: tauri::State<'_, AppState>) -> Result<(), String> {
    info!("remove_tag called with tag_id={tag_id}");
    state
        .db
        .remove_tag(tag_id)
        .with_context(|| format!("failed to remove tag_id={tag_id}"))
        .map_err(|e| to_frontend_error("remove_tag failed", e))?;

    info!("Successfully removed tag_id={tag_id}");
    Ok(())
}

#[tauri::command]
fn update_tag(tag: Tag, state: tauri::State<'_, AppState>) -> Result<(), String> {
    info!("update_tag called for tag_id={}, name={}", tag.id, tag.name);
    state
        .db
        .update_tag(&tag)
        .with_context(|| format!("failed to update tag_id={}", tag.id))
        .map_err(|e| to_frontend_error("update_tag failed", e))?;

    info!("Successfully updated tag_id={}", tag.id);
    Ok(())
}

#[tauri::command]
fn get_medias_by_tag(
    tag_id: IdType,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<data_model::Media>, String> {
    info!("get_medias_by_tag called with tag_id={tag_id}");
    state
        .db
        .get_medias_by_tag(tag_id)
        .with_context(|| format!("failed to fetch medias for tag_id={tag_id}"))
        .map_err(|e| to_frontend_error("get_medias_by_tag failed", e))
}

#[tauri::command]
fn insert_tag(tag: Tag, state: tauri::State<'_, AppState>) -> Result<(), String> {
    info!("insert_tag called for name={}", tag.name);
    state
        .db
        .insert_tag(&tag)
        .with_context(|| format!("failed to insert tag name={}", tag.name))
        .map_err(|e| to_frontend_error("insert_tag failed", e))
}

#[tauri::command]
fn insert_media_tag(
    media_id: IdType,
    tag_id: IdType,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("insert_media_tag called with media_id={media_id}, tag_id={tag_id}");
    state
        .db
        .insert_media_tag(media_id, tag_id)
        .with_context(|| {
            format!("failed to insert tag relation media_id={media_id}, tag_id={tag_id}")
        })
        .map_err(|e| to_frontend_error("insert_media_tag failed", e))
}

#[tauri::command]
fn remove_media_tag(
    media_id: IdType,
    tag_id: IdType,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("remove_media_tag called with media_id={media_id}, tag_id={tag_id}");
    state
        .db
        .remove_media_tag(media_id, tag_id)
        .with_context(|| {
            format!("failed to remove tag relation media_id={media_id}, tag_id={tag_id}")
        })
        .map_err(|e| to_frontend_error("remove_media_tag failed", e))
}

#[tauri::command]
fn delete_media(media_id: IdType, state: tauri::State<'_, AppState>) -> Result<(), String> {
    info!("delete_media called with media_id={media_id}");
    state
        .db
        .delete_media(media_id)
        .with_context(|| format!("failed to delete media_id={media_id}"))
        .map_err(|e| to_frontend_error("delete_media failed", e))
}

#[tauri::command]
async fn export_data(file_path: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    use std::fs;

    info!("export_data called with file_path={file_path}");
    let result: anyhow::Result<()> = (|| {
        let medias = state
            .db
            .get_all_medias()
            .context("failed to fetch all medias for export")?;
        let tags = state
            .db
            .get_tags()
            .context("failed to fetch tags for export")?;

        let data = ExportedData { medias, tags };
        let json = serde_json::to_string(&data).context("failed to serialize export payload")?;

        fs::write(&file_path, json)
            .with_context(|| format!("failed to write export file at path={file_path}"))?;

        info!("Successfully exported data to {file_path}");
        Ok(())
    })();

    result.map_err(|e| to_frontend_error("export_data failed", e))
}

#[tauri::command]
fn import_data(data: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    info!("import_data called");

    let result: anyhow::Result<()> = (|| {
        let exported: ExportedData =
            serde_json::from_str(&data).context("failed to deserialize import payload")?;

        state
            .db
            .import_data(&exported)
            .context("failed to import data into DB")?;

        info!("Successfully imported data");
        Ok(())
    })();

    result.map_err(|e| to_frontend_error("import_data failed", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .format(|out, message, record| {
                    out.finish(format_args!("[{}] {}", record.level(), message))
                })
                .level(if cfg!(debug_assertions) {
                    LevelFilter::Debug
                } else {
                    LevelFilter::Error
                })
                .targets([
                    Target::new(TargetKind::LogDir {
                        file_name: Some("app".into()),
                    }),
                    // Target::new(TargetKind::Stdout),
                ])
                .timezone_strategy(TimezoneStrategy::UseLocal)
                .rotation_strategy(RotationStrategy::KeepAll)
                .max_file_size(1_000_000)
                .build(),
        )
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            sync_files,
            get_countries,
            get_genres,
            filter_medias,
            get_media_by_id,
            search_imdb,
            get_people,
            update_media_imdb,
            create_media_from_imdb,
            update_media_watched,
            update_season_watched,
            update_episode_watched,
            update_media_my_ranking,
            update_watch_list,
            get_tags,
            remove_tag,
            update_tag,
            get_medias_by_tag,
            insert_tag,
            insert_media_tag,
            remove_media_tag,
            delete_media,
            export_data,
            import_data
        ])
        .setup(|app| {
            let db = Sqlite::from_app_handle(app.app_handle())?;
            app.manage(AppState {
                db,
                sync_lock: tokio::sync::Mutex::new(()),
            });
            info!("Tauri app setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod backfill_tests {
    use super::*;

    #[test]
    fn needs_imdb_refresh_flags_stale_people() {
        let person = |url: &str| crate::data_model::Person {
            id: url.to_string(),
            name: "X".to_string(),
            url: url.to_string(),
        };

        let mut media = Media::default();
        assert!(needs_imdb_refresh(&media), "no imdb at all");

        let imdb = crate::data_model::Imdb {
            actors: vec![person("http://photo")],
            ..Default::default()
        };
        media.imdb = Some(imdb.clone());
        assert!(!needs_imdb_refresh(&media));

        let stale = crate::data_model::Imdb {
            writers: vec![person("")],
            ..imdb
        };
        media.imdb = Some(stale);
        assert!(needs_imdb_refresh(&media), "writer without photo is stale");
    }
}
