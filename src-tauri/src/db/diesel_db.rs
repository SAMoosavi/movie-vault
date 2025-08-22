use super::{DB, FilterValues, Result};
use crate::data_model::{Episode, Imdb, Media, MediaFile, Season, Tag};
use data_models::{
    NewActor, NewCountry, NewDirector, NewEpisode, NewFile, NewGenre, NewImdb, NewImdbActor,
    NewImdbCountry, NewImdbDirector, NewImdbGenre, NewImdbLanguage, NewImdbWriter, NewLanguage,
    NewMedia, NewMediaTag, NewSeason, NewTag, NewWriter,
};
use diesel::{
    Connection, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, SqliteConnection,
    connection::SimpleConnection,
    dsl::sql,
    r2d2::{ConnectionManager, Pool},
    sql_types::BigInt,
};
use diesel_migrations::{MigrationHarness, embed_migrations};
use r2d2::PooledConnection;
pub use schema::{
    actors, countries, directors, episodes, files, genres, imdb_actors, imdb_countries,
    imdb_directors, imdb_genres, imdb_languages, imdb_writers, imdbs, languages, media_tags,
    medias, seasons, tags, writers,
};
use std::path::{Path, PathBuf};
use tauri::Manager;

mod data_models;
pub(crate) mod schema;

type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations = embed_migrations!();

#[derive(Clone)]
pub struct DieselDb {
    pool: DbPool,
}

impl DieselDb {
    fn get_conn(&self) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>> {
        self.pool.get().map_err(Into::into)
    }
    fn new_with_path(db_path: PathBuf) -> Result<Self> {
        if let Some(p) = db_path.parent() {
            std::fs::create_dir_all(p)?;
        }
        if !db_path.exists() {
            std::fs::File::create(&db_path)?;
        }

        let url = db_path.to_string_lossy().to_string();
        let manager = ConnectionManager::<SqliteConnection>::new(url);
        let pool = Pool::builder().max_size(8).build(manager)?;

        let mut conn = pool.get()?;
        conn.batch_execute(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;",
        )?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;

        Ok(Self { pool })
    }

    pub fn from_app_handle(app: &tauri::AppHandle) -> Result<Self> {
        let mut db_path = app.path().app_data_dir()?;

        db_path.push("movies.db");
        Self::new_with_path(db_path)
    }
}

// insert
impl DieselDb {
    fn insert_or_get_id_genre(conn: &mut SqliteConnection, name_val: &str) -> QueryResult<i32> {
        diesel::insert_or_ignore_into(genres::table)
            .values(&NewGenre { name: name_val })
            .execute(conn)?;

        genres::table
            .filter(genres::name.eq(name_val))
            .select(genres::id)
            .first(conn)
    }

    fn insert_or_get_id_director(conn: &mut SqliteConnection, name_val: &str) -> QueryResult<i32> {
        diesel::insert_or_ignore_into(directors::table)
            .values(&NewDirector { name: name_val })
            .execute(conn)?;

        directors::table
            .filter(directors::name.eq(name_val))
            .select(directors::id)
            .first(conn)
    }

    fn insert_imdb_genre_by_name(
        conn: &mut SqliteConnection,
        imdb_id_val: &str,
        entity_name: &str,
    ) -> QueryResult<()> {
        let ent_id = Self::insert_or_get_id_genre(conn, entity_name)?;
        diesel::insert_or_ignore_into(imdb_genres::table)
            .values(&NewImdbGenre {
                imdb_id: imdb_id_val,
                genre_id: ent_id,
            })
            .execute(conn)?;
        Ok(())
    }

    fn insert_imdb_director_by_name(
        conn: &mut SqliteConnection,
        imdb_id_val: &str,
        entity_name: &str,
    ) -> QueryResult<()> {
        let ent_id = Self::insert_or_get_id_director(conn, entity_name)?;
        diesel::insert_or_ignore_into(imdb_directors::table)
            .values(&NewImdbDirector {
                imdb_id: imdb_id_val,
                director_id: ent_id,
            })
            .execute(conn)?;
        Ok(())
    }

    fn insert_or_get_id_writer(conn: &mut SqliteConnection, name_val: &str) -> QueryResult<i32> {
        diesel::insert_or_ignore_into(writers::table)
            .values(&NewWriter { name: name_val })
            .execute(conn)?;

        writers::table
            .filter(writers::name.eq(name_val))
            .select(writers::id)
            .first(conn)
    }

    fn insert_imdb_writer_by_name(
        conn: &mut SqliteConnection,
        imdb_id_val: &str,
        entity_name: &str,
    ) -> QueryResult<()> {
        let ent_id = Self::insert_or_get_id_writer(conn, entity_name)?;
        diesel::insert_or_ignore_into(imdb_writers::table)
            .values(&NewImdbWriter {
                imdb_id: imdb_id_val,
                writer_id: ent_id,
            })
            .execute(conn)?;
        Ok(())
    }

    fn insert_or_get_id_actor(conn: &mut SqliteConnection, name_val: &str) -> QueryResult<i32> {
        diesel::insert_or_ignore_into(actors::table)
            .values(&NewActor { name: name_val })
            .execute(conn)?;

        actors::table
            .filter(actors::name.eq(name_val))
            .select(actors::id)
            .first(conn)
    }

