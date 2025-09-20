use std::{fmt, path::PathBuf};

use crate::data_model::{IdType, Imdb, Media, MediaFile, Tag};

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
            ContentType::Movie => "Movie",
            ContentType::Series => "TVSeries",
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
    pub actor: Vec<(String, String)>,
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
    fn update_media_my_ranking(&self, media_id: IdType, my_ranking: u8) -> Result<usize>;
    fn update_watch_list(&self, media_id: IdType, watch_list: bool) -> Result<()>;
    fn update_media_watched(&self, media_id: IdType, watched: bool) -> Result<()>;
    fn update_season_watched(&self, season_id: IdType, watched: bool) -> Result<()>;
    fn update_episode_watched(&self, episode_id: IdType, watched: bool) -> Result<()>;
    fn update_media_imdb(&self, media_id: IdType, imdb_id: &str) -> Result<IdType>;
    fn insert_imdb(&self, imdb: &Imdb) -> Result<()>;
    #[allow(dead_code)]
    fn clear_empty_data(&self) -> Result<()>;
    fn get_genres(&self) -> Result<Vec<NumericalString>>;
    fn get_countries(&self) -> Result<Vec<NumericalString>>;
    fn get_actors(&self) -> Result<Vec<(String, String)>>;
    fn remove_file_by_path(&self, paths: &[PathBuf]) -> Result<()>;
    fn get_all_files(&self) -> Result<Vec<MediaFile>>;
    fn filter_medias(&self, filters: &FilterValues, page: u32) -> Result<Vec<Media>>;
    fn get_media_by_id(&self, media_id: IdType) -> Result<Option<Media>>;
    fn get_tags(&self) -> Result<Vec<Tag>>;
    fn remove_tag(&self, tag_id: IdType) -> Result<()>;
    fn update_tag(&self, tag: &Tag) -> Result<()>;
    fn get_medias_by_tag(&self, tag_id: IdType) -> Result<Vec<Media>>;
    fn insert_tag(&self, tag: &Tag) -> Result<()>;
    fn insert_media_tag(&self, media_id: IdType, tag_id: IdType) -> Result<()>;
    fn remove_media_tag(&self, media_id: IdType, tag_id: IdType) -> Result<()>;
}
