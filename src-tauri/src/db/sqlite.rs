mod data_models;
pub mod schema;

use super::{
    ContentType, DB, FilterValues, NumericalString, Result, SortByType, SortDirectionType,
};
use crate::data_model::{Episode, IdType, Imdb, Media, MediaFile, Season, Tag};
use anyhow::Ok;
use data_models::{
    DbEpisode, DbFile, DbImdb, DbMedia, DbSeason, NewActor, NewCountry, NewDirector, NewEpisode,
    NewFile, NewGenre, NewImdb, NewImdbActor, NewImdbCountry, NewImdbDirector, NewImdbGenre,
    NewImdbLanguage, NewImdbWriter, NewLanguage, NewMedia, NewMediaTag, NewSeason, NewTag,
    NewWriter,
};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, NullableExpressionMethods, QueryDsl,
    RunQueryDsl, SqliteConnection,
    connection::SimpleConnection,
    dsl::{exists, sql},
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    sql_types::{BigInt, Double, Text},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
pub use schema::{
    actors, countries, directors, episodes, files, genres, imdb_actors, imdb_countries,
    imdb_directors, imdb_genres, imdb_languages, imdb_writers, imdbs, languages, media_tags,
    medias, seasons, tags, writers,
};
use std::path::PathBuf;
use tauri::Manager;

type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct Sqlite {
    pool: DbPool,
}

impl Sqlite {
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
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;",
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

