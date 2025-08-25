use std::{fmt, path::PathBuf};

use crate::data_model::{Imdb, Media, MediaFile, Tag};

mod sqlite;
pub use sqlite::Sqlite;

#[cfg(test)]
mod moke;

#[cfg(test)]
pub use moke::MokeDB;

pub type NumericalString = (i32, String);

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    All,
    Movie,
    Series,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let content_type = match self {
            ContentType::All => "all",
            ContentType::Movie => "movie",
            ContentType::Series => "series",
        };
        write!(f, "{content_type}")
    }
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SortByType {
    Name,
    Year,
    Imdb,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SortDirectionType {
    Asc,
    Desc,
}

impl fmt::Display for SortDirectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sort_type = match self {
            SortDirectionType::Asc => "ASC",
            SortDirectionType::Desc => "DESC",
        };
        write!(f, "{sort_type}")
    }
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FilterValues {
    pub name: String,
    pub r#type: ContentType,
    pub min_rating: Option<f64>,
    pub country: Vec<NumericalString>,
    pub genre: Vec<NumericalString>,
    pub actor: Vec<NumericalString>,
    pub exist_imdb: Option<bool>,
    pub exist_multi_file: Option<bool>,
    pub watched: Option<bool>,
    pub sort_by: SortByType,
    pub sort_direction: SortDirectionType,
    pub watch_list: Option<bool>,
    pub tags: Vec<NumericalString>,
}

pub type Result<T> = std::result::Result<T, anyhow::Error>;

pub trait DB {
    fn insert_medias(&self, medias: &[Media]) -> Result<()>;
    fn update_media_my_ranking_to_db(&self, media_id: i32, my_ranking: u8) -> Result<usize>;
    fn update_watch_list_to_db(&self, media_id: i32, watch_list: bool) -> Result<()>;
    fn update_media_watched(&self, media_id: i32, watched: bool) -> Result<()>;
    fn update_season_watched(&self, season_id: i32, watched: bool) -> Result<()>;
    fn update_episode_watched_to_db(&self, episode_id: i32, watched: bool) -> Result<()>;
    fn update_media_imdb_to_db(&self, media_id: i32, imdb_id: &str) -> Result<()>;
    fn insert_imdb_to_db(&self, imdb: &Imdb) -> Result<()>;
    fn clear_empty_data_from_db(&self) -> Result<()>;
    fn get_genres_from_db(&self) -> Result<Vec<NumericalString>>;
    fn get_countries_from_db(&self) -> Result<Vec<NumericalString>>;
    fn get_actors_from_db(&self) -> Result<Vec<NumericalString>>;
    fn remove_file_by_path_from_db(&self, paths: &[PathBuf]) -> Result<()>;
    fn get_all_files_from_db(&self) -> Result<Vec<MediaFile>>;
    fn filter_medias_on_db(&self, filters: &FilterValues) -> Result<Vec<Media>>;
    fn get_media_by_id_from_db(&self, media_id: i32) -> Result<Option<Media>>;
    fn get_tags_from_db(&self) -> Result<Vec<Tag>>;
    fn remove_tag_from_db(&self, tag_id: i32) -> Result<()>;
    fn update_tag_from_db(&self, tag: &Tag) -> Result<()>;
    fn get_medias_by_tag_from_db(&self, tag_id: i32) -> Result<Vec<Media>>;
    fn insert_tag(&self, tag: &Tag) -> Result<()>;
    fn insert_media_tag(&self, media_id: i32, tag_id: i32) -> Result<()>;
    fn remove_media_tag(&self, media_id: i32, tag_id: i32) -> Result<()>;
}
