use super::schema::{
    countries, episodes, files, genres, imdb_countries, imdb_genres, imdb_people, imdbs,
    media_tags, medias, people, seasons, tags,
};
use crate::data_model::{
    Episode, IdType, Imdb, LanguageFormat, Media, MediaFile, Person, Season, Tag,
};
use diesel::{Identifiable, Insertable, Queryable};

#[derive(Debug, Clone, Queryable, Identifiable, serde::Serialize)]
#[diesel(table_name = episodes)]
pub struct DbEpisode {
    pub id: IdType,
    pub season_id: IdType,
    pub episode_number: i32,
    pub watched: bool,
}

impl From<DbEpisode> for Episode {
    fn from(db: DbEpisode) -> Self {
        Self {
            id: db.id,
            number: db.episode_number,
            watched: db.watched,
            files: vec![],
        }
    }
}

#[derive(Debug, Clone, Queryable, Identifiable, serde::Serialize)]
#[diesel(table_name = seasons)]
pub struct DbSeason {
    pub id: IdType,
    pub media_id: IdType,
    pub season_number: i32,
    pub watched: bool,
}

impl From<DbSeason> for Season {
    fn from(db: DbSeason) -> Self {
        Self {
            id: db.id,
            number: db.season_number,
            watched: db.watched,
            episodes: vec![],
        }
    }
}

#[derive(Debug, Clone, Queryable, Identifiable, serde::Serialize)]
#[diesel(table_name = medias)]
pub struct DbMedia {
    pub id: IdType,
    pub name: String,
    pub year: Option<i32>,
    pub watched: bool,
    pub my_ranking: i32,
    pub watch_list: bool,
    pub imdb_id: Option<String>,
}

impl From<DbMedia> for Media {
    fn from(db: DbMedia) -> Self {
        Media {
            id: db.id,
            name: db.name,
            year: db.year,
            watched: db.watched,
            my_ranking: db.my_ranking as u8,
            watch_list: db.watch_list,
            seasons: vec![],
            files: vec![],
            imdb: None,
            tags: vec![],
        }
    }
}

#[derive(Debug, Clone, Queryable, Identifiable, serde::Serialize)]
#[diesel(table_name = imdbs)]
#[diesel(primary_key(imdb_id))]
pub struct DbImdb {
    pub imdb_id: String,
    pub title: String,
    pub year: i32,
    pub rated: Option<String>,
    pub runtime: Option<String>,
    pub plot: Option<String>,
    pub awards: Option<String>,
    pub poster: Option<String>,
    pub imdb_rating: Option<String>,
    pub imdb_votes: i32,
    pub box_office: Option<String>,
    pub total_seasons: Option<String>,
    #[diesel(column_name = "type")]
    pub type_: String,
}

impl From<DbImdb> for Imdb {
    fn from(db: DbImdb) -> Self {
        Self {
            imdb_id: db.imdb_id,
            title: db.title,
            year: db.year,
            plot: db.plot.unwrap_or_default(),
            poster: db.poster.unwrap_or_default(),
            imdb_rating: db.imdb_rating.unwrap_or_default(),
            imdb_votes: db.imdb_votes,
            r#type: db.type_,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, Queryable)]
#[diesel(table_name = files)]
pub struct DbFile {
    pub id: IdType,
    pub media_id: Option<i32>,
    pub episode_id: Option<i32>,
    pub file_name: String,
    pub path: String,
    pub quality: Option<String>,
    pub language_format: LanguageFormat,
}

impl From<DbFile> for MediaFile {
    fn from(db: DbFile) -> Self {
        Self {
            id: db.id,
            file_name: db.file_name,
            path: db.path,
            quality: db.quality,
            language_format: db.language_format,
        }
    }
}

#[derive(Debug, Clone, Queryable, Identifiable, serde::Serialize)]
#[diesel(table_name = tags)]
pub struct DbTag {
    pub id: IdType,
    pub name: String,
}

impl From<DbTag> for Tag {
    fn from(db: DbTag) -> Self {
        Self {
            id: db.id,
            name: db.name,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, Queryable)]
#[diesel(table_name = files)]
pub struct DbPerson {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

impl From<&DbPerson> for Person {
    fn from(db: &DbPerson) -> Self {
        Self {
            id: db.id.clone(),
            name: db.name.clone(),
            url: db.url.clone().unwrap_or_default(),
        }
    }
}

impl From<DbPerson> for Person {
    fn from(db: DbPerson) -> Self {
        Self::from(&db)
    }
}

#[derive(Insertable)]
#[diesel(table_name = medias)]
pub struct NewMedia<'a> {
    pub name: &'a str,
    pub year: Option<i32>,
    pub watched: bool,
    pub my_ranking: i32,
    pub watch_list: bool,
    pub imdb_id: Option<&'a str>,
}

#[derive(Insertable)]
#[diesel(table_name = seasons)]
pub struct NewSeason {
    pub media_id: IdType,
    pub season_number: i32,
    pub watched: bool,
}

#[derive(Insertable)]
#[diesel(table_name = episodes)]
pub struct NewEpisode {
    pub season_id: IdType,
    pub episode_number: i32,
    pub watched: bool,
}

#[derive(Insertable)]
#[diesel(table_name = files)]
pub struct NewFile<'a> {
    pub media_id: Option<i32>,
    pub episode_id: Option<i32>,
    pub file_name: &'a str,
    pub path: &'a str,
    pub quality: Option<&'a str>,
    pub language_format: LanguageFormat,
}

#[derive(Insertable)]
#[diesel(table_name = imdbs)]
pub struct NewImdb<'a> {
    pub imdb_id: &'a str,
    pub title: &'a str,
    pub year: i32,
    pub plot: Option<&'a str>,
    pub poster: Option<&'a str>,
    pub imdb_rating: Option<&'a str>,
    pub imdb_votes: i32,
    // #[diesel(column_name = "type")]
    pub type_: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = genres)]
pub struct NewGenre<'a> {
    pub name: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = imdb_genres)]
pub struct NewImdbGenre<'a> {
    pub imdb_id: &'a str,
    pub genre_id: IdType,
}

#[derive(Insertable)]
#[diesel(table_name = people)]
pub struct NewPerson<'a> {
    pub id: &'a str,
    pub url: &'a str,
    pub name: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = imdb_people)]
pub struct NewImdbPerson<'a> {
    pub imdb_id: &'a str,
    pub person_id: &'a str,
    pub person_type: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = countries)]
pub struct NewCountry<'a> {
    pub name: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = imdb_countries)]
pub struct NewImdbCountry<'a> {
    pub imdb_id: &'a str,
    pub country_id: IdType,
}

#[derive(Insertable)]
#[diesel(table_name = tags)]
pub struct NewTag<'a> {
    pub name: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = media_tags)]
pub struct NewMediaTag {
    pub media_id: IdType,
    pub tag_id: IdType,
}
