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
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncFileProgressBare {
    inserted: usize,
    total: usize,
}

fn to_frontend_error(context: &str, err: anyhow::Error) -> String {
    let msg = format!("{context}: {err:#}");
    error!("{msg}");
    msg
}

#[tauri::command]
async fn sync_files(
    root: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<usize, String> {
    let db = &state.db;
    info!("Starting sync_files for root: {root}");

    let result: anyhow::Result<usize> = async {
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
        let mut inserted = 0usize;

        for (i, chunk_slice) in metadata.chunks(chunk_size).enumerate() {
            let mut chunk = chunk_slice.to_vec();
            info!(
                "Processing chunk {} (items {}..{})",
                i + 1,
                inserted + 1,
                inserted + chunk.len()
            );

            fetch_imdb::set_imdb_data(&mut chunk)
                .await
                .with_context(|| format!("failed to enrich imdb data for chunk {}", i + 1))?;
            info!("Fetched IMDB data for chunk {}", i + 1);

            db.insert_medias(&chunk)
                .with_context(|| format!("failed to insert chunk {} into DB", i + 1))?;

            inserted += chunk.len();
            info!(
                "Inserted {} items from chunk {} (total inserted: {} / {})",
                chunk.len(),
                i + 1,
                inserted,
                total
            );

            let progress = SyncFileProgressBare { inserted, total };
            if let Err(e) = app_handle.emit("sync-progress", progress) {
                error!("Failed to emit sync-progress for chunk {}: {}", i + 1, e);
            } else {
                info!("Emitted sync-progress: {inserted}/{total}");
            }
        }

        info!("sync_files completed successfully, inserted {inserted} items");
        Ok(inserted)
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
async fn update_media_imdb(
    media_id: IdType,
    imdb_id: &str,
    state: tauri::State<'_, AppState>,
) -> Result<IdType, String> {
    info!("update_media_imdb called with media_id={media_id}, imdb_id={imdb_id}");

    let result: anyhow::Result<IdType> = async {
        let imdb = fetch_imdb::get_imdb_data_by_id(imdb_id)
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
    state: tauri::State<'_, AppState>,
) -> Result<IdType, String> {
    info!("create_media_from_imdb called with imdb_id={imdb_id}");

    let result: anyhow::Result<IdType> = async {
        let imdb = fetch_imdb::get_imdb_data_by_id(imdb_id)
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
            app.manage(AppState { db });
            info!("Tauri app setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
