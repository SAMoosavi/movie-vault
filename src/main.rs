
mod media_scanner;
mod metadata_extractor;


#[tokio::main]
async fn main() {
    let root = "/run/media/sam/film/";

    let found_files = media_scanner::find_movies(root.into()).await;

    let meta_datas = metadata_extractor::match_subtitles(found_files);

    for meta_data in &meta_datas {
        println!("{:#?}", meta_data);
    }
}
