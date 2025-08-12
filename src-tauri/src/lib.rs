mod media_scanner;
mod metadata_extractor;
mod omdb_meta_data;
mod sqlite;

#[tauri::command]
async fn sync_app_files(root: &str, api_key: &str) -> Result<usize, String> {
    media_scanner::sync_files()
        .await
        .map_err(|e| e.to_string())?;

    let found_files = media_scanner::find_movies(root.into()).await;

    let metadatas = metadata_extractor::get_metadata(found_files);
    let data = omdb_meta_data::get_omdb_metadata(&metadatas, api_key).await;

    sqlite::insert(&data).map_err(|e| e.to_string())?;
    Ok(data.len())
}

#[tauri::command]
fn get_all_video_metadata_app() -> Result<Vec<metadata_extractor::VideoMetaData>, String> {
    sqlite::get_all_video_metadata_from_db().map_err(|e| e.to_string())
}

#[tauri::command]
fn create_table_app() -> Result<(), String> {
    sqlite::create_table().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_countries_app() -> Result<Vec<(usize, String)>, String> {
    sqlite::get_countries_from_db().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_genres_app() -> Result<Vec<(usize, String)>, String> {
    sqlite::get_genres_from_db().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_actors_app() -> Result<Vec<(usize, String)>, String> {
    sqlite::get_actors_from_db().map_err(|e| e.to_string())
}

#[tauri::command]
fn search_videos_app(
    filters: sqlite::FilterValues,
) -> Result<Vec<metadata_extractor::VideoMetaData>, String> {
    sqlite::search_videos_on_db(&filters).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_video_by_id_app(video_id: i64) -> Result<metadata_extractor::VideoMetaData, String> {
    sqlite::get_video_by_id_from_db(video_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "movie not found".to_string())
}

#[tauri::command]
async fn update_video_imdb_app(video_id: i64, imdb_id: &str, api_key: &str) -> Result<(), String> {
    let imdb = omdb_meta_data::get_omdb_by_id(imdb_id, api_key)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(data) = imdb {
        sqlite::insert_imdb_metadata_to_db(&data).map_err(|e| e.to_string())?;
        sqlite::update_video_imdb_to_db(video_id, imdb_id).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("omdb can't find movie".to_string())
    }
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
            get_all_video_metadata_app,
            get_countries_app,
            get_genres_app,
            search_videos_app,
            get_video_by_id_app,
            get_actors_app,
            update_video_imdb_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
