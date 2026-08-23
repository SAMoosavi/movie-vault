use std::path::PathBuf;

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

// ponytail: heals medias inserted while the old IMDb API was dead; cheap SELECT once healed
async fn backfill_missing_imdb(db: &Sqlite, api_keys: &[String]) -> Result<usize, String> {
    let missing: Vec<Media> = db
        .get_all_medias()
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|m| m.imdb.is_none())
        .collect();

    if missing.is_empty() {
        return Ok(0);
    }
    info!("Backfilling IMDb data for {} medias", missing.len());

    let mut updated = 0usize;
    for chunk in missing.chunks(50) {
        let mut chunk = chunk.to_vec();
        fetch_imdb::set_imdb_data(&mut chunk, api_keys).await;

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

    info!("Starting sync_files for root: {}", root);

    if let Err(e) = backfill_missing_imdb(db, &api_keys).await {
        error!("IMDb backfill failed: {}", e);
    }

    if let Err(e) = media_scanner::sync_files(db).await {
        error!("media_scanner::sync_files failed: {}", e);
        return Err(e.to_string());
    }
    info!("media_scanner::sync_files completed");

    let found_files = match media_scanner::find_movies(db, PathBuf::from(&root)).await {
        Ok(files) => {
            info!("Found {} candidate files under {}", files.len(), root);
            files
        }
        Err(e) => {
            error!("media_scanner::find_movies failed: {}", e);
            return Err(e.to_string());
        }
    };

    if found_files.is_empty() {
        warn!("No candidate files found under {}", root);
    }

    let metadata = metadata_extractor::get_metadata(&found_files);
    let total = metadata.len();
    info!("Extracted metadata for {} items", total);

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

        fetch_imdb::set_imdb_data(&mut chunk, &api_keys).await;
        info!("Fetched IMDB data for chunk {}", i + 1);

        match db.insert_medias(&chunk) {
            Ok(_) => {
                inserted += chunk.len();
                info!(
                    "Inserted {} items from chunk {} (total inserted: {} / {})",
                    chunk.len(),
                    i + 1,
                    inserted,
                    total
                );
            }
            Err(e) => {
                error!("db.insert_medias failed on chunk {}: {}", i + 1, e);
                return Err(e.to_string());
            }
        }

        // emit progress to frontend
        let progress = SyncFileProgressBare { inserted, total };
        if let Err(e) = app_handle.emit("sync-progress", progress) {
            error!("Failed to emit sync-progress: {:?}", e);
        } else {
            info!("Emitted sync-progress: {}/{}", inserted, total);
        }
    }

    info!(
        "sync_files completed successfully, inserted {} items",
        inserted
    );
    Ok(inserted)
}