    fn insert_imdb_actor_by_name(
        conn: &mut SqliteConnection,
        imdb_id_val: &str,
        entity_name: &str,
    ) -> QueryResult<()> {
        let ent_id = Self::insert_or_get_id_actor(conn, entity_name)?;
        diesel::insert_or_ignore_into(imdb_actors::table)
            .values(&NewImdbActor {
                imdb_id: imdb_id_val,
                actor_id: ent_id,
            })
            .execute(conn)?;
        Ok(())
    }

    fn insert_or_get_id_language(conn: &mut SqliteConnection, name_val: &str) -> QueryResult<i32> {
        diesel::insert_or_ignore_into(languages::table)
            .values(&NewLanguage { name: name_val })
            .execute(conn)?;

        languages::table
            .filter(languages::name.eq(name_val))
            .select(languages::id)
            .first(conn)
    }

    fn insert_imdb_language_by_name(
        conn: &mut SqliteConnection,
        imdb_id_val: &str,
        entity_name: &str,
    ) -> QueryResult<()> {
        let ent_id = Self::insert_or_get_id_language(conn, entity_name)?;
        diesel::insert_or_ignore_into(imdb_languages::table)
            .values(&NewImdbLanguage {
                imdb_id: imdb_id_val,
                language_id: ent_id,
            })
            .execute(conn)?;
        Ok(())
    }

    fn insert_or_get_id_country(conn: &mut SqliteConnection, name_val: &str) -> QueryResult<i32> {
        diesel::insert_or_ignore_into(countries::table)
            .values(&NewCountry { name: name_val })
            .execute(conn)?;

        countries::table
            .filter(countries::name.eq(name_val))
            .select(countries::id)
            .first(conn)
    }

    fn insert_imdb_country_by_name(
        conn: &mut SqliteConnection,
        imdb_id_val: &str,
        entity_name: &str,
    ) -> QueryResult<()> {
        let ent_id = Self::insert_or_get_id_country(conn, entity_name)?;
        diesel::insert_or_ignore_into(imdb_countries::table)
            .values(&NewImdbCountry {
                imdb_id: imdb_id_val,
                country_id: ent_id,
            })
            .execute(conn)?;
        Ok(())
    }

    fn insert_imdb(conn: &mut SqliteConnection, imdb: &Imdb) -> QueryResult<()> {
        let new = NewImdb {
            imdb_id: imdb.imdb_id.as_str(),
            title: imdb.title.as_str(),
            year: Some(imdb.year.as_str()),
            rated: Some(imdb.rated.as_str()),
            released: Some(imdb.released.as_str()),
            runtime: Some(imdb.runtime.as_str()),
            plot: Some(imdb.plot.as_str()),
            awards: Some(imdb.awards.as_str()),
            poster: Some(imdb.poster.as_str()),
            imdb_rating: Some(imdb.imdb_rating.as_str()),
            imdb_votes: Some(imdb.imdb_votes.as_str()),
            box_office: imdb.box_office.as_deref(),
            total_seasons: imdb.total_seasons.as_deref(),
            type_: imdb.r#type.as_str(),
        };
        diesel::insert_into(imdbs::table)
            .values(&new)
            .execute(conn)?;

        for g in &imdb.genres {
            Self::insert_imdb_genre_by_name(conn, &imdb.imdb_id, g)?;
        }
        for d in &imdb.directors {
            Self::insert_imdb_director_by_name(conn, &imdb.imdb_id, d)?;
        }
        for w in &imdb.writers {
            Self::insert_imdb_writer_by_name(conn, &imdb.imdb_id, w)?;
        }
        for a in &imdb.actors {
            Self::insert_imdb_actor_by_name(conn, &imdb.imdb_id, a)?;
        }
        for l in &imdb.languages {
            Self::insert_imdb_language_by_name(conn, &imdb.imdb_id, l)?;
        }
        for c in &imdb.countries {
            Self::insert_imdb_country_by_name(conn, &imdb.imdb_id, c)?;
        }

        Ok(())
    }
    fn insert_media(conn: &mut SqliteConnection, media: &Media) -> QueryResult<()> {
        let imdb_id = media.imdb.as_ref().map(|imdb| imdb.imdb_id.as_str());

        if let Some(imdb) = &media.imdb {
            Self::insert_imdb(conn, imdb)?;
        }

        let new = NewMedia {
            name: media.name.as_str(),
            year: media.year,
            watched: media.watched,
            my_ranking: media.my_ranking as i32,
            watch_list: media.watch_list,
            imdb_id,
        };

        diesel::insert_into(medias::table)
            .values(&new)
            .execute(conn)?;

        let id: i64 = diesel::select(sql::<BigInt>("last_insert_rowid()")).get_result(conn)?;
        let id = id as i32;

        for season in &media.seasons {
            Self::insert_season(conn, id, season)?;
        }

        Self::insert_files(conn, &media.files, Some(id), None)?;

        Ok(())
    }
    fn insert_season(
        conn: &mut SqliteConnection,
        media_id: i32,
        season: &Season,
    ) -> QueryResult<()> {
        let new_episode = NewSeason {
            media_id,
            season_number: season.number,
            watched: season.watched,
        };

        diesel::insert_into(seasons::table)
            .values(&new_episode)
            .execute(conn)?;

        let id: i64 = diesel::select(sql::<BigInt>("last_insert_rowid()")).get_result(conn)?;
        let id = id as i32;

        for episode in &season.episodes {
            Self::insert_episodes(conn, id, episode)?;
        }

        Ok(())
    }
    fn insert_episodes(
        conn: &mut SqliteConnection,
        season_id: i32,
        episode: &Episode,
    ) -> QueryResult<()> {
        let new_episode = NewEpisode {
            season_id,
            episode_number: episode.number,
            watched: episode.watched,
        };
        diesel::insert_into(episodes::table)
            .values(&new_episode)
            .execute(conn)?;

        let id: i64 = diesel::select(sql::<BigInt>("last_insert_rowid()")).get_result(conn)?;
        let id = id as i32;

        Self::insert_files(conn, &episode.files, None, Some(id))?;

        Ok(())
    }
    fn insert_files(
        conn: &mut SqliteConnection,
        files_in: &[MediaFile],
        media_id_val: Option<i32>,
        episode_id_val: Option<i32>,
    ) -> QueryResult<()> {
        let new_files: Vec<NewFile> = files_in
            .iter()
            .map(|f| NewFile {
                media_id: media_id_val,
                episode_id: episode_id_val,
                file_name: &f.file_name,
                path: &f.path,
                quality: f.quality.as_deref(),
                language_format: f.language_format.clone(),
            })
            .collect();

        diesel::insert_into(files::table)
            .values(&new_files)
            .execute(conn)?;

        Ok(())
    }

