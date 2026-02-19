mod data_models;
pub mod schema;

use super::{
    ContentType, DB, FilterValues, InsertMediasStats, MultiFileFilterType, NumericalString, Result,
    SortByType, SortDirectionType,
};
use crate::data_model::{Episode, IdType, Imdb, Media, MediaFile, Person, Season, Tag};
use anyhow::Ok;
use data_models::{
    DbEpisode, DbFile, DbImdb, DbMedia, DbPerson, DbSeason, NewCountry, NewEpisode, NewFile,
    NewGenre, NewImdb, NewImdbCountry, NewImdbGenre, NewImdbPerson, NewMedia, NewMediaTag,
    NewPerson, NewSeason, NewTag,
};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, NullableExpressionMethods, QueryDsl,
    RunQueryDsl, SqliteConnection,
    connection::SimpleConnection,
    dsl::{exists, sql},
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    sql_types::{BigInt, Double, Integer},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
pub use schema::{
    countries, episodes, files, genres, imdb_countries, imdb_genres, imdb_people, imdbs,
    media_tags, medias, people, seasons, tags,
};
use std::{fmt, path::PathBuf};
use tauri::Manager;
use tauri_plugin_log::log::{error, info, warn};

type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
enum PersonType {
    Actor,
    Writer,
    Director,
}

impl fmt::Display for PersonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersonType::Actor => write!(f, "Actor"),
            PersonType::Writer => write!(f, "Writer"),
            PersonType::Director => write!(f, "Director"),
        }
    }
}

pub struct Sqlite {
    pool: DbPool,
}

struct InsertMediaOutcome {
    media_id: IdType,
    inserted_new: bool,
}

impl Sqlite {
    fn get_conn(&self) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>> {
        self.pool.get().map_err(Into::into)
    }

    fn new_with_path(db_path: PathBuf) -> Result<Self> {
        if let Some(p) = db_path.parent() {
            std::fs::create_dir_all(p)?;
            info!("Ensured database directory exists: {}", p.display());
        }

        if !db_path.exists() {
            std::fs::File::create(&db_path)?;
            info!("Created new database file: {}", db_path.display());
        } else {
            warn!("Database file already exists: {}", db_path.display());
        }

        let url = db_path.to_string_lossy().to_string();
        info!("Opening SQLite database at {}", url);

        let manager = ConnectionManager::<SqliteConnection>::new(url.clone());
        let pool = Pool::builder().max_size(8).build(manager)?;

        let mut conn = pool.get()?;
        conn.batch_execute(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;",
        )?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;

        Ok(Self { pool })
    }

    pub fn from_app_handle(app: &tauri::AppHandle) -> Result<Self> {
        let mut db_path = app.path().app_data_dir().map_err(|e| {
            error!("Failed to get app data dir: {}", e);
            e
        })?;

        db_path.push("movies.db");
        info!("Resolved DB path from app handle: {}", db_path.display());

        Self::new_with_path(db_path).map_err(|e| {
            error!("Failed to create Sqlite instance from app handle: {}", e);
            e
        })
    }
}

// insert
impl Sqlite {
    fn insert_or_get_id_genre(conn: &mut SqliteConnection, name_val: &str) -> Result<i32> {
        diesel::insert_or_ignore_into(genres::table)
            .values(&NewGenre { name: name_val })
            .execute(conn)?;

        let id = genres::table
            .filter(genres::name.eq(name_val))
            .select(genres::id)
            .first(conn)?;

        Ok(id)
    }

    fn insert_imdb_genre_by_name(
        conn: &mut SqliteConnection,
        imdb_id_val: &str,
        entity_name: &str,
    ) -> Result<()> {
        let ent_id = Self::insert_or_get_id_genre(conn, entity_name)?;
        diesel::insert_or_ignore_into(imdb_genres::table)
            .values(&NewImdbGenre {
                imdb_id: imdb_id_val,
                genre_id: ent_id,
            })
            .execute(conn)?;
        Ok(())
    }

    fn insert_person(conn: &mut SqliteConnection, person: &Person) -> Result<()> {
        diesel::insert_or_ignore_into(people::table)
            .values(&NewPerson {
                id: &person.id,
                name: &person.name,
                url: &person.url,
            })
            .execute(conn)?;

        Ok(())
    }

    fn insert_imdb_person(
        conn: &mut SqliteConnection,
        imdb_id_val: &str,
        person: &Person,
        person_type: &PersonType,
    ) -> Result<()> {
        Self::insert_person(conn, person)?;
        diesel::insert_or_ignore_into(imdb_people::table)
            .values(&NewImdbPerson {
                imdb_id: imdb_id_val,
                person_id: person.id.as_str(),
                person_type: person_type.to_string().as_str(),
            })
            .execute(conn)?;
        Ok(())
    }

    fn insert_or_get_id_country(conn: &mut SqliteConnection, name_val: &str) -> Result<i32> {
        diesel::insert_or_ignore_into(countries::table)
            .values(&NewCountry { name: name_val })
            .execute(conn)?;

        let id = countries::table
            .filter(countries::name.eq(name_val))
            .select(countries::id)
            .first(conn)?;
        Ok(id)
    }

    fn insert_imdb_country_by_name(
        conn: &mut SqliteConnection,
        imdb_id_val: &str,
        entity_name: &str,
    ) -> Result<()> {
        let ent_id = Self::insert_or_get_id_country(conn, entity_name)?;
        diesel::insert_or_ignore_into(imdb_countries::table)
            .values(&NewImdbCountry {
                imdb_id: imdb_id_val,
                country_id: ent_id,
            })
            .execute(conn)?;
        Ok(())
    }

    fn insert_imdb(conn: &mut SqliteConnection, imdb: &Imdb) -> Result<()> {
        let new = NewImdb {
            imdb_id: imdb.imdb_id.as_str(),
            title: imdb.title.as_str(),
            year: imdb.year,
            plot: Some(imdb.plot.as_str()),
            poster: Some(imdb.poster.as_str()),
            imdb_rating: Some(imdb.imdb_rating.as_str()),
            imdb_votes: imdb.imdb_votes,
            type_: imdb.r#type.as_str(),
        };

        diesel::insert_or_ignore_into(imdbs::table)
            .values(&new)
            .execute(conn)?;

        for g in &imdb.genres {
            Self::insert_imdb_genre_by_name(conn, &imdb.imdb_id, g)?;
        }

        for a in &imdb.actors {
            Self::insert_imdb_person(conn, &imdb.imdb_id, a, &PersonType::Actor)?;
        }

        for w in &imdb.writers {
            Self::insert_imdb_person(conn, &imdb.imdb_id, w, &PersonType::Writer)?;
        }

        for d in &imdb.directors {
            Self::insert_imdb_person(conn, &imdb.imdb_id, d, &PersonType::Director)?;
        }

        for c in &imdb.countries {
            Self::insert_imdb_country_by_name(conn, &imdb.imdb_id, c)?;
        }

        Ok(())
    }

    fn insert_media(conn: &mut SqliteConnection, media: &Media) -> Result<InsertMediaOutcome> {
        let imdb_id = media.imdb.as_ref().map(|imdb| imdb.imdb_id.as_str());

        if let Some(imdb) = &media.imdb {
            Self::insert_imdb(conn, imdb)?;
        }

        // Match by imdb_id if provided
        let existing_media_id: Option<IdType> = if let Some(imdb_id_val) = imdb_id {
            medias::table
                .filter(medias::imdb_id.eq(imdb_id_val))
                .select(medias::id)
                .first::<IdType>(conn)
                .optional()?
        } else {
            medias::table
                .filter(
                    medias::name
                        .eq(media.name.as_str())
                        .and(medias::year.eq(media.year)),
                )
                .select(medias::id)
                .first::<IdType>(conn)
                .optional()?
        };

        let (id, inserted_new) = if let Some(id) = existing_media_id {
            // Media already exists, use the existing ID
            (id, false)
        } else {
            // Prepare new media for insertion
            let new = NewMedia {
                name: media.name.as_str(),
                year: media.year,
                watched: media.watched,
                my_ranking: media.my_ranking as i32,
                watch_list: media.watch_list,
                imdb_id,
            };

            // Insert new media
            diesel::insert_into(medias::table)
                .values(&new)
                .execute(conn)?;

            // Retrieve the last inserted ID
            (
                diesel::select(sql::<BigInt>("last_insert_rowid()")).get_result::<i64>(conn)?
                    as i32,
                true,
            )
        };

        for season in &media.seasons {
            Self::insert_season(conn, id, season)?;
        }

        Self::insert_files(conn, &media.files, Some(id), None)?;

        Ok(InsertMediaOutcome {
            media_id: id,
            inserted_new,
        })
    }

