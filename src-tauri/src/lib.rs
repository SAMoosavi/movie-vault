use tauri::Manager;

use crate::db::{DieselDb, NumericalString};
use crate::{
    data_model::Tag,
    db::{DB, FilterValues},
};

mod data_model;
mod db;
mod media_scanner;
mod metadata_extractor;
mod omdb_meta_data;

struct AppState {
    db: DieselDb,
}

#[tauri::command]
async fn sync_files(
    root: &str,
    api_key: &str,
    state: tauri::State<'_, AppState>,
) -> Result<usize, String> {
    let db = &state.db;

    media_scanner::sync_files(db)
        .await
        .map_err(|e| e.to_string())?;

    let found_files = media_scanner::find_movies(db, root.into())
        .await
        .map_err(|e| e.to_string())?;

    let metadata = metadata_extractor::get_metadata(&found_files);
    let data = omdb_meta_data::get_omdb_of_medias(&metadata, api_key)
        .await
        .map_err(|e| e.to_string())?;

    db.insert_medias(&data).map_err(|e| e.to_string())?;
    Ok(data.len())
}

#[tauri::command]
fn get_countries(state: tauri::State<'_, AppState>) -> Result<Vec<NumericalString>, String> {
    let db = &state.db;
    db.get_countries_from_db().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_genres(state: tauri::State<'_, AppState>) -> Result<Vec<NumericalString>, String> {
    let db = &state.db;

    db.get_genres_from_db().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_actors(state: tauri::State<'_, AppState>) -> Result<Vec<NumericalString>, String> {
    let db = &state.db;

    db.get_actors_from_db().map_err(|e| e.to_string())
}

#[tauri::command]
fn filter_medias(
    filters: FilterValues,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<data_model::Media>, String> {
    let db = &state.db;
    db.filter_medias_on_db(&filters).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_media_by_id(
    media_id: i32,
    state: tauri::State<'_, AppState>,
) -> Result<data_model::Media, String> {
    let db = &state.db;
    db.get_media_by_id_from_db(media_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "movie not found".to_string())
}

#[tauri::command]
async fn update_media_imdb(
    media_id: i32,
    imdb_id: &str,
    api_key: &str,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    let imdb = omdb_meta_data::get_omdb_by_id(imdb_id, api_key)
        .await
        .map_err(|e| e.to_string())?;

    db.insert_imdb_to_db(&imdb).map_err(|e| e.to_string())?;
    db.update_media_imdb_to_db(media_id, imdb_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_watch_list(
    media_id: i32,
    watch_list: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    db.update_watch_list_to_db(media_id, watch_list)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_media_watched(
    media_id: i32,
    watched: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    db.update_media_watched(media_id, watched)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_season_watched(
    season_id: i32,
    watched: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    db.update_season_watched(season_id, watched)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_episode_watched(
    episode_id: i32,
    watched: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    db.update_episode_watched_to_db(episode_id, watched)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_media_my_ranking(
    media_id: i32,
    my_ranking: u8,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    db.update_media_my_ranking_to_db(media_id, my_ranking)
        .map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
fn get_tags(state: tauri::State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let db = &state.db;
    db.get_tags_from_db().map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_tag(tag_id: i32, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let db = &state.db;
    db.remove_tag_from_db(tag_id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_tag(tag: Tag, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let db = &state.db;
    db.update_tag_from_db(&tag).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_medias_by_tag(
    tag_id: i32,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<data_model::Media>, String> {
    let db = &state.db;
    db.get_medias_by_tag_from_db(tag_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn insert_tag(tag: Tag, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let db = &state.db;
    db.insert_tag(&tag).map_err(|e| e.to_string())
}

#[tauri::command]
fn insert_media_tag(
    media_id: i32,
    tag_id: i32,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    db.insert_media_tag(media_id, tag_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_media_tag(
    media_id: i32,
    tag_id: i32,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    println!("{media_id} {tag_id}");
    db.remove_media_tag(media_id, tag_id)
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            get_actors,
            update_media_imdb,
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
            remove_media_tag
        ])
        .setup(|app| {
            let db = DieselDb::from_app_handle(app.app_handle())?;
            app.manage(AppState { db });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
