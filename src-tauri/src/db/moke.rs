use std::path::{Path, PathBuf};

use crate::data_model::{Imdb, Media, MediaFile, Tag};

use super::{DB, NumericalString, Result};

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

    fn insert_medias(&self, _medias: &[Media]) -> Result<()> {
        todo!()
    }

    fn update_media_my_ranking_to_db(&self, _media_id: i32, _my_ranking: u8) -> Result<usize> {
        todo!()
    }

    fn update_media_watched(&self, _media_id: i32, _watched: bool) -> Result<()> {
        todo!()
    }

    fn update_season_watched(&self, _season_id: i32, _watched: bool) -> Result<()> {
        todo!()
    }

    fn update_episode_watched_to_db(&self, _episode_id: i32, _watched: bool) -> Result<()> {
        todo!()
    }

    fn update_media_imdb_to_db(&self, _media_id: i32, _imdb_id: &str) -> Result<()> {
        todo!()
    }

    fn insert_imdb_to_db(&self, _imdb: &Imdb) -> Result<()> {
        todo!()
    }

    fn clear_empty_data_from_db(&self) -> Result<()> {
        todo!()
    }

    fn get_genres_from_db(&self) -> Result<Vec<NumericalString>> {
        todo!()
    }

    fn get_countries_from_db(&self) -> Result<Vec<NumericalString>> {
        todo!()
    }

    fn get_actors_from_db(&self) -> Result<Vec<NumericalString>> {
        todo!()
    }

    fn remove_file_by_path_from_db(&self, _paths: &[PathBuf]) -> Result<()> {
        todo!()
    }

    fn get_all_files_from_db(&self) -> Result<Vec<MediaFile>> {
        todo!()
    }

    fn filter_medias_on_db(&self, _filters: &super::FilterValues) -> Result<Vec<Media>> {
        todo!()
    }

    fn get_media_by_id_from_db(&self, _media_id: i32) -> Result<Option<Media>> {
        todo!()
    }

    fn update_watch_list_to_db(&self, _media_id: i32, _watch_list: bool) -> Result<()> {
        todo!()
    }

    fn get_tags_from_db(&self) -> Result<Vec<Tag>> {
        todo!()
    }

    fn remove_tag_from_db(&self, _tag_id: i32) -> Result<()> {
        todo!()
    }

    fn update_tag_from_db(&self, _tag: &Tag) -> Result<()> {
        todo!()
    }

    fn get_medias_by_tag_from_db(&self, _tag_id: i32) -> Result<Vec<Media>> {
        todo!()
    }

    fn insert_tag(&self, _tag: &Tag) -> Result<()> {
        todo!()
    }

    fn insert_media_tag(&self, _media_id: i32, _tag_id: i32) -> Result<()> {
        todo!()
    }

    fn remove_media_tag(&self, _media_id: i32, _tag_id: i32) -> Result<()> {
        todo!()
    }
}