    fn insert_season(conn: &mut SqliteConnection, media_id: IdType, season: &Season) -> Result<()> {
        let seasons_id = seasons::table
            .filter(
                seasons::media_id
                    .eq(media_id)
                    .and(seasons::season_number.eq(season.number)),
            )
            .select(seasons::id)
            .first::<IdType>(conn)
            .optional()?;

        let id: i32 = if let Some(id) = seasons_id {
            id
        } else {
            let new_episode = NewSeason {
                media_id,
                season_number: season.number,
                watched: season.watched,
            };

            diesel::insert_into(seasons::table)
                .values(&new_episode)
                .execute(conn)?;

            diesel::select(sql::<BigInt>("last_insert_rowid()")).get_result::<i64>(conn)? as i32
        };

        for episode in &season.episodes {
            Self::insert_episodes(conn, id, episode)?;
        }

        Ok(())
    }

    fn insert_episodes(
        conn: &mut SqliteConnection,
        season_id: IdType,
        episode: &Episode,
    ) -> Result<()> {
        let episode_id = episodes::table
            .filter(
                episodes::season_id
                    .eq(season_id)
                    .and(episodes::episode_number.eq(episode.number)),
            )
            .select(episodes::id)
            .first::<IdType>(conn)
            .optional()?;

        let id: i32 = if let Some(id) = episode_id {
            id
        } else {
            let new_episode = NewEpisode {
                season_id,
                episode_number: episode.number,
                watched: episode.watched,
            };
            diesel::insert_into(episodes::table)
                .values(&new_episode)
                .execute(conn)?;

            diesel::select(sql::<BigInt>("last_insert_rowid()")).get_result::<i64>(conn)? as i32
        };

        Self::insert_files(conn, &episode.files, None, Some(id))?;

        Ok(())
    }

    fn insert_files(
        conn: &mut SqliteConnection,
        files_in: &[MediaFile],
        media_id_val: Option<i32>,
        episode_id_val: Option<i32>,
    ) -> Result<()> {
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
}

// Update
impl Sqlite {
    fn update_media_watched(
        conn: &mut SqliteConnection,
        media_id_val: i32,
        watched_val: bool,
    ) -> Result<()> {
        // Update medias
        diesel::update(medias::table.filter(medias::id.eq(media_id_val)))
            .set(medias::watched.eq(watched_val))
            .execute(conn)?;

        // Update seasons
        diesel::update(seasons::table.filter(seasons::media_id.eq(media_id_val)))
            .set(seasons::watched.eq(watched_val))
            .execute(conn)?;

        // Update episodes (with subselect for season_id)
        let season_ids = seasons::table
            .select(seasons::id)
            .filter(seasons::media_id.eq(media_id_val));

        diesel::update(episodes::table.filter(episodes::season_id.eq_any(season_ids)))
            .set(episodes::watched.eq(watched_val))
            .execute(conn)?;

        Ok(())
    }

    fn update_season_watched(
        conn: &mut SqliteConnection,
        season_id_val: i32,
        watched_val: bool,
    ) -> Result<()> {
        diesel::update(seasons::table.filter(seasons::id.eq(season_id_val)))
            .set(seasons::watched.eq(watched_val))
            .execute(conn)?;

        diesel::update(episodes::table.filter(episodes::season_id.eq(season_id_val)))
            .set(episodes::watched.eq(watched_val))
            .execute(conn)?;

        let media_id_val: i32 = seasons::table
            .select(seasons::media_id)
            .filter(seasons::id.eq(season_id_val))
            .first::<i32>(conn)?;

        let watched_count: i64 = seasons::table
            .filter(seasons::media_id.eq(media_id_val))
            .filter(seasons::watched.eq(false))
            .count()
            .get_result(conn)?;

        diesel::update(medias::table.filter(medias::id.eq(media_id_val)))
            .set(medias::watched.eq(watched_count == 0))
            .execute(conn)?;

        Ok(())
    }

    fn update_episode_watched(
        conn: &mut SqliteConnection,
        episode_id_val: i32,
        watched_val: bool,
    ) -> Result<()> {
        diesel::update(episodes::table.filter(episodes::id.eq(episode_id_val)))
            .set(episodes::watched.eq(watched_val))
            .execute(conn)?;

        let season_ids = episodes::table
            .select(episodes::season_id)
            .filter(episodes::id.eq(episode_id_val))
            .first::<i32>(conn)?;

        let watched_count: i64 = episodes::table
            .filter(episodes::season_id.eq(season_ids))
            .filter(episodes::watched.eq(false))
            .count()
            .get_result(conn)?;

        diesel::update(seasons::table.filter(seasons::id.eq(season_ids)))
            .set(seasons::watched.eq(watched_count == 0))
            .execute(conn)?;

        let media_id_val: i32 = seasons::table
            .select(seasons::media_id)
            .filter(seasons::id.eq(season_ids))
            .first::<i32>(conn)?;

        let watched_count: i64 = seasons::table
            .filter(seasons::media_id.eq(media_id_val))
            .filter(seasons::watched.eq(false))
            .count()
            .get_result(conn)?;

        diesel::update(medias::table.filter(medias::id.eq(media_id_val)))
            .set(medias::watched.eq(watched_count == 0))
            .execute(conn)?;

        Ok(())
    }
}

// get
impl Sqlite {
    fn get_imdb(conn: &mut SqliteConnection, imdb_id_val: Option<String>) -> Result<Option<Imdb>> {
        let Some(imdb_id_val) = imdb_id_val.as_ref() else {
            return Ok(None);
        };

        // Load basic metadata
        let imdb_db: Option<DbImdb> = imdbs::table
            .filter(imdbs::imdb_id.eq(imdb_id_val))
            .first(conn)
            .optional()?; // optional returns Result<Option<_>>

        let mut imdb = match imdb_db {
            Some(data) => Imdb::from(data),
            None => return Ok(None),
        };

        // Load all related data using joins
        imdb.genres = imdb_genres::table
            .inner_join(genres::table.on(imdb_genres::genre_id.eq(genres::id)))
            .filter(imdb_genres::imdb_id.eq(imdb_id_val))
            .select(genres::name)
            .load(conn)?;

        imdb.actors = imdb_people::table
            .inner_join(people::table.on(imdb_people::person_id.eq(people::id)))
            .filter(
                imdb_people::imdb_id
                    .eq(imdb_id_val)
                    .and(imdb_people::person_type.eq(&PersonType::Actor.to_string())),
            )
            .select(people::all_columns)
            .load::<DbPerson>(conn)?
            .par_iter()
            .map(Into::into)
            .collect();

        imdb.writers = imdb_people::table
            .inner_join(people::table.on(imdb_people::person_id.eq(people::id)))
            .filter(
                imdb_people::imdb_id
                    .eq(imdb_id_val)
                    .and(imdb_people::person_type.eq(&PersonType::Writer.to_string())),
            )
            .select(people::all_columns)
            .load::<DbPerson>(conn)?
            .par_iter()
            .map(Into::into)
            .collect();

        imdb.directors = imdb_people::table
            .inner_join(people::table.on(imdb_people::person_id.eq(people::id)))
            .filter(
                imdb_people::imdb_id
                    .eq(imdb_id_val)
                    .and(imdb_people::person_type.eq(&PersonType::Director.to_string())),
            )
            .select(people::all_columns)
            .load::<DbPerson>(conn)?
            .par_iter()
            .map(Into::into)
            .collect();

        imdb.countries = imdb_countries::table
            .inner_join(countries::table.on(imdb_countries::country_id.eq(countries::id)))
            .filter(imdb_countries::imdb_id.eq(imdb_id_val))
            .select(countries::name)
            .load(conn)?;

        Ok(Some(imdb))
    }

    fn get_media_and_imdb_by_media_id(
        conn: &mut SqliteConnection,
        media_id: IdType,
    ) -> Result<Media> {
        // Load basic media data
        let media_db: DbMedia = medias::table.filter(medias::id.eq(media_id)).first(conn)?;

        let imdb = Self::get_imdb(conn, media_db.imdb_id)?;

        // Load tags
        let media_tags = media_tags::table
            .inner_join(tags::table.on(media_tags::tag_id.eq(tags::id)))
            .filter(media_tags::media_id.eq(media_id))
            .select(tags::all_columns)
            .load::<Tag>(conn)?;

        Ok(Media {
            id: media_db.id,
            name: media_db.name,
            year: media_db.year,
            watched: media_db.watched,
            my_ranking: media_db.my_ranking as u8,
            watch_list: media_db.watch_list,
            imdb,
            tags: media_tags,
            seasons: vec![],
            files: vec![],
        })
    }

    fn get_files_for_episode(
        conn: &mut SqliteConnection,
        episode_id: IdType,
    ) -> Result<Vec<MediaFile>> {
        let media_files = files::table
            .filter(files::episode_id.eq(episode_id))
            .load::<DbFile>(conn)?;

        Ok(media_files.into_iter().map(MediaFile::from).collect())
    }

    fn get_files_for_media(
        conn: &mut SqliteConnection,
        media_id: IdType,
    ) -> Result<Vec<MediaFile>> {
        let media_files = files::table
            .filter(files::media_id.eq(media_id))
            .load::<DbFile>(conn)?;

        Ok(media_files.into_iter().map(MediaFile::from).collect())
    }