#[tauri::command]
fn get_countries(state: tauri::State<'_, AppState>) -> Result<Vec<NumericalString>, String> {
    let db = &state.db;
    info!("get_countries called");
    match db.get_countries() {
        Ok(res) => {
            info!("get_countries returned {} entries", res.len());
            Ok(res)
        }
        Err(e) => {
            error!("get_countries failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn get_genres(state: tauri::State<'_, AppState>) -> Result<Vec<NumericalString>, String> {
    let db = &state.db;
    info!("get_genres called");
    match db.get_genres() {
        Ok(res) => {
            info!("get_genres returned {} entries", res.len());
            Ok(res)
        }
        Err(e) => {
            error!("get_genres failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn get_people(state: tauri::State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    let db = &state.db;
    info!("get_people called");
    match db.get_people() {
        Ok(res) => {
            info!("get_people returned {} entries", res.len());
            Ok(res)
        }
        Err(e) => {
            error!("get_people failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn filter_medias(
    filters: FilterValues,
    page: u32,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<data_model::Media>, String> {
    let db = &state.db;
    info!(
        "filter_medias called with page {} and filters: {:?}",
        page, filters
    );
    match db.filter_medias(&filters, page) {
        Ok(res) => {
            info!(
                "filter_medias returned {} items for page {}",
                res.len(),
                page
            );
            Ok(res)
        }
        Err(e) => {
            error!("filter_medias failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn get_media_by_id(
    media_id: IdType,
    state: tauri::State<'_, AppState>,
) -> Result<data_model::Media, String> {
    let db = &state.db;
    info!("get_media_by_id called with media_id: {:?}", media_id);
    match db.get_media_by_id(media_id).map_err(|e| e.to_string())? {
        Some(media) => {
            info!("Media found: {:?}", media);
            Ok(media)
        }
        None => {
            warn!("Media not found for id: {:?}", media_id);
            Err("movie not found".to_string())
        }
    }
}

#[tauri::command]
async fn update_media_imdb(
    media_id: IdType,
    imdb_id: &str,
    api_keys: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<IdType, String> {
    let db = &state.db;
    info!(
        "Updating media with ID: {} and IMDb ID: {}",
        media_id, imdb_id
    );

    let imdb = fetch_imdb::get_imdb_data_by_id(imdb_id, &api_keys)
        .await
        .map_err(|e| e.to_string())?;

    db.insert_imdb(&imdb).map_err(|e| e.to_string())?;
    info!("Inserted IMDb data for media ID: {}", media_id);

    match db
        .update_media_imdb(media_id, imdb_id)
        .map_err(|e| e.to_string())
    {
        Ok(id) => {
            info!("update_media_imdb  updated");
            Ok(id)
        }
        Err(e) => {
            error!("update_media_imdb failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn create_media_from_imdb(
    imdb_id: &str,
    api_keys: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<IdType, String> {
    let db = &state.db;
    info!("Creating media from IMDb ID: {}", imdb_id);

    let imdb = fetch_imdb::get_imdb_data_by_id(imdb_id, &api_keys)
        .await
        .map_err(|e| {
            error!("Failed to fetch IMDb data for ID {}: {}", imdb_id, e);
            e.to_string()
        })?;

    let media = Media {
        name: imdb.title.clone(),
        year: Some(imdb.year),
        imdb: Some(imdb.clone()),
        ..Media::default()
    };

    db.insert_media(&media).map_err(|e| {
        error!("Failed to insert media for IMDb ID {}: {}", imdb_id, e);
        e.to_string()
    })?;

    info!("Successfully created media from IMDb ID: {}", imdb_id);
    Ok(media.id)
}

#[tauri::command]
fn update_watch_list(
    media_id: IdType,
    watch_list: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    info!(
        "Updating watch list for media ID: {} to {}",
        media_id, watch_list
    );
    db.update_watch_list(media_id, watch_list).map_err(|e| {
        error!(
            "Failed to update watch list for media ID {}: {}",
            media_id, e
        );
        e.to_string()
    })?;
    info!("Successfully updated watch list for media ID: {}", media_id);
    Ok(())
}

#[tauri::command]
fn update_media_watched(
    media_id: IdType,
    watched: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    info!(
        "Updating watched status for media ID: {} to {}",
        media_id, watched
    );
    db.update_media_watched(media_id, watched).map_err(|e| {
        error!(
            "Failed to update watched status for media ID {}: {}",
            media_id, e
        );
        e.to_string()
    })?;
    info!(
        "Successfully updated watched status for media ID: {}",
        media_id
    );
    info!(
        "Successfully updated watched status for media ID: {}",
        media_id
    );
    Ok(())
}

#[tauri::command]
fn update_season_watched(
    season_id: IdType,
    watched: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    info!(
        "Updating watched status for season ID: {} to {}",
        season_id, watched
    );
    db.update_season_watched(season_id, watched).map_err(|e| {
        error!(
            "Failed to update watched status for season ID {}: {}",
            season_id, e
        );
        e.to_string()
    })?;
    info!(
        "Successfully updated watched status for season ID: {}",
        season_id
    );
    Ok(())
}

#[tauri::command]
fn update_episode_watched(
    episode_id: IdType,
    watched: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    info!(
        "Updating watched status for episode ID: {} to {}",
        episode_id, watched
    );
    db.update_episode_watched(episode_id, watched)
        .map_err(|e| {
            error!(
                "Failed to update watched status for episode ID {}: {}",
                episode_id, e
            );
            e.to_string()
        })?;
    Ok(())
}

#[tauri::command]
fn update_media_my_ranking(
    media_id: IdType,
    my_ranking: u8,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    info!(
        "Updating my ranking for media ID: {} to {}",
        media_id, my_ranking
    );
    db.update_media_my_ranking(media_id, my_ranking)
        .map_err(|e| {
            error!(
                "Failed to update my ranking for media ID {}: {}",
                media_id, e
            );
            e.to_string()
        })?;
    info!("Successfully updated my ranking for media ID: {}", media_id);
    Ok(())
}

#[tauri::command]
fn get_tags(state: tauri::State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let db = &state.db;
    info!("get_tags called");
    db.get_tags().map_err(|e| {
        error!("get_tags failed: {}", e);
        e.to_string()
    })
}

#[tauri::command]
fn remove_tag(tag_id: IdType, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let db = &state.db;
    info!("Removing tag with ID: {}", tag_id);
    db.remove_tag(tag_id).map_err(|e| {
        error!("Failed to remove tag with ID {}: {}", tag_id, e);
        e.to_string()
    })?;
    info!("Successfully removed tag with ID: {}", tag_id);
    Ok(())
}

#[tauri::command]
fn update_tag(tag: Tag, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let db = &state.db;
    info!("Updating tag: {:?}", tag);
    db.update_tag(&tag).map_err(|e| {
        error!("Failed to update tag: {}", e);
        e.to_string()
    })?;
    info!("Successfully updated tag");
    Ok(())
}

#[tauri::command]
fn get_medias_by_tag(
    tag_id: IdType,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<data_model::Media>, String> {
    let db = &state.db;
    info!("get_medias_by_tag called with tag_id: {:?}", tag_id);
    db.get_medias_by_tag(tag_id).map_err(|e| {
        error!("get_medias_by_tag failed: {}", e);
        e.to_string()
    })
}

#[tauri::command]
fn insert_tag(tag: Tag, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let db = &state.db;
    info!("Inserting tag: {:?}", tag);
    db.insert_tag(&tag).map_err(|e| {
        error!("Failed to insert tag: {}", e);
        e.to_string()
    })
}

#[tauri::command]
fn insert_media_tag(
    media_id: IdType,
    tag_id: IdType,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    info!(
        "Inserting media tag - media_id: {:?}, tag_id: {:?}",
        media_id, tag_id
    );
    db.insert_media_tag(media_id, tag_id).map_err(|e| {
        error!(
            "Failed to insert media tag - media_id: {:?}, tag_id: {:?}: {}",
            media_id, tag_id, e
        );
        e.to_string()
    })
}

#[tauri::command]
fn remove_media_tag(
    media_id: IdType,
    tag_id: IdType,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    info!(
        "Removing media tag - media_id: {:?}, tag_id: {:?}",
        media_id, tag_id
    );
    db.remove_media_tag(media_id, tag_id).map_err(|e| {
        error!(
            "Failed to remove media tag - media_id: {:?}, tag_id: {:?}: {}",
            media_id, tag_id, e
        );
        e.to_string()
    })
}

#[tauri::command]
fn delete_media(media_id: IdType, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let db = &state.db;
    info!("Deleting media with ID: {:?}", media_id);
    db.delete_media(media_id).map_err(|e| {
        error!("Failed to delete media with ID {:?}: {}", media_id, e);
        e.to_string()
    })
}

#[tauri::command]
async fn export_data(file_path: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    use std::fs;

    let db = &state.db;
    info!("export_data called with file_path: {}", file_path);

    let medias = db.get_all_medias().map_err(|e| {
        error!("Failed to get all medias: {}", e);
        e.to_string()
    })?;
    let tags = db.get_tags().map_err(|e| {
        error!("Failed to get tags: {}", e);
        e.to_string()
    })?;

    let data = ExportedData { medias, tags };
    let json = serde_json::to_string(&data).map_err(|e| {
        error!("Failed to serialize data to JSON: {}", e);
        e.to_string()
    })?;

    fs::write(&file_path, json).map_err(|e| {
        error!("Failed to write file {}: {}", file_path, e);
        format!("Failed to write file: {}", e)
    })?;

    info!("Successfully exported data to {}", file_path);
    Ok(())
}

#[tauri::command]
fn import_data(data: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let db = &state.db;
    info!("import_data called");

    let exported: ExportedData = serde_json::from_str(&data).map_err(|e| {
        error!("Failed to deserialize import data: {}", e);
        e.to_string()
    })?;

    db.import_data(&exported).map_err(|e| {
        error!("Failed to import data: {}", e);
        e.to_string()
    })?;

    info!("Successfully imported data");
    Ok(())
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
                .max_file_size(1_000_000) // 1MB
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
