use std::path::PathBuf;

use crate::data_model::{IdType, Imdb, Media, MediaFile, Tag};

use super::{DB, NumericalString, Result};

#[derive(Default, Clone)]
pub struct MokeDB {
    inserted_file: Vec<MediaFile>,
}

impl MokeDB {
    pub fn insert_file(&mut self, path: PathBuf) {
        self.inserted_file.push(MediaFile::from(path));
    }
}

impl DB for MokeDB {
    fn insert_medias(&self, _medias: &[Media]) -> Result<()> {
        todo!()
    }

    fn update_media_my_ranking(&self, _media_id: IdType, _my_ranking: u8) -> Result<usize> {
        todo!()
    }

    fn update_watch_list(&self, _media_id: IdType, _watch_list: bool) -> Result<()> {
        todo!()
    }

    fn update_media_watched(&self, _media_id: IdType, _watched: bool) -> Result<()> {
        todo!()
    }

    fn update_season_watched(&self, _season_id: IdType, _watched: bool) -> Result<()> {
        todo!()
    }

    fn update_episode_watched(&self, _episode_id: IdType, _watched: bool) -> Result<()> {
        todo!()
    }

    fn update_media_imdb(&self, _media_id: IdType, _imdb_id: &str) -> Result<IdType> {
        todo!()
    }

    fn insert_imdb(&self, _imdb: &Imdb) -> Result<()> {
        todo!()
    }

    fn clear_empty_data(&self) -> Result<()> {
        todo!()
    }

    fn get_genres(&self) -> Result<Vec<NumericalString>> {
        todo!()
    }

    fn get_countries(&self) -> Result<Vec<NumericalString>> {
        todo!()
    }

    fn get_actors(&self) -> Result<Vec<(String, String)>> {
        todo!()
    }

    fn remove_file_by_path(&self, _paths: &[PathBuf]) -> Result<()> {
        todo!()
    }

    fn get_all_files(&self) -> Result<Vec<MediaFile>> {
        Ok(self.inserted_file.clone())
    }

    fn filter_medias(&self, _filters: &super::FilterValues, _page: u32) -> Result<Vec<Media>> {
        todo!()
    }

    fn get_media_by_id(&self, _media_id: IdType) -> Result<Option<Media>> {
        todo!()
    }

    fn get_tags(&self) -> Result<Vec<Tag>> {
        todo!()
    }

    fn remove_tag(&self, _tag_id: IdType) -> Result<()> {
        todo!()
    }

    fn update_tag(&self, _tag: &Tag) -> Result<()> {
        todo!()
    }

    fn get_medias_by_tag(&self, _tag_id: IdType) -> Result<Vec<Media>> {
        todo!()
    }

    fn insert_tag(&self, _tag: &Tag) -> Result<()> {
        todo!()
    }

    fn insert_media_tag(&self, _media_id: IdType, _tag_id: IdType) -> Result<()> {
        todo!()
    }

    fn remove_media_tag(&self, _media_id: IdType, _tag_id: IdType) -> Result<()> {
        todo!()
    }
}