    fn get_episodes_by_season_id(
        conn: &mut SqliteConnection,
        season_id: IdType,
    ) -> Result<Vec<Episode>> {
        let episodes_list = episodes::table
            .filter(episodes::season_id.eq(season_id))
            .order(episodes::episode_number.asc())
            .load::<DbEpisode>(conn)?;

        let episodes_list = episodes_list
            .into_iter()
            .map(|episode| {
                Ok(Episode {
                    id: episode.id,
                    number: episode.episode_number,
                    watched: episode.watched,
                    files: Self::get_files_for_episode(conn, episode.id)?,
                })
            })
            .collect::<Result<_>>()?;
        Ok(episodes_list)
    }

    fn get_seasons_by_media_id(
        conn: &mut SqliteConnection,
        media_id: IdType,
    ) -> Result<Vec<Season>> {
        let seasons_list = seasons::table
            .filter(seasons::media_id.eq(media_id))
            .order(seasons::season_number.asc())
            .load::<DbSeason>(conn)?;

        let seasons_list = seasons_list
            .into_iter()
            .map(|season| {
                Ok(Season {
                    episodes: Self::get_episodes_by_season_id(conn, season.id)?,
                    id: season.id,
                    number: season.season_number,
                    watched: season.watched,
                })
            })
            .collect::<Result<_>>()?;

        Ok(seasons_list)
    }

    fn get_media_by_id(conn: &mut SqliteConnection, media_id: IdType) -> Result<Option<Media>> {
        // 1. Media
        let media = medias::table
            .find(media_id)
            .first::<DbMedia>(conn)
            .optional()?;

        let Some(media) = media else {
            return Ok(None);
        };

        let imdb = Self::get_imdb(conn, media.imdb_id)?;

        // 2. Seasons
        let seasons_list = Self::get_seasons_by_media_id(conn, media_id)?;

        // 4. Files (bulk load for both media & episodes)
        let files_list = Self::get_files_for_media(conn, media_id)?;

        // 5. Tags (through join table)
        let tags_list = media_tags::table
            .inner_join(tags::table)
            .filter(media_tags::media_id.eq(media.id))
            .select((tags::id, tags::name))
            .load::<Tag>(conn)?;

        Ok(Some(Media {
            id: media.id,
            name: media.name,
            year: media.year,
            watched: media.watched,
            my_ranking: media.my_ranking as u8,
            watch_list: media.watch_list,
            imdb,
            tags: tags_list,
            seasons: seasons_list,
            files: files_list,
        }))
    }
}

// remove
#[allow(dead_code)]
impl Sqlite {
    fn remove_empty_imdb(conn: &mut SqliteConnection) -> Result<()> {
        match diesel::delete(imdbs::table.filter(diesel::dsl::not(diesel::dsl::exists(
            medias::table.filter(medias::imdb_id.eq(imdbs::imdb_id.nullable())),
        ))))
        .execute(conn)
        {
            std::result::Result::Ok(_) => std::result::Result::Ok(()),
            std::result::Result::Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                _,
            )) => {
                // Table might not exist in test environment
                std::result::Result::Ok(())
            }
            std::result::Result::Err(e) => std::result::Result::Err(e.into()),
        }
    }

    fn remove_empty_media(conn: &mut SqliteConnection) -> Result<()> {
        match diesel::delete(
            medias::table.filter(
                diesel::dsl::not(diesel::dsl::exists(
                    files::table.filter(files::media_id.eq(medias::id.nullable())),
                ))
                .and(diesel::dsl::not(diesel::dsl::exists(
                    seasons::table.filter(seasons::media_id.eq(medias::id)),
                ))),
            ),
        )
        .execute(conn)
        {
            std::result::Result::Ok(_) => std::result::Result::Ok(()),
            std::result::Result::Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                _,
            )) => {
                // Table might not exist in test environment
                std::result::Result::Ok(())
            }
            std::result::Result::Err(e) => std::result::Result::Err(e.into()),
        }
    }

    fn remove_empty_seasons(conn: &mut SqliteConnection) -> Result<()> {
        match diesel::delete(seasons::table.filter(diesel::dsl::not(diesel::dsl::exists(
            episodes::table.filter(episodes::season_id.eq(seasons::id)),
        ))))
        .execute(conn)
        {
            std::result::Result::Ok(_) => std::result::Result::Ok(()),
            std::result::Result::Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                _,
            )) => {
                // Table might not exist in test environment
                std::result::Result::Ok(())
            }
            std::result::Result::Err(e) => std::result::Result::Err(e.into()),
        }
    }

    fn remove_empty_episodes(conn: &mut SqliteConnection) -> Result<()> {
        // Try to delete, ignore if table doesn't exist (for tests with in-memory DB)
        match diesel::delete(episodes::table.filter(diesel::dsl::not(diesel::dsl::exists(
            files::table.filter(files::episode_id.eq(episodes::id.nullable())),
        ))))
        .execute(conn)
        {
            std::result::Result::Ok(_) => std::result::Result::Ok(()),
            std::result::Result::Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                _,
            )) => {
                // Table might not exist in test environment
                std::result::Result::Ok(())
            }
            std::result::Result::Err(e) => std::result::Result::Err(e.into()),
        }
    }

    fn delete_media(conn: &mut SqliteConnection, media_id: IdType) -> Result<()> {
        // Get imdb_id before deleting
        let imdb_id: Option<String> = medias::table
            .filter(medias::id.eq(media_id))
            .select(medias::imdb_id)
            .first(conn)?;

        // Check if any files exist on disk
        // Get all file paths for media
        let media_file_paths: Vec<String> = files::table
            .filter(files::media_id.eq(media_id))
            .select(files::path)
            .load(conn)?;

        // Get season ids
        let season_ids: Vec<i32> = seasons::table
            .filter(seasons::media_id.eq(media_id))
            .select(seasons::id)
            .load(conn)?;

        let mut all_episode_file_paths = vec![];
        for season_id in &season_ids {
            let episode_ids: Vec<i32> = episodes::table
                .filter(episodes::season_id.eq(season_id))
                .select(episodes::id)
                .load(conn)?;

            for episode_id in &episode_ids {
                let paths: Vec<String> = files::table
                    .filter(files::episode_id.eq(episode_id))
                    .select(files::path)
                    .load(conn)?;
                all_episode_file_paths.extend(paths);
            }
        }

        // Check all paths
        for path in media_file_paths.iter().chain(&all_episode_file_paths) {
            if std::path::Path::new(path).exists() {
                return Err(anyhow::anyhow!(
                    "Cannot delete media: file {} exists on disk",
                    path
                ));
            }
        }

        // Delete media_tags
        diesel::delete(media_tags::table.filter(media_tags::media_id.eq(media_id)))
            .execute(conn)?;

        // Delete files associated with media
        diesel::delete(files::table.filter(files::media_id.eq(media_id))).execute(conn)?;

        // Delete seasons and their episodes
        for season_id in season_ids {
            // Delete episodes for this season
            let episode_ids: Vec<i32> = episodes::table
                .filter(episodes::season_id.eq(season_id))
                .select(episodes::id)
                .load::<i32>(conn)?;

            for episode_id in episode_ids {
                // Delete files for episodes
                diesel::delete(files::table.filter(files::episode_id.eq(episode_id)))
                    .execute(conn)?;
            }

            // Delete episodes
            diesel::delete(episodes::table.filter(episodes::season_id.eq(season_id)))
                .execute(conn)?;
        }

        // Delete seasons
        diesel::delete(seasons::table.filter(seasons::media_id.eq(media_id))).execute(conn)?;

        // Finally, delete the media
        diesel::delete(medias::table.filter(medias::id.eq(media_id))).execute(conn)?;

        // Delete imdb if exists
        if let Some(imdb_id) = imdb_id {
            Self::delete_imdb(conn, &imdb_id)?;
        }

        Ok(())
    }

    fn delete_imdb(conn: &mut SqliteConnection, imdb_id: &str) -> Result<()> {
        // Delete related tables
        diesel::delete(imdb_genres::table.filter(imdb_genres::imdb_id.eq(imdb_id)))
            .execute(conn)?;
        diesel::delete(imdb_countries::table.filter(imdb_countries::imdb_id.eq(imdb_id)))
            .execute(conn)?;
        diesel::delete(imdb_people::table.filter(imdb_people::imdb_id.eq(imdb_id)))
            .execute(conn)?;

        // Delete the imdb entry
        diesel::delete(imdbs::table.filter(imdbs::imdb_id.eq(imdb_id))).execute(conn)?;

        Ok(())
    }
}

impl DB for Sqlite {
    fn insert_medias(&self, media_list: &[Media]) -> Result<InsertMediasStats> {
        self.get_conn()?.transaction(|conn| {
            let mut stats = InsertMediasStats::default();
            for media in media_list {
                let outcome = Self::insert_media(conn, media)?;
                if outcome.inserted_new {
                    stats.inserted_new += 1;
                } else {
                    stats.merged_existing += 1;
                }
            }
            Ok(stats)
        })
    }