    fn insert_or_get_id_director(conn: &mut SqliteConnection, name_val: &str) -> Result<i32> {
        diesel::insert_or_ignore_into(directors::table)
            .values(&NewDirector { name: name_val })
            .execute(conn)?;

        let id = directors::table
            .filter(directors::name.eq(name_val))
            .select(directors::id)
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

    fn insert_imdb_director_by_name(
        conn: &mut SqliteConnection,
        imdb_id_val: &str,
        entity_name: &str,
    ) -> Result<()> {
        let ent_id = Self::insert_or_get_id_director(conn, entity_name)?;
        diesel::insert_or_ignore_into(imdb_directors::table)
            .values(&NewImdbDirector {
                imdb_id: imdb_id_val,
                director_id: ent_id,
            })
            .execute(conn)?;
        Ok(())
    }

    fn insert_or_get_id_writer(conn: &mut SqliteConnection, name_val: &str) -> Result<i32> {
        diesel::insert_or_ignore_into(writers::table)
            .values(&NewWriter { name: name_val })
            .execute(conn)?;

        let id = writers::table
            .filter(writers::name.eq(name_val))
            .select(writers::id)
            .first(conn)?;
        Ok(id)
    }

    fn insert_imdb_writer_by_name(
        conn: &mut SqliteConnection,
        imdb_id_val: &str,
        entity_name: &str,
    ) -> Result<()> {
        let ent_id = Self::insert_or_get_id_writer(conn, entity_name)?;
        diesel::insert_or_ignore_into(imdb_writers::table)
            .values(&NewImdbWriter {
                imdb_id: imdb_id_val,
                writer_id: ent_id,
            })
            .execute(conn)?;
        Ok(())
    }

    fn insert_or_get_id_actor(conn: &mut SqliteConnection, name_val: &str) -> Result<i32> {
        diesel::insert_or_ignore_into(actors::table)
            .values(&NewActor { name: name_val })
            .execute(conn)?;

        let id = actors::table
            .filter(actors::name.eq(name_val))
            .select(actors::id)
            .first(conn)?;
        Ok(id)
    }

    fn insert_imdb_actor_by_name(
        conn: &mut SqliteConnection,
        imdb_id_val: &str,
        entity_name: &str,
    ) -> Result<()> {
        let ent_id = Self::insert_or_get_id_actor(conn, entity_name)?;
        diesel::insert_or_ignore_into(imdb_actors::table)
            .values(&NewImdbActor {
                imdb_id: imdb_id_val,
                actor_id: ent_id,
            })
            .execute(conn)?;
        Ok(())
    }

    fn insert_or_get_id_language(conn: &mut SqliteConnection, name_val: &str) -> Result<i32> {
        diesel::insert_or_ignore_into(languages::table)
            .values(&NewLanguage { name: name_val })
            .execute(conn)?;

        let id = languages::table
            .filter(languages::name.eq(name_val))
            .select(languages::id)
            .first(conn)?;
        Ok(id)
    }

    fn insert_imdb_language_by_name(
        conn: &mut SqliteConnection,
        imdb_id_val: &str,
        entity_name: &str,
    ) -> Result<()> {
        let ent_id = Self::insert_or_get_id_language(conn, entity_name)?;
        diesel::insert_or_ignore_into(imdb_languages::table)
            .values(&NewImdbLanguage {
                imdb_id: imdb_id_val,
                language_id: ent_id,
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

        diesel::insert_or_ignore_into(imdbs::table)
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

    fn insert_media(conn: &mut SqliteConnection, media: &Media) -> Result<()> {
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

        let id = if let Some(id) = existing_media_id {
            // Media already exists, use the existing ID
            id
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
            diesel::select(sql::<BigInt>("last_insert_rowid()")).get_result::<i64>(conn)? as i32
        };

        for season in &media.seasons {
            Self::insert_season(conn, id, season)?;
        }

        Self::insert_files(conn, &media.files, Some(id), None)?;

        Ok(())
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
        if imdb_id_val.is_none() {
            return Ok(None);
        }

        let imdb_id_val = &imdb_id_val.unwrap();

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

        imdb.directors = imdb_directors::table
            .inner_join(directors::table.on(imdb_directors::director_id.eq(directors::id)))
            .filter(imdb_directors::imdb_id.eq(imdb_id_val))
            .select(directors::name)
            .load(conn)?;

        imdb.writers = imdb_writers::table
            .inner_join(writers::table.on(imdb_writers::writer_id.eq(writers::id)))
            .filter(imdb_writers::imdb_id.eq(imdb_id_val))
            .select(writers::name)
            .load(conn)?;

        imdb.actors = imdb_actors::table
            .inner_join(actors::table.on(imdb_actors::actor_id.eq(actors::id)))
            .filter(imdb_actors::imdb_id.eq(imdb_id_val))
            .select(actors::name)
            .load(conn)?;

        imdb.languages = imdb_languages::table
            .inner_join(languages::table.on(imdb_languages::language_id.eq(languages::id)))
            .filter(imdb_languages::imdb_id.eq(imdb_id_val))
            .select(languages::name)
            .load(conn)?;

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
impl Sqlite {
    fn remove_empty_imdb(conn: &mut SqliteConnection) -> Result<()> {
        diesel::delete(imdbs::table.filter(diesel::dsl::not(diesel::dsl::exists(
            medias::table.filter(medias::imdb_id.eq(imdbs::imdb_id.nullable())),
        ))))
        .execute(conn)?;

        Ok(())
    }

    fn remove_empty_media(conn: &mut SqliteConnection) -> Result<()> {
        diesel::delete(
            medias::table.filter(
                diesel::dsl::not(diesel::dsl::exists(
                    files::table.filter(files::media_id.eq(medias::id.nullable())),
                ))
                .and(diesel::dsl::not(diesel::dsl::exists(
                    seasons::table.filter(seasons::media_id.eq(medias::id)),
                ))),
            ),
        )
        .execute(conn)?;

        Ok(())
    }

    fn remove_empty_seasons(conn: &mut SqliteConnection) -> Result<()> {
        diesel::delete(seasons::table.filter(diesel::dsl::not(diesel::dsl::exists(
            episodes::table.filter(episodes::season_id.eq(seasons::id)),
        ))))
        .execute(conn)?;

        Ok(())
    }

    fn remove_empty_episodes(conn: &mut SqliteConnection) -> Result<()> {
        diesel::delete(episodes::table.filter(diesel::dsl::not(diesel::dsl::exists(
            files::table.filter(files::episode_id.eq(episodes::id.nullable())),
        ))))
        .execute(conn)?;

        Ok(())
    }
}

impl DB for Sqlite {
    fn insert_medias(&self, media_list: &[Media]) -> Result<()> {
        self.get_conn()?.transaction(|conn| {
            for media in media_list {
                Self::insert_media(conn, media)?;
            }
            Ok(())
        })
    }

    fn update_media_my_ranking(&self, media_id: IdType, my_ranking: u8) -> Result<usize> {
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
            let existing_media = medias::table
                .filter(medias::imdb_id.eq(imdb_id))
                .filter(medias::id.ne(media_id))
                .first::<DbMedia>(conn)
                .optional()?;

            if let Some(existing) = existing_media {
                diesel::update(seasons::table.filter(seasons::media_id.eq(media_id)))
                    .set(seasons::media_id.eq(existing.id))
                    .execute(conn)?;

                diesel::update(files::table.filter(files::media_id.nullable().eq(media_id)))
                    .set(files::media_id.eq(existing.id))
                    .execute(conn)?;

                let media_tags = media_tags::table
                    .filter(media_tags::media_id.eq(media_id))
                    .select(media_tags::tag_id)
                    .load::<IdType>(conn)?;

                for tag_id in media_tags {
                    diesel::insert_or_ignore_into(media_tags::table)
                        .values(&NewMediaTag {
                            media_id: existing.id,
                            tag_id,
                        })
                        .execute(conn)?;
                }

                diesel::delete(medias::table.filter(medias::id.eq(media_id))).execute(conn)?;
                Ok(existing.id)
            } else {
                diesel::update(medias::table.filter(medias::id.eq(media_id)))
                    .set(medias::imdb_id.eq(imdb_id))
                    .execute(conn)?;
                Ok(media_id)
            }
        })
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

    fn get_actors(&self) -> Result<Vec<NumericalString>> {
        let conn = &mut self.get_conn()?;
        let results = actors::table
            .select((actors::id, actors::name))
            .order(actors::name.asc())
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

    fn filter_medias(&self, filters: &FilterValues) -> Result<Vec<Media>> {
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
            for country_id in filters.country.iter().map(|(id, _)| id) {
                query = query.filter(exists(
                    imdb_countries::table
                        .filter(imdb_countries::imdb_id.nullable().eq(medias::imdb_id))
                        .filter(imdb_countries::country_id.eq(country_id)),
                ));
            }
        }

        if !filters.genre.is_empty() {
            for genre_id in filters.genre.iter().map(|(id, _)| id) {
                query = query.filter(exists(
                    imdb_genres::table
                        .filter(imdb_genres::imdb_id.nullable().eq(medias::imdb_id))
                        .filter(imdb_genres::genre_id.eq(genre_id)),
                ));
            }
        }

        if !filters.actor.is_empty() {
            for actor_id in filters.actor.iter().map(|(id, _)| id) {
                query = query.filter(exists(
                    imdb_actors::table
                        .filter(imdb_actors::imdb_id.nullable().eq(medias::imdb_id))
                        .filter(imdb_actors::actor_id.eq(actor_id)),
                ));
            }
        }

        if !filters.tags.is_empty() {
            for tag_id in filters.tags.iter().map(|(id, _)| id) {
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

        if let Some(exist_multi_file) = filters.exist_multi_file {
            let media_file_count = files::table
                .select(files::media_id)
                .filter(files::media_id.eq(medias::id.nullable()))
                .group_by(files::media_id)
                .having(diesel::dsl::count_star().gt(1));

            let episode_file_count = files::table
                .left_join(episodes::table.on(files::episode_id.eq(episodes::id.nullable())))
                .left_join(seasons::table.on(episodes::season_id.eq(seasons::id)))
                .filter(seasons::media_id.eq(medias::id))
                .group_by(files::episode_id)
                .having(diesel::dsl::count_star().gt(1));

            let condition =
                diesel::dsl::exists(media_file_count).or(diesel::dsl::exists(episode_file_count));

            query = if exist_multi_file {
                query.filter(condition)
            } else {
                query.filter(diesel::dsl::not(condition))
            };
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
                let year_sql = sql::<Text>("NULLIF(imdbs.year, '')");
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

        // Execute the query and return the results
        let media_ids = query.select(medias::id).distinct().load::<i32>(conn)?;
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
}
