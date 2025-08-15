use crate::db::{DB, FilterValues};

mod data_model;
mod db;
mod media_scanner;
mod metadata_extractor;
mod omdb_meta_data;

fn get_db() -> impl DB {
    db::Sqlite::default()
}

#[tauri::command]
async fn sync_app_files(root: &str, api_key: &str) -> Result<usize, String> {
    let db = get_db();

    media_scanner::sync_files(&db)
        .await
        .map_err(|e| e.to_string())?;

    let found_files = media_scanner::find_movies(&db, root.into())
        .await
        .map_err(|e| e.to_string())?;

    let metadatas = metadata_extractor::get_metadata(&found_files);
    let data = omdb_meta_data::get_omdb_of_medias(&metadatas, api_key).await;

    db.insert_medias(&data).map_err(|e| e.to_string())?;
    Ok(data.len())
}

#[tauri::command]
async fn create_table_app() -> Result<(), String> {
    let db = get_db();

    db.create_table().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_countries_app() -> Result<Vec<(usize, String)>, String> {
    let db = get_db();
    db.get_countries_from_db().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_genres_app() -> Result<Vec<(usize, String)>, String> {
    let db = get_db();

    db.get_genres_from_db().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_actors_app() -> Result<Vec<(usize, String)>, String> {
    let db = get_db();

    db.get_actors_from_db().map_err(|e| e.to_string())
}

#[tauri::command]
fn filter_medias_app(filters: FilterValues) -> Result<Vec<data_model::Media>, String> {
    let db = get_db();
    db.filter_medias_on_db(&filters).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_media_by_id_app(media_id: i64) -> Result<data_model::Media, String> {
    let db = get_db();
    db.get_media_by_id_from_db(media_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "movie not found".to_string())
}

#[tauri::command]
async fn update_media_imdb_app(media_id: i64, imdb_id: &str, api_key: &str) -> Result<(), String> {
    let db = get_db();
    let imdb = omdb_meta_data::get_omdb_by_id(imdb_id, api_key)
        .await
        .map_err(|e| e.to_string())?;

    db.insert_imdb_to_db(&imdb).map_err(|e| e.to_string())?;
    db.update_media_imdb_to_db(media_id, imdb_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_watch_list_app(media_id: i64, watch_list: bool) -> Result<(), String> {
    let db = get_db();
    db.update_watch_list_to_db(media_id, watch_list)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_media_watched_app(media_id: i64, watched: bool) -> Result<(), String> {
    let db = get_db();
    db.update_media_watched_to_db(media_id, watched)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_season_watched_app(season_id: i64, watched: bool) -> Result<(), String> {
    let db = get_db();
    db.update_season_watched_to_db(season_id, watched)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_episode_watched_app(episode_id: i64, watched: bool) -> Result<(), String> {
    let db = get_db();
    db.update_episode_watched_to_db(episode_id, watched)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_media_my_ranking_app(media_id: i64, my_ranking: u8) -> Result<(), String> {
    let db = get_db();
    db.update_media_my_ranking_to_db(media_id, my_ranking)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            create_table_app,
            sync_app_files,
            get_countries_app,
            get_genres_app,
            filter_medias_app,
            get_media_by_id_app,
            get_actors_app,
            update_media_imdb_app,
            update_media_watched_app,
            update_season_watched_app,
            update_episode_watched_app,
            update_media_my_ranking_app,
            update_watch_list_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