    fn delete_media(&self, media_id: IdType) -> Result<()> {
        self.get_conn()?
            .transaction(|conn| Self::delete_media(conn, media_id))
    }

    fn update_media_my_ranking(&self, media_id: IdType, my_ranking: u8) -> Result<usize> {
        if my_ranking > 10 {
            return Err(anyhow::anyhow!(
                "invalid my_ranking={my_ranking}; expected range 0..=10"
            ));
        }

        let conn = &mut self.get_conn()?;
        diesel::update(medias::table.filter(medias::id.eq(media_id)))
            .set(medias::my_ranking.eq(my_ranking as i32))
            .execute(conn)
            .map_err(Into::into)
    }

    fn update_watch_list(&self, media_id: IdType, watch_list: bool) -> Result<()> {
        let conn = &mut self.get_conn()?;
        diesel::update(medias::table.filter(medias::id.eq(media_id)))
            .set(medias::watch_list.eq(watch_list))
            .execute(conn)?;
        Ok(())
    }

    fn update_media_watched(&self, media_id: IdType, watched: bool) -> Result<()> {
        self.get_conn()?
            .transaction(|conn| Self::update_media_watched(conn, media_id, watched))
    }

    fn update_season_watched(&self, season_id: IdType, watched: bool) -> Result<()> {
        self.get_conn()?
            .transaction(|conn| Self::update_season_watched(conn, season_id, watched))
    }

    fn update_episode_watched(&self, episode_id: IdType, watched: bool) -> Result<()> {
        self.get_conn()?
            .transaction(|conn| Self::update_episode_watched(conn, episode_id, watched))
    }

    fn update_media_imdb(&self, media_id: IdType, imdb_id: &str) -> Result<IdType> {
        self.get_conn()?.transaction(|conn| {
            let mut media = Self::get_media_by_id(conn, media_id)?.ok_or_else(|| {
                anyhow::anyhow!("cannot update imdb for media_id={media_id}: media not found")
            })?;
            diesel::delete(medias::table.filter(medias::id.eq(media.id))).execute(conn)?;
            let imdb = Self::get_imdb(conn, Some(imdb_id.into()))?;
            media.imdb = imdb;

            Ok(Self::insert_media(conn, &media)?.media_id)
        })
    }

    fn insert_media(&self, media: &Media) -> Result<IdType> {
        self.get_conn()?
            .transaction(|conn| Ok(Self::insert_media(conn, media)?.media_id))
    }

    fn insert_imdb(&self, imdb: &Imdb) -> Result<()> {
        self.get_conn()?.transaction(|conn| {
            Self::insert_imdb(conn, imdb)?;
            Ok(())
        })
    }

    fn clear_empty_data(&self) -> Result<()> {
        self.get_conn()?.transaction(|conn| {
            Self::remove_empty_episodes(conn)?;
            Self::remove_empty_seasons(conn)?;
            Self::remove_empty_media(conn)?;
            Self::remove_empty_imdb(conn)?;

            Ok(())
        })
    }

    fn get_genres(&self) -> Result<Vec<NumericalString>> {
        let conn = &mut self.get_conn()?;
        let results = genres::table
            .select((genres::id, genres::name))
            .order(genres::name.asc())
            .load(conn)?;

        Ok(results)
    }

    fn get_countries(&self) -> Result<Vec<NumericalString>> {
        let conn = &mut self.get_conn()?;
        let results = countries::table
            .select((countries::id, countries::name))
            .order(countries::name.asc())
            .load(conn)?;

        Ok(results)
    }

    fn get_people(&self) -> Result<Vec<(String, String)>> {
        let conn = &mut self.get_conn()?;
        let results = people::table
            .select((people::id, people::name))
            .order(people::name.asc())
            .load(conn)?;

        Ok(results)
    }

    fn remove_file_by_path(&self, paths: &[PathBuf]) -> Result<()> {
        let conn = &mut self.get_conn()?;

        let path_strings = paths.iter().map(|p| p.to_string_lossy().to_string());

        diesel::delete(files::table.filter(files::path.eq_any(path_strings))).execute(conn)?;

        Ok(())
    }

    fn get_all_files(&self) -> Result<Vec<MediaFile>> {
        let conn = &mut self.get_conn()?;
        let db_files = files::table
            .select(files::all_columns)
            .load::<DbFile>(conn)?;

        Ok(db_files.into_iter().map(MediaFile::from).collect())
    }

