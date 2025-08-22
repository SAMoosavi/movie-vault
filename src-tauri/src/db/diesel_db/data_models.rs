use super::schema::{
    actors, countries, directors, episodes, files, genres, imdb_actors, imdb_countries,
    imdb_directors, imdb_genres, imdb_languages, imdb_writers, imdbs, languages, media_tags,
    medias, seasons, tags, writers,
};
use crate::data_model::{Episode, Imdb, LanguageFormat, Media, Season};
use diesel::{Identifiable, Insertable, Queryable};

#[derive(Debug, Clone, Queryable, Identifiable, serde::Serialize)]
#[diesel(table_name = episodes)]
pub struct DbEpisode {
    pub id: i32,
    pub season_id: i32,
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
    pub id: i32,
    pub media_id: i32,
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
    pub id: i32,
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
    pub year: Option<String>,
    pub rated: Option<String>,
    pub released: Option<String>,
    pub runtime: Option<String>,
    pub plot: Option<String>,
    pub awards: Option<String>,
    pub poster: Option<String>,
    pub imdb_rating: Option<String>,
    pub imdb_votes: Option<String>,
    pub box_office: Option<String>,
    pub total_seasons: Option<String>,
    #[diesel(column_name = "type")]
    pub type_: String,
}

impl From<DbImdb> for Imdb {
    fn from(db: DbImdb) -> Self {
        Imdb {
            imdb_id: db.imdb_id,
            title: db.title,
            year: db.year.unwrap_or_default(),
            rated: db.rated.unwrap_or_default(),
            released: db.released.unwrap_or_default(),
            runtime: db.runtime.unwrap_or_default(),
            plot: db.plot.unwrap_or_default(),
            awards: db.awards.unwrap_or_default(),
            poster: db.poster.unwrap_or_default(),
            imdb_rating: db.imdb_rating.unwrap_or_default(),
            imdb_votes: db.imdb_votes.unwrap_or_default(),
            box_office: db.box_office,
            total_seasons: db.total_seasons,
            r#type: db.type_,
            genres: vec![],
            directors: vec![],
            writers: vec![],
            actors: vec![],
            languages: vec![],
            countries: vec![],
        }
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
    pub media_id: i32,
    pub season_number: i32,
    pub watched: bool,
}

#[derive(Insertable)]
#[diesel(table_name = episodes)]
pub struct NewEpisode {
    pub season_id: i32,
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
    pub year: Option<&'a str>,
    pub rated: Option<&'a str>,
    pub released: Option<&'a str>,
    pub runtime: Option<&'a str>,
    pub plot: Option<&'a str>,
    pub awards: Option<&'a str>,
    pub poster: Option<&'a str>,
    pub imdb_rating: Option<&'a str>,
    pub imdb_votes: Option<&'a str>,
    pub box_office: Option<&'a str>,
    pub total_seasons: Option<&'a str>,
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
    pub genre_id: i32,
}
#[derive(Insertable)]
#[diesel(table_name = directors)]
pub struct NewDirector<'a> {
    pub name: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = imdb_directors)]
pub struct NewImdbDirector<'a> {
    pub imdb_id: &'a str,
    pub director_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = writers)]
pub struct NewWriter<'a> {
    pub name: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = imdb_writers)]
pub struct NewImdbWriter<'a> {
    pub imdb_id: &'a str,
    pub writer_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = actors)]
pub struct NewActor<'a> {
    pub name: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = imdb_actors)]
pub struct NewImdbActor<'a> {
    pub imdb_id: &'a str,
    pub actor_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = languages)]
pub struct NewLanguage<'a> {
    pub name: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = imdb_languages)]
pub struct NewImdbLanguage<'a> {
    pub imdb_id: &'a str,
    pub language_id: i32,
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
    pub country_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = tags)]
pub struct NewTag<'a> {
    pub name: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = media_tags)]
pub struct NewMediaTag {
    pub media_id: i32,
    pub tag_id: i32,
}