    pub fn insert_tag(conn: &mut SqliteConnection, tag: &crate::Tag) -> QueryResult<i32> {
        diesel::insert_or_ignore_into(tags::table)
            .values(&NewTag { name: &tag.name })
            .execute(conn)?;

        tags::table
            .filter(tags::name.eq(&tag.name))
            .select(tags::id)
            .first(conn)
    }

    pub fn insert_media_tag(
        conn: &mut SqliteConnection,
        media_id_val: i32,
        tag_id_val: i32,
    ) -> QueryResult<()> {
        diesel::insert_or_ignore_into(media_tags::table)
            .values(&NewMediaTag {
                media_id: media_id_val,
                tag_id: tag_id_val,
            })
            .execute(conn)?;
        Ok(())
    }
}

impl DB for DieselDb {
    fn exist_file_by_path_from_db(&self, path: &Path) -> Result<bool> {
        todo!()
    }

    fn create_table(&self) -> Result<()> {
        todo!()
    }

    fn insert_medias(&self, medias: &[Media]) -> Result<()> {
        self.get_conn()?.transaction(|conn| {
            for media in medias {
                Self::insert_media(conn, media)?;
            }
            Ok(())
        })
    }

    fn update_media_my_ranking_to_db(&self, media_id: i64, my_ranking: u8) -> Result<usize> {
        todo!()
    }

    fn update_watch_list_to_db(&self, media_id: i64, watch_list: bool) -> Result<()> {
        todo!()
    }

    fn update_media_watched_to_db(&self, media_id: i64, watched: bool) -> Result<()> {
        todo!()
    }

    fn update_season_watched_to_db(&self, season_id: i64, watched: bool) -> Result<()> {
        todo!()
    }

    fn update_episode_watched_to_db(&self, episode_id: i64, watched: bool) -> Result<()> {
        todo!()
    }

    fn update_media_imdb_to_db(&self, media_id: i64, imdb_id: &str) -> Result<()> {
        todo!()
    }

    fn insert_imdb_to_db(&self, imdb: &Imdb) -> Result<()> {
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

    fn remove_file_by_path_from_db(&self, paths: &[PathBuf]) -> Result<()> {
        todo!()
    }

    fn get_all_files_from_db(&self) -> Result<Vec<MediaFile>> {
        todo!()
    }

    fn filter_medias_on_db(&self, filters: &FilterValues) -> Result<Vec<Media>> {
        todo!()
    }

    fn get_media_by_id_from_db(&self, media_id: i64) -> Result<Option<Media>> {
        todo!()
    }

    fn get_tags_from_db(&self) -> Result<Vec<Tag>> {
        todo!()
    }

    fn remove_tag_from_db(&self, tag_id: i64) -> Result<usize> {
        todo!()
    }

    fn update_tag_from_db(&self, tag: &Tag) -> Result<()> {
        todo!()
    }

    fn get_medias_by_tag_from_db(&self, tag_id: i64) -> Result<Vec<Media>> {
        todo!()
    }

    fn insert_tag(&self, tag: &Tag) -> Result<()> {
        todo!()
    }

    fn insert_media_tag(&self, media_id: i64, tag_id: i64) -> Result<()> {
        todo!()
    }

    fn remove_media_tag(&self, media_id: i64, tag_id: i64) -> Result<()> {
        todo!()
    }
}