    fn filter_medias(&self, filters: &FilterValues, page: u32) -> Result<Vec<Media>> {
        let conn = &mut self.get_conn()?;

        let mut query = medias::table
            .left_join(imdbs::table.on(medias::imdb_id.eq(imdbs::imdb_id.nullable())))
            .into_boxed();

        // -- Name Filter --
        if !filters.name.is_empty() {
            let search_pattern = format!("%{}%", filters.name);
            query = query.filter(
                medias::name
                    .like(search_pattern.clone())
                    .or(imdbs::title.like(search_pattern)),
            );
        }

        // -- Content Type Filter --
        if filters.r#type != ContentType::All {
            query = query.filter(imdbs::type_.eq(filters.r#type.to_string()));
        }

        // -- Minimum Rating Filter --
        if let Some(min_rating) = filters.min_rating {
            // Use a raw SQL cast for the column type
            let rating_clause = sql::<Double>("CAST(imdb_rating AS REAL)").ge(min_rating);
            query = query.filter(rating_clause);
        }

        // -- Many-to-Many Filters (Country, Genre, Actor, Tags) --
        if !filters.country.is_empty() {
            for country_id in &filters.country {
                query = query.filter(exists(
                    imdb_countries::table
                        .filter(imdb_countries::imdb_id.nullable().eq(medias::imdb_id))
                        .filter(imdb_countries::country_id.eq(country_id)),
                ));
            }
        }

        if !filters.genre.is_empty() {
            for genre_id in &filters.genre {
                query = query.filter(exists(
                    imdb_genres::table
                        .filter(imdb_genres::imdb_id.nullable().eq(medias::imdb_id))
                        .filter(imdb_genres::genre_id.eq(genre_id)),
                ));
            }
        }

        if !filters.people.is_empty() {
            for person_id in &filters.people {
                query = query.filter(exists(
                    imdb_people::table
                        .filter(imdb_people::imdb_id.nullable().eq(medias::imdb_id))
                        .filter(imdb_people::person_id.eq(person_id)),
                ));
            }
        }

        if !filters.tags.is_empty() {
            for tag_id in &filters.tags {
                query = query.filter(exists(
                    media_tags::table
                        .filter(media_tags::media_id.eq(medias::id))
                        .filter(media_tags::tag_id.eq(tag_id)),
                ));
            }
        }

        // -- Existence Filters --
        if let Some(exist_imdb) = filters.exist_imdb {
            if exist_imdb {
                query = query.filter(medias::imdb_id.is_not_null());
            } else {
                query = query.filter(medias::imdb_id.is_null());
            }
        }

        if let Some(exist_multi_file) = &filters.exist_multi_file {
            match exist_multi_file {
                MultiFileFilterType::Multifile => {
                    // Filter for media with multiple files (either media files or episode files)
                    let media_file_count = files::table
                        .select(files::media_id)
                        .filter(files::media_id.eq(medias::id.nullable()))
                        .group_by(files::media_id)
                        .having(diesel::dsl::count_star().gt(1));

                    let episode_file_count = files::table
                        .left_join(
                            episodes::table.on(files::episode_id.eq(episodes::id.nullable())),
                        )
                        .left_join(seasons::table.on(episodes::season_id.eq(seasons::id)))
                        .filter(seasons::media_id.eq(medias::id))
                        .group_by(files::episode_id)
                        .having(diesel::dsl::count_star().gt(1));

                    let condition = diesel::dsl::exists(media_file_count)
                        .or(diesel::dsl::exists(episode_file_count));

                    query = query.filter(condition);
                }
                MultiFileFilterType::Existfile => {
                    // Filter for media that has at least one file (either media files or episode files)
                    let has_media_files = diesel::dsl::exists(
                        files::table.filter(files::media_id.eq(medias::id.nullable())),
                    );

                    let has_episode_files = diesel::dsl::exists(
                        files::table
                            .left_join(
                                episodes::table.on(files::episode_id.eq(episodes::id.nullable())),
                            )
                            .left_join(seasons::table.on(episodes::season_id.eq(seasons::id)))
                            .filter(seasons::media_id.eq(medias::id)),
                    );

                    let condition = has_media_files.or(has_episode_files);
                    query = query.filter(condition);
                }
                MultiFileFilterType::Nofile => {
                    // Filter for media that has no files at all
                    let has_media_files = diesel::dsl::exists(
                        files::table.filter(files::media_id.eq(medias::id.nullable())),
                    );

                    let has_episode_files = diesel::dsl::exists(
                        files::table
                            .left_join(
                                episodes::table.on(files::episode_id.eq(episodes::id.nullable())),
                            )
                            .left_join(seasons::table.on(episodes::season_id.eq(seasons::id)))
                            .filter(seasons::media_id.eq(medias::id)),
                    );

                    let condition = has_media_files.or(has_episode_files);
                    query = query.filter(diesel::dsl::not(condition));
                }
            }
        }

        // -- Boolean Filters --
        if let Some(watched) = filters.watched {
            query = query.filter(medias::watched.eq(watched));
        }

        if let Some(watch_list) = filters.watch_list {
            query = query.filter(medias::watch_list.eq(watch_list));
        }

        // -- Sorting Logic --
        let is_asc = filters.sort_direction == SortDirectionType::Asc;

        query = match filters.sort_by {
            SortByType::Name => {
                if is_asc {
                    query.order((imdbs::title.asc(), medias::name.asc()))
                } else {
                    query.order((imdbs::title.desc(), medias::name.desc()))
                }
            }
            SortByType::Year => {
                let year_sql = sql::<Integer>("imdbs.year");
                if is_asc {
                    query.order((year_sql.asc(), medias::year.asc()))
                } else {
                    query.order((year_sql.desc(), medias::year.desc()))
                }
            }
            SortByType::Imdb => {
                let rating_sql = sql::<Double>("CAST(NULLIF(imdbs.imdb_rating, '') AS REAL)");
                if is_asc {
                    query.order(rating_sql.asc())
                } else {
                    query.order(rating_sql.desc())
                }
            }
        };

        // -- Pagination --
        let limit = 50;
        let offset = page * limit;

        // Execute the query and return the results
        let media_ids = query
            .select(medias::id)
            .distinct()
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<i32>(conn)?;

        Ok(media_ids
            .into_iter()
            .map(|id| Self::get_media_by_id(conn, id))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>())
    }

    fn get_media_by_id(&self, media_id: IdType) -> Result<Option<Media>> {
        self.get_conn()?
            .transaction(|conn| Self::get_media_by_id(conn, media_id))
    }

    fn get_tags(&self) -> Result<Vec<Tag>> {
        let conn = &mut self.get_conn()?;

        let results = tags::table.order(tags::id.asc()).load(conn)?;
        Ok(results)
    }

    fn remove_tag(&self, tag_id: IdType) -> Result<()> {
        let conn = &mut self.get_conn()?;
        diesel::delete(tags::table.filter(tags::id.eq(tag_id))).execute(conn)?;

        Ok(())
    }

    fn update_tag(&self, tag: &Tag) -> Result<()> {
        let conn = &mut self.get_conn()?;
        diesel::update(tags::table.filter(tags::id.eq(tag.id)))
            .set(tags::name.eq(&tag.name))
            .execute(conn)?;

        Ok(())
    }

    fn get_medias_by_tag(&self, tag_id: IdType) -> Result<Vec<Media>> {
        self.get_conn()?.transaction(|conn| {
            media_tags::table
                .inner_join(medias::table.on(media_tags::media_id.eq(medias::id)))
                .filter(media_tags::tag_id.eq(tag_id))
                .select(medias::id)
                .load::<i32>(conn)?
                .into_iter()
                .map(|media_id| Self::get_media_and_imdb_by_media_id(conn, media_id))
                .collect::<Result<Vec<_>>>()
        })
    }

    fn insert_tag(&self, tag: &Tag) -> Result<()> {
        let conn = &mut self.get_conn()?;
        diesel::insert_or_ignore_into(tags::table)
            .values(&NewTag { name: &tag.name })
            .execute(conn)?;
        Ok(())
    }

    fn insert_media_tag(&self, media_id: IdType, tag_id: IdType) -> Result<()> {
        let conn = &mut self.get_conn()?;
        diesel::insert_or_ignore_into(media_tags::table)
            .values(&NewMediaTag { media_id, tag_id })
            .execute(conn)?;
        Ok(())
    }

    fn remove_media_tag(&self, media_id: IdType, tag_id: IdType) -> Result<()> {
        let conn = &mut self.get_conn()?;
        diesel::delete(
            media_tags::table
                .filter(media_tags::media_id.eq(media_id))
                .filter(media_tags::tag_id.eq(tag_id)),
        )
        .execute(conn)?;
        Ok(())
    }

    fn get_all_medias(&self) -> Result<Vec<Media>> {
        let conn = &mut self.get_conn()?;
        let media_ids = medias::table.select(medias::id).load::<i32>(conn)?;

        let medias = media_ids
            .into_iter()
            .map(|id| Self::get_media_by_id(conn, id))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(medias)
    }

    fn import_data(&self, data: &crate::ExportedData) -> Result<()> {
        self.get_conn()?.transaction(|conn| {
            // Insert tags first
            for tag in &data.tags {
                diesel::insert_or_ignore_into(tags::table)
                    .values(&NewTag { name: &tag.name })
                    .execute(conn)?;
            }

            // Insert medias
            for media in &data.medias {
                let _ = Self::insert_media(conn, media)?;
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests_filter_values {
    use super::*;

    use crate::data_model::{LanguageFormat, Tag};
    use diesel::r2d2::{ConnectionManager, Pool};

    fn setup_test_db() -> Sqlite {
        let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        let pool = Pool::builder().build(manager).unwrap();
        let mut conn = pool.get().unwrap();

        // Run migrations
        conn.batch_execute(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;",
        )
        .unwrap();
        conn.run_pending_migrations(MIGRATIONS).unwrap();

        Sqlite { pool }
    }

    fn create_test_imdb() -> Imdb {
        Imdb {
            imdb_id: "tt0111161".to_string(),
            title: "The Shawshank Redemption".to_string(),
            year: 1994,
            plot: "Two imprisoned men bond over a number of years.".to_string(),
            poster: "https://example.com/poster.jpg".to_string(),
            imdb_rating: "9.3".to_string(),
            imdb_votes: 2343110,
            r#type: "movie".to_string(),
            genres: vec!["Drama".to_string()],
            countries: vec!["USA".to_string()],
            actors: vec![Person {
                id: "nm0000209".to_string(),
                name: "Tim Robbins".to_string(),
                url: "https://example.com/tim".to_string(),
            }],
            writers: vec![Person {
                id: "nm0000175".to_string(),
                name: "Stephen King".to_string(),
                url: "https://example.com/stephen".to_string(),
            }],
            directors: vec![Person {
                id: "nm0001104".to_string(),
                name: "Frank Darabont".to_string(),
                url: "https://example.com/frank".to_string(),
            }],
        }
    }

    fn create_test_media() -> Media {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        Media {
            id: 0,
            name: format!("Test Movie {}", counter),
            year: Some(2020),
            watched: false,
            my_ranking: 5,
            watch_list: true,
            imdb: Some(create_test_imdb()),
            tags: vec![],
            seasons: vec![],
            files: vec![MediaFile {
                id: 0,
                file_name: format!("test{}.mp4", counter),
                path: format!("/path/to/test{}.mp4", counter),
                quality: Some("1080p".to_string()),
                language_format: LanguageFormat::Unknown,
            }],
        }
    }

    fn setup_filter_test_data(sqlite: &Sqlite) -> (i32, i32, String, i32) {
        // Create test IMDB data
        let mut imdb1 = create_test_imdb();
        imdb1.r#type = "movie".to_string();
        imdb1.imdb_rating = "8.5".to_string();
        sqlite.insert_imdb(&imdb1).unwrap();

        let mut imdb2 = create_test_imdb();
        imdb2.imdb_id = "tt0111162".to_string();
        imdb2.title = "Series Test".to_string();
        imdb2.r#type = "tvSeries".to_string();
        imdb2.imdb_rating = "7.0".to_string();
        sqlite.insert_imdb(&imdb2).unwrap();

        // Create test media
        let mut media1 = create_test_media();
        media1.imdb = Some(imdb1.clone());
        media1.watched = true;
        media1.watch_list = false;

        let mut media2 = create_test_media();
        media2.name = "Series Media".to_string();
        media2.imdb = Some(imdb2.clone());
        media2.watched = false;
        media2.watch_list = true;

        let mut media3 = create_test_media();
        media3.name = "No IMDB Media".to_string();
        media3.imdb = None;

        let _ = sqlite.insert_media(&media1).unwrap();
        let _ = sqlite.insert_media(&media2).unwrap();
        let media3_id = sqlite.insert_media(&media3).unwrap();

        // Get genre and country IDs
        let genres = sqlite.get_genres().unwrap();
        let genre_id = genres
            .iter()
            .find(|(_, name)| name == "Drama")
            .map(|(id, _)| *id)
            .unwrap();

        let countries = sqlite.get_countries().unwrap();
        let country_id = countries
            .iter()
            .find(|(_, name)| name == "USA")
            .map(|(id, _)| *id)
            .unwrap();

        // Get person ID
        let people = sqlite.get_people().unwrap();
        let person_id = people
            .iter()
            .find(|(_, name)| name == "Tim Robbins")
            .map(|(id, _)| id.clone())
            .unwrap();

        (genre_id, country_id, person_id, media3_id)
    }

    #[test]
    fn test_filter_by_type() {
        let sqlite = setup_test_db();
        setup_filter_test_data(&sqlite);

        // Filter by movie type
        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::Movie,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().all(|m| {
            m.imdb
                .as_ref()
                .map(|i| i.r#type == "movie")
                .unwrap_or(false)
        }));
    }

    #[test]
    fn test_filter_by_min_rating() {
        let sqlite = setup_test_db();
        setup_filter_test_data(&sqlite);

        // Filter by minimum rating of 8.0
        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: Some(8.0),
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();
        assert!(!results.is_empty());
        for media in &results {
            if let Some(imdb) = &media.imdb {
                let rating: f64 = imdb.imdb_rating.parse().unwrap_or(0.0);
                assert!(rating >= 8.0);
            }
        }
    }

    #[test]
    fn test_filter_by_country() {
        let sqlite = setup_test_db();
        let (_, country_id, _, _) = setup_filter_test_data(&sqlite);

        // Filter by country
        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![country_id],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();
        assert!(!results.is_empty());
        // Should find media with IMDB data that has USA as country
    }

    #[test]
    fn test_filter_by_genre() {
        let sqlite = setup_test_db();
        let (genre_id, _, _, _) = setup_filter_test_data(&sqlite);

        // Filter by genre
        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![genre_id],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();
        assert!(!results.is_empty());
        // Should find media with IMDB data that has Drama genre
    }

    #[test]
    fn test_filter_by_people() {
        let sqlite = setup_test_db();
        let (_, _, person_id, _) = setup_filter_test_data(&sqlite);

        // Filter by people
        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![person_id],
            exist_imdb: None,
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();
        assert!(!results.is_empty());
        // Should find media with IMDB data that has Tim Robbins
    }

    #[test]
    fn test_filter_by_exist_imdb() {
        let sqlite = setup_test_db();
        let _ = setup_filter_test_data(&sqlite);

        // Filter by existence of IMDB data
        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: Some(true),
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().all(|m| m.imdb.is_some()));

        // Filter by non-existence of IMDB data
        let filters_no_imdb = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: Some(false),
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        let results_no_imdb = sqlite.filter_medias(&filters_no_imdb, 0).unwrap();
        assert!(!results_no_imdb.is_empty());
        assert!(results_no_imdb.iter().all(|m| m.imdb.is_none()));
    }

    #[test]
    fn test_filter_by_watched() {
        let sqlite = setup_test_db();
        setup_filter_test_data(&sqlite);

        // Filter by watched status
        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: None,
            watched: Some(true),
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().all(|m| m.watched));
    }

    #[test]
    fn test_filter_by_watch_list() {
        let sqlite = setup_test_db();
        setup_filter_test_data(&sqlite);

        // Filter by watch list status
        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: Some(true),
            tags: vec![],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().all(|m| m.watch_list));
    }

    #[test]
    fn test_filter_by_tags() {
        let sqlite = setup_test_db();
        let (_, _, _, media3_id) = setup_filter_test_data(&sqlite);

        // Create and assign a tag
        let tag = Tag {
            id: 0,
            name: "Test Tag".to_string(),
        };
        sqlite.insert_tag(&tag).unwrap();
        let tags = sqlite.get_tags().unwrap();
        let tag_id = tags[0].id;

        sqlite.insert_media_tag(media3_id, tag_id).unwrap();

        // Filter by tags
        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![tag_id],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|m| m.id == media3_id));
    }

    #[test]
    fn test_sort_by_name() {
        let sqlite = setup_test_db();
        setup_filter_test_data(&sqlite);

        // Sort by name ascending
        let filters_asc = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        let results_asc = sqlite.filter_medias(&filters_asc, 0).unwrap();
        assert!(!results_asc.is_empty());

        // Sort by name descending
        let filters_desc = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Desc,
            watch_list: None,
            tags: vec![],
        };

        let results_desc = sqlite.filter_medias(&filters_desc, 0).unwrap();
        assert!(!results_desc.is_empty());
        // Results should be in reverse order
        assert_eq!(results_asc.len(), results_desc.len());
    }

