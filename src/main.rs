use dotenv::dotenv;
use std::env;

mod media_scanner;
mod metadata_extractor;
mod omdb_meta_data;
mod sqlite;

#[tokio::main]
async fn main() {
    let root = "/run/media/sam/film/marvel/";
    sqlite::create_table().unwrap();

    media_scanner::sync_files().await;

    let found_files = media_scanner::find_movies(root.into()).await;

    let meta_datas = metadata_extractor::match_subtitles(found_files);

    dotenv().ok();
    let api_key = env::var("OMDB_API_KEY").expect("OMDB_API_KEY not set in .env");

    let data = omdb_meta_data::get_omdb_metadata(&meta_datas, &api_key).await;

    sqlite::insert(&data).unwrap();
}
