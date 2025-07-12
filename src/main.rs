mod media_scanner;
mod metadata_extractor;
mod omdb_meta_data;

#[tokio::main]
async fn main() {
    let root = "/run/media/sam/film/marvel/";

    let found_files = media_scanner::find_movies(root.into()).await;

    let meta_datas = metadata_extractor::match_subtitles(found_files);

    let data = omdb_meta_data::get_omdb_metadata(&meta_datas, "4c602a26").await;

    for meta_data in &data {
        println!("{meta_data:#?}",);
    }
}