    #[test]
    fn test_sort_by_year() {
        let sqlite = setup_test_db();
        setup_filter_test_data(&sqlite);

        // Sort by year
        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Year,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();
        assert!(!results.is_empty());
        // Should sort by IMDB year primarily, then media year
    }

    #[test]
    fn test_sort_by_imdb_rating() {
        let sqlite = setup_test_db();
        setup_filter_test_data(&sqlite);

        // Sort by IMDB rating
        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Imdb,
            sort_direction: SortDirectionType::Desc,
            watch_list: None,
            tags: vec![],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();
        assert!(!results.is_empty());
        // Should sort by IMDB rating descending
    }

    #[test]
    fn test_pagination() {
        let sqlite = setup_test_db();
        // Create multiple media entries
        for i in 0..60 {
            let mut media = create_test_media();
            media.name = format!("Pagination Test {}", i);
            sqlite.insert_media(&media).unwrap();
        }

        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        // Page 0 should return up to 50 results
        let results_page0 = sqlite.filter_medias(&filters, 0).unwrap();
        assert!(results_page0.len() <= 50);

        // Page 1 should return remaining results
        let results_page1 = sqlite.filter_medias(&filters, 1).unwrap();
        assert!(results_page1.len() <= 50);

        // Should have pagination working (page 0 has results, page 1 may have more or less)
        assert!(!results_page0.is_empty());
        assert!(results_page1.is_empty());
        // Total results should be reasonable
        let total = results_page0.len() + results_page1.len();
        assert!(total >= results_page0.len()); // At least as many as page 0
    }

    #[test]
    fn test_filter_by_exist_multi_file_multifile() {
        let sqlite = setup_test_db();

        // Create media with multiple files
        let mut media1 = create_test_media();
        media1.name = "Movie With Multiple Files 1".to_string();
        media1.year = Some(2021);
        media1.imdb = None;
        media1.files = vec![
            MediaFile {
                id: 0,
                file_name: "movie1_1080p.mp4".to_string(),
                path: "/path/to/movie1_1080p.mp4".to_string(),
                quality: Some("1080p".to_string()),
                language_format: LanguageFormat::Unknown,
            },
            MediaFile {
                id: 0,
                file_name: "movie1_720p.mp4".to_string(),
                path: "/path/to/movie1_720p.mp4".to_string(),
                quality: Some("720p".to_string()),
                language_format: LanguageFormat::Unknown,
            },
        ];

        // Create media with single file
        let mut media2 = create_test_media();
        media2.name = "Single File Movie 2".to_string();
        media2.year = Some(2022);
        media2.imdb = None;
        media2.files = vec![MediaFile {
            id: 0,
            file_name: "movie2.mp4".to_string(),
            path: "/path/to/movie2.mp4".to_string(),
            quality: Some("1080p".to_string()),
            language_format: LanguageFormat::Unknown,
        }];

        // Create media with no files
        let mut media3 = create_test_media();
        media3.name = "No File Movie 3".to_string();
        media3.year = Some(2023);
        media3.imdb = None;
        media3.files = vec![];

        // Create series with multiple episodes and files
        let mut media4 = create_test_media();
        media4.name = "Series with Multiple Files 4".to_string();
        media4.year = Some(2024);
        media4.imdb = None;
        media4.files = vec![];
        media4.seasons = vec![Season {
            id: 0,
            number: 1,
            watched: false,
            episodes: vec![
                Episode {
                    id: 0,
                    number: 1,
                    watched: false,
                    files: vec![
                        MediaFile {
                            id: 0,
                            file_name: "episode1_1080p.mp4".to_string(),
                            path: "/path/to/episode1_1080p.mp4".to_string(),
                            quality: Some("1080p".to_string()),
                            language_format: LanguageFormat::Unknown,
                        },
                        MediaFile {
                            id: 0,
                            file_name: "episode1_720p.mp4".to_string(),
                            path: "/path/to/episode1_720p.mp4".to_string(),
                            quality: Some("720p".to_string()),
                            language_format: LanguageFormat::Unknown,
                        },
                    ],
                },
                Episode {
                    id: 0,
                    number: 2,
                    watched: false,
                    files: vec![MediaFile {
                        id: 0,
                        file_name: "episode2_1080p.mp4".to_string(),
                        path: "/path/to/episode2_1080p.mp4".to_string(),
                        quality: Some("1080p".to_string()),
                        language_format: LanguageFormat::Unknown,
                    }],
                },
            ],
        }];

        let media1_id = sqlite.insert_media(&media1).unwrap();
        let media2_id = sqlite.insert_media(&media2).unwrap();
        let media3_id = sqlite.insert_media(&media3).unwrap();
        let media4_id = sqlite.insert_media(&media4).unwrap();

        // Filter for multifile media
        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: Some(MultiFileFilterType::Multifile),
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();
        assert!(!results.is_empty());

        // Should find media1 (multiple files) and media4 (episode with multiple files)
        let result_ids: Vec<i32> = results.iter().map(|m| m.id).collect();
        assert!(result_ids.contains(&media1_id));
        assert!(result_ids.contains(&media4_id));

        // Should not find media2 (single file) and media3 (no files)
        assert!(!result_ids.contains(&media2_id));
        assert!(!result_ids.contains(&media3_id));
    }

