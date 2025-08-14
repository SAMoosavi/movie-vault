use std::path::{Path, PathBuf};

use crate::{db::DB, metadata_extractor};
use rusqlite::Result;

#[derive(Default, Clone)]
pub struct MokeDB {}

impl DB for MokeDB {
    fn exist_file_by_path_from_db(&self, path: &Path) -> Result<bool> {
        if path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .contains("valid")
        {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    fn insert_medias(&self, _medias: &[metadata_extractor::Media]) -> Result<()> {
        todo!()
    }

    fn update_media_my_ranking_to_db(&self, _media_id: i64, _my_ranking: u8) -> Result<usize> {
        todo!()
    }

    fn update_media_watched_to_db(&self, _media_id: i64, _watched: bool) -> Result<()> {
        todo!()
    }

    fn update_season_watched_to_db(&self, _season_id: i64, _watched: bool) -> Result<()> {
        todo!()
    }

    fn update_episode_watched_to_db(&self, _episode_id: i64, _watched: bool) -> Result<()> {
        todo!()
    }

    fn update_media_imdb_to_db(&self, _media_id: i64, _imdb_id: &str) -> Result<()> {
        todo!()
    }

    fn insert_imdb_to_db(&self, _imdb: &metadata_extractor::Imdb) -> Result<()> {
        todo!()
    }

    fn clear_empty_data_from_db(&self) -> Result<()> {
        todo!()
    }

    fn get_genres_from_db(&self) -> Result<Vec<(usize, String)>> {
        todo!()
    }

    fn get_countries_from_db(&self) -> Result<Vec<(usize, String)>> {
        todo!()
    }

    fn get_actors_from_db(&self) -> Result<Vec<(usize, String)>> {
        todo!()
    }

    fn remove_file_by_path_from_db(&self, _paths: &[PathBuf]) -> Result<()> {
        todo!()
    }

    fn get_all_files_from_db(&self) -> Result<Vec<metadata_extractor::MediaFile>> {
        todo!()
    }

    fn filter_medias_on_db(
        &self,
        _filters: &super::FilterValues,
    ) -> Result<Vec<metadata_extractor::Media>> {
        todo!()
    }

    fn get_media_by_id_from_db(&self, _media_id: i64) -> Result<Option<metadata_extractor::Media>> {
        todo!()
    }

    fn create_table(&self) -> Result<()> {
        todo!()
    }
}