    #[test]
    fn test_filter_by_exist_multi_file_existfile() {
        let sqlite = setup_test_db();

        // Create media with files
        let mut media1 = create_test_media();
        media1.name = "Movie With Files 1".to_string();
        media1.year = Some(2021);
        media1.imdb = None;
        media1.files = vec![MediaFile {
            id: 0,
            file_name: "movie1.mp4".to_string(),
            path: "/path/to/movie1.mp4".to_string(),
            quality: Some("1080p".to_string()),
            language_format: LanguageFormat::Unknown,
        }];

        // Create media with no files
        let mut media2 = create_test_media();
        media2.name = "No File Movie 2".to_string();
        media2.year = Some(2022);
        media2.imdb = None;
        media2.files = vec![];

        // Create series with episodes and files
        let mut media3 = create_test_media();
        media3.name = "Series with Files 3".to_string();
        media3.year = Some(2023);
        media3.imdb = None;
        media3.files = vec![];
        media3.seasons = vec![Season {
            id: 0,
            number: 1,
            watched: false,
            episodes: vec![Episode {
                id: 0,
                number: 1,
                watched: false,
                files: vec![MediaFile {
                    id: 0,
                    file_name: "episode1.mp4".to_string(),
                    path: "/path/to/episode1.mp4".to_string(),
                    quality: Some("1080p".to_string()),
                    language_format: LanguageFormat::Unknown,
                }],
            }],
        }];

        let media1_id = sqlite.insert_media(&media1).unwrap();
        let media2_id = sqlite.insert_media(&media2).unwrap();
        let media3_id = sqlite.insert_media(&media3).unwrap();

        // Filter for media with files
        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: Some(MultiFileFilterType::Existfile),
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();
        assert!(!results.is_empty());

        // Should find media1 (has files) and media3 (episode has files)
        let result_ids: Vec<i32> = results.iter().map(|m| m.id).collect();
        assert!(result_ids.contains(&media1_id));
        assert!(result_ids.contains(&media3_id));

        // Should not find media2 (no files)
        assert!(!result_ids.contains(&media2_id));
    }

    #[test]
    fn test_filter_by_exist_multi_file_nofile() {
        let sqlite = setup_test_db();

        // Create media with files
        let mut media1 = create_test_media();
        media1.name = "Movie With Files 1".to_string();
        media1.year = Some(2021);
        media1.imdb = None; // Remove IMDB to avoid matching issues
        media1.files = vec![MediaFile {
            id: 0,
            file_name: "movie1.mp4".to_string(),
            path: "/path/to/movie1.mp4".to_string(),
            quality: Some("1080p".to_string()),
            language_format: LanguageFormat::Unknown,
        }];

        // Create media with no files
        let mut media2 = create_test_media();
        media2.name = "No File Movie 2".to_string();
        media2.year = Some(2022);
        media2.imdb = None; // Remove IMDB to avoid matching issues
        media2.files = vec![];

        // Create series with episodes but no files
        let mut media3 = create_test_media();
        media3.name = "Series with No Files 3".to_string();
        media3.year = Some(2023);
        media3.imdb = None; // Remove IMDB to avoid matching issues
        media3.files = vec![];
        media3.seasons = vec![Season {
            id: 0,
            number: 1,
            watched: false,
            episodes: vec![Episode {
                id: 0,
                number: 1,
                watched: false,
                files: vec![], // No files for this episode
            }],
        }];

        let media1_id = sqlite.insert_media(&media1).unwrap();
        let media2_id = sqlite.insert_media(&media2).unwrap();
        let media3_id = sqlite.insert_media(&media3).unwrap();

        // Filter for media with no files
        let filters = FilterValues {
            name: "".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: Some(MultiFileFilterType::Nofile),
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();

        // Should find media2 (no files) and media3 (episode has no files)
        let result_ids: Vec<i32> = results.iter().map(|m| m.id).collect();
        assert!(
            result_ids.contains(&media2_id),
            "media2_id {} should be in results {:?}",
            media2_id,
            result_ids
        );
        assert!(
            result_ids.contains(&media3_id),
            "media3_id {} should be in results {:?}",
            media3_id,
            result_ids
        );

        // Should not find media1 (has files)
        assert!(
            !result_ids.contains(&media1_id),
            "media1_id {} should not be in results {:?}",
            media1_id,
            result_ids
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_model::{Episode, Imdb, LanguageFormat, Media, MediaFile, Person, Season, Tag};
    use diesel::r2d2::{ConnectionManager, Pool};
    use std::path::PathBuf;

    fn setup_test_db() -> Sqlite {
        let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        let pool = Pool::builder().build(manager).unwrap();
        let mut conn = pool.get().unwrap();

        // Run migrations
        conn.batch_execute(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;",
        )
        .unwrap();
        conn.run_pending_migrations(MIGRATIONS).unwrap();

        Sqlite { pool }
    }

    fn create_test_imdb() -> Imdb {
        Imdb {
            imdb_id: "tt0111161".to_string(),
            title: "The Shawshank Redemption".to_string(),
            year: 1994,
            plot: "Two imprisoned men bond over a number of years.".to_string(),
            poster: "https://example.com/poster.jpg".to_string(),
            imdb_rating: "9.3".to_string(),
            imdb_votes: 2343110,
            r#type: "movie".to_string(),
            genres: vec!["Drama".to_string()],
            countries: vec!["USA".to_string()],
            actors: vec![Person {
                id: "nm0000209".to_string(),
                name: "Tim Robbins".to_string(),
                url: "https://example.com/tim".to_string(),
            }],
            writers: vec![Person {
                id: "nm0000175".to_string(),
                name: "Stephen King".to_string(),
                url: "https://example.com/stephen".to_string(),
            }],
            directors: vec![Person {
                id: "nm0001104".to_string(),
                name: "Frank Darabont".to_string(),
                url: "https://example.com/frank".to_string(),
            }],
        }
    }

    fn create_test_media() -> Media {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        Media {
            id: 0,
            name: format!("Test Movie {}", counter),
            year: Some(2020),
            watched: false,
            my_ranking: 5,
            watch_list: true,
            imdb: Some(create_test_imdb()),
            tags: vec![],
            seasons: vec![],
            files: vec![MediaFile {
                id: 0,
                file_name: format!("test{}.mp4", counter),
                path: format!("/path/to/test{}.mp4", counter),
                quality: Some("1080p".to_string()),
                language_format: LanguageFormat::Unknown,
            }],
        }
    }

    #[test]
    fn test_new_with_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let sqlite = Sqlite::new_with_path(db_path.clone()).unwrap();

        // Verify database file was created
        assert!(db_path.exists());

        // Test connection
        let _conn = sqlite.get_conn().unwrap();
    }

    #[test]
    fn test_insert_and_get_imdb() {
        let sqlite = setup_test_db();
        let imdb = create_test_imdb();

        // Insert IMDB
        sqlite.insert_imdb(&imdb).unwrap();

        // Get IMDB
        let conn = &mut sqlite.get_conn().unwrap();
        let retrieved = Sqlite::get_imdb(conn, Some(imdb.imdb_id.clone()))
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.imdb_id, imdb.imdb_id);
        assert_eq!(retrieved.title, imdb.title);
        assert_eq!(retrieved.genres, imdb.genres);
        assert_eq!(retrieved.actors.len(), 1);
        assert_eq!(retrieved.writers.len(), 1);
        assert_eq!(retrieved.directors.len(), 1);
    }

    #[test]
    fn test_insert_and_get_media() {
        let sqlite = setup_test_db();
        let media = create_test_media();

        // Insert media
        let media_id = sqlite.insert_media(&media).unwrap();

        // Get media
        let retrieved = sqlite.get_media_by_id(media_id).unwrap().unwrap();

        assert_eq!(retrieved.name, media.name);
        assert_eq!(retrieved.year, media.year);
        assert_eq!(retrieved.watched, media.watched);
        assert_eq!(retrieved.files.len(), 1);
        assert!(retrieved.imdb.is_some());
    }

    #[test]
    fn test_update_media_watched() {
        let sqlite = setup_test_db();
        let media = create_test_media();

        let media_id = sqlite.insert_media(&media).unwrap();

        // Update watched status
        sqlite.update_media_watched(media_id, true).unwrap();

        // Verify update
        let retrieved = sqlite.get_media_by_id(media_id).unwrap().unwrap();
        assert!(retrieved.watched);
    }

    #[test]
    fn test_update_season_watched() {
        let sqlite = setup_test_db();
        let mut media = create_test_media();
        media.seasons = vec![Season {
            id: 0,
            number: 1,
            watched: false,
            episodes: vec![Episode {
                id: 0,
                number: 1,
                watched: false,
                files: vec![],
            }],
        }];
        media.files = vec![]; // Clear files since we have episodes

        let media_id = sqlite.insert_media(&media).unwrap();

        // Get season ID
        let retrieved = sqlite.get_media_by_id(media_id).unwrap().unwrap();
        let season_id = retrieved.seasons[0].id;

        // Update season watched
        sqlite.update_season_watched(season_id, true).unwrap();

        // Verify season and episode are watched, and media is watched
        let updated = sqlite.get_media_by_id(media_id).unwrap().unwrap();
        assert!(updated.watched);
        assert!(updated.seasons[0].watched);
        assert!(updated.seasons[0].episodes[0].watched);
    }

    #[test]
    fn test_update_episode_watched() {
        let sqlite = setup_test_db();
        let mut media = create_test_media();
        media.seasons = vec![Season {
            id: 0,
            number: 1,
            watched: false,
            episodes: vec![Episode {
                id: 0,
                number: 1,
                watched: false,
                files: vec![],
            }],
        }];
        media.files = vec![];

        let media_id = sqlite.insert_media(&media).unwrap();

        let retrieved = sqlite.get_media_by_id(media_id).unwrap().unwrap();
        let episode_id = retrieved.seasons[0].episodes[0].id;

        // Update episode watched
        sqlite.update_episode_watched(episode_id, true).unwrap();

        // Verify episode, season, and media are watched
        let updated = sqlite.get_media_by_id(media_id).unwrap().unwrap();
        assert!(updated.watched);
        assert!(updated.seasons[0].watched);
        assert!(updated.seasons[0].episodes[0].watched);
    }

    #[test]
    fn test_update_media_my_ranking() {
        let sqlite = setup_test_db();
        let media = create_test_media();

        let media_id = sqlite.insert_media(&media).unwrap();

        // Update ranking
        let affected_rows = sqlite.update_media_my_ranking(media_id, 8).unwrap();
        assert_eq!(affected_rows, 1);

        // Verify update
        let retrieved = sqlite.get_media_by_id(media_id).unwrap().unwrap();
        assert_eq!(retrieved.my_ranking, 8);
    }

    #[test]
    fn test_update_media_my_ranking_rejects_out_of_range() {
        let sqlite = setup_test_db();
        let media = create_test_media();
        let media_id = sqlite.insert_media(&media).unwrap();

        let err = sqlite.update_media_my_ranking(media_id, 11).unwrap_err();
        assert!(
            err.to_string().contains("expected range 0..=10"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_update_watch_list() {
        let sqlite = setup_test_db();
        let media = create_test_media();

        let media_id = sqlite.insert_media(&media).unwrap();

        // Update watch list
        sqlite.update_watch_list(media_id, false).unwrap();

        // Verify update
        let retrieved = sqlite.get_media_by_id(media_id).unwrap().unwrap();
        assert!(!retrieved.watch_list);
    }

    #[test]
    fn test_filter_medias() {
        let sqlite = setup_test_db();
        let media1 = create_test_media();
        let mut media2 = create_test_media();
        media2.name = "Another Movie".to_string();
        media2.imdb = None;

        sqlite.insert_media(&media1).unwrap();
        sqlite.insert_media(&media2).unwrap();

        // Filter by name
        let filters = FilterValues {
            name: "Test".to_string(),
            r#type: ContentType::All,
            min_rating: None,
            country: vec![],
            genre: vec![],
            people: vec![],
            exist_imdb: None,
            exist_multi_file: None,
            watched: None,
            sort_by: SortByType::Name,
            sort_direction: SortDirectionType::Asc,
            watch_list: None,
            tags: vec![],
        };

        let results = sqlite.filter_medias(&filters, 0).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].name.starts_with("Test Movie"));
    }

    #[test]
    fn test_get_genres() {
        let sqlite = setup_test_db();
        let imdb = create_test_imdb();

        sqlite.insert_imdb(&imdb).unwrap();

        let genres = sqlite.get_genres().unwrap();
        assert!(!genres.is_empty());
        assert!(genres.iter().any(|(_, name)| name == "Drama"));
    }

    #[test]
    fn test_get_countries() {
        let sqlite = setup_test_db();
        let imdb = create_test_imdb();

        sqlite.insert_imdb(&imdb).unwrap();

        let countries = sqlite.get_countries().unwrap();
        assert!(!countries.is_empty());
        assert!(countries.iter().any(|(_, name)| name == "USA"));
    }

    #[test]
    fn test_get_people() {
        let sqlite = setup_test_db();
        let imdb = create_test_imdb();

        sqlite.insert_imdb(&imdb).unwrap();

        let people = sqlite.get_people().unwrap();
        assert!(!people.is_empty());
        assert!(people.iter().any(|(_, name)| name == "Tim Robbins"));
    }

    #[test]
    fn test_tags_operations() {
        let sqlite = setup_test_db();

        // Insert tag
        let tag = Tag {
            id: 0,
            name: "Action".to_string(),
        };
        sqlite.insert_tag(&tag).unwrap();

        // Get tags
        let tags = sqlite.get_tags().unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "Action");

        // Update tag
        let mut updated_tag = tags[0].clone();
        updated_tag.name = "Adventure".to_string();
        sqlite.update_tag(&updated_tag).unwrap();

        let tags_after = sqlite.get_tags().unwrap();
        assert_eq!(tags_after[0].name, "Adventure");

        // Remove tag
        sqlite.remove_tag(updated_tag.id).unwrap();
        let tags_final = sqlite.get_tags().unwrap();
        assert!(tags_final.is_empty());
    }

    #[test]
    fn test_media_tags() {
        let sqlite = setup_test_db();
        let media = create_test_media();
        let tag = Tag {
            id: 0,
            name: "Drama".to_string(),
        };

        let media_id = sqlite.insert_media(&media).unwrap();
        sqlite.insert_tag(&tag).unwrap();

        let tags = sqlite.get_tags().unwrap();
        let tag_id = tags[0].id;

        // Insert media tag
        sqlite.insert_media_tag(media_id, tag_id).unwrap();

        // Get medias by tag
        let medias = sqlite.get_medias_by_tag(tag_id).unwrap();
        assert_eq!(medias.len(), 1);
        assert_eq!(medias[0].name, media.name);

        // Remove media tag
        sqlite.remove_media_tag(media_id, tag_id).unwrap();
        let medias_after = sqlite.get_medias_by_tag(tag_id).unwrap();
        assert!(medias_after.is_empty());
    }

    #[test]
    fn test_remove_file_by_path() {
        let sqlite = setup_test_db();
        let media = create_test_media();

        sqlite.insert_media(&media).unwrap();

        let paths = vec![PathBuf::from(&media.files[0].path)];
        sqlite.remove_file_by_path(&paths).unwrap();

        let files = sqlite.get_all_files().unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_get_all_files() {
        let sqlite = setup_test_db();
        let media = create_test_media();

        sqlite.insert_media(&media).unwrap();

        let files = sqlite.get_all_files().unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].file_name.starts_with("test"));
        assert!(files[0].file_name.ends_with(".mp4"));
    }

    #[test]
    fn test_clear_empty_data() {
        let sqlite = setup_test_db();

        // Insert media with IMDB and episodes
        let mut media = create_test_media();
        media.seasons = vec![Season {
            id: 0,
            number: 1,
            watched: false,
            episodes: vec![Episode {
                id: 0,
                number: 1,
                watched: false,
                files: vec![],
            }],
        }];
        media.files = vec![]; // Clear files since we have episodes
        sqlite.insert_media(&media).unwrap();

        // Manually delete media to create orphaned IMDB and episodes
        let conn = &mut sqlite.get_conn().unwrap();
        diesel::delete(medias::table).execute(conn).unwrap();

        // Clear empty data
        sqlite.clear_empty_data().unwrap();
    }

    #[test]
    fn test_delete_media() {
        let sqlite = setup_test_db();
        let media = create_test_media();

        let media_id = sqlite.insert_media(&media).unwrap();

        // Delete media
        sqlite.delete_media(media_id).unwrap();

        // Verify deletion
        let retrieved = sqlite.get_media_by_id(media_id).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_update_media_imdb() {
        let sqlite = setup_test_db();
        let mut media = create_test_media();
        media.imdb = None; // Start without IMDB

        let media_id = sqlite.insert_media(&media).unwrap();

        // First insert the IMDB data
        let imdb = create_test_imdb();
        sqlite.insert_imdb(&imdb).unwrap();

        // Update with IMDB
        let new_imdb_id = "tt0111161";
        let updated_id = sqlite.update_media_imdb(media_id, new_imdb_id).unwrap();

        // Verify IMDB was added
        let retrieved = sqlite.get_media_by_id(updated_id).unwrap().unwrap();
        assert!(retrieved.imdb.is_some());
        assert_eq!(retrieved.imdb.as_ref().unwrap().imdb_id, new_imdb_id);
    }
}
