use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, Result, params};

use crate::{
    data_model::{Episode, Imdb, Media, MediaFile, Season},
    db::{ContentType, DB, FilterValues, SortByType},
};

#[derive(Clone)]
pub struct Sqlite {
    path: String,
}

impl Default for Sqlite {
    fn default() -> Self {
        Self {
            path: "movies.db".into(),
        }
    }
}

// helper
impl Sqlite {
    fn get_conn(&self) -> Result<Connection> {
        Connection::open(&self.path)
    }

    fn insert_or_get_id(conn: &Connection, table: &str, name: &str) -> Result<i64> {
        let insert_sql = format!("INSERT OR IGNORE INTO {table} (name) VALUES (?1)");
        conn.execute(&insert_sql, params![name])?;

        let select_sql = format!("SELECT id FROM {table} WHERE name = ?1");
        conn.query_row(&select_sql, params![name], |row| row.get(0))
    }

    fn insert_many_to_many(
        conn: &Connection,
        table: &str,
        field: &str,
        imdb_id: &str,
        value: &str,
    ) -> Result<()> {
        let entity_id = Self::insert_or_get_id(conn, table, value)?;
        let sql = format!("INSERT OR IGNORE INTO imdb_{table} (imdb_id, {field}) VALUES (?1, ?2)");
        conn.execute(&sql, params![imdb_id, entity_id])?;
        Ok(())
    }

    fn get_related_names(
        conn: &Connection,
        name_table: &str,
        relation_table: &str,
        imdb_id: &str,
        field: &str,
    ) -> Result<Vec<String>> {
        let query = format!(
            "
            SELECT {name_table}.name FROM {name_table}
            JOIN {relation_table} ON {name_table}.id = {relation_table}.{field}_id
            WHERE {relation_table}.imdb_id = ?1
            ORDER BY {name_table}.name
            "
        );

        let mut stmt = conn.prepare_cached(&query)?;
        let names = stmt
            .query_map(params![imdb_id], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        Ok(names)
    }

    fn get_imdb_id_for_media(conn: &Connection, media_id: i64) -> Result<Option<String>> {
        conn.query_row(
            "SELECT imdb_id FROM medias WHERE id = ?1",
            params![media_id],
            |row| row.get(0),
        )
        .optional()
    }
}

// insert
impl Sqlite {
    fn insert_imdb(conn: &Connection, imdb: &Imdb) -> Result<()> {
        conn.execute(
            "INSERT OR REPLACE INTO imdbs (
            imdb_id, title, year, rated, released, runtime, plot, awards,
            poster, imdb_rating, imdb_votes, box_office, total_seasons, type
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                imdb.imdb_id,
                imdb.title,
                imdb.year,
                imdb.rated,
                imdb.released,
                imdb.runtime,
                imdb.plot,
                imdb.awards,
                imdb.poster,
                imdb.imdb_rating,
                imdb.imdb_votes,
                imdb.box_office,
                imdb.total_seasons,
                imdb.r#type,
            ],
        )?;

        Self::insert_imdb_relationships(conn, imdb)?;

        Ok(())
    }

    fn insert_imdb_relationships(conn: &Connection, imdb: &Imdb) -> Result<()> {
        // Insert genres
        for g in &imdb.genres {
            Self::insert_many_to_many(conn, "genres", "genre_id", &imdb.imdb_id, g)?;
        }

        // Insert directors
        for d in &imdb.directors {
            Self::insert_many_to_many(conn, "directors", "director_id", &imdb.imdb_id, d)?;
        }

        // Insert writers
        for w in &imdb.writers {
            Self::insert_many_to_many(conn, "writers", "writer_id", &imdb.imdb_id, w)?;
        }

        // Insert actors
        for a in &imdb.actors {
            Self::insert_many_to_many(conn, "actors", "actor_id", &imdb.imdb_id, a)?;
        }

        // Insert languages
        for l in &imdb.languages {
            Self::insert_many_to_many(conn, "languages", "language_id", &imdb.imdb_id, l)?;
        }

        // Insert countries
        for c in &imdb.countries {
            Self::insert_many_to_many(conn, "countries", "country_id", &imdb.imdb_id, c)?;
        }

        Ok(())
    }

    fn insert_media(conn: &Connection, media: &Media) -> Result<i64> {
        let imdb_id = media
            .imdb
            .as_ref()
            .map(|imdb| imdb.imdb_id.as_str())
            .or(Some(""));

        if let Some(imdb) = &media.imdb {
            Self::insert_imdb(conn, imdb)?;
        }

        let mut stmt = conn.prepare_cached(
            "
                INSERT OR IGNORE INTO medias (name, year, watched, my_ranking, imdb_id)
                VALUES (?1, ?2, ?3, ?4, ?5)
                RETURNING id",
        )?;

        let result = stmt.query_row(
            params![
                &media.name,
                media.year,
                media.watched,
                media.my_ranking,
                imdb_id
            ],
            |row| row.get(0),
        );

        let media_id = match result {
            Ok(id) => id,
            Err(rusqlite::Error::QueryReturnedNoRows) => conn.query_row(
                "SELECT id FROM medias WHERE name = ?1 AND year = ?2",
                params![&media.name, media.year],
                |row| row.get(0),
            )?,
            Err(e) => return Err(e),
        };

        for season in &media.seasons {
            Self::insert_season(conn, media_id, season)?;
        }

        for file in &media.files {
            Self::insert_file(conn, file, Some(media_id), None)?;
        }

        Ok(media_id)
    }

    fn insert_season(conn: &Connection, media_id: i64, season: &Season) -> Result<i64> {
        let mut stmt = conn.prepare_cached(
            "
                INSERT INTO seasons (media_id, season_number, watched)
                VALUES (?1, ?2, ?3)
                RETURNING id
            ",
        )?;

        let season_id: i64 = stmt
            .query_row(params![media_id, season.number, season.watched], |row| {
                row.get(0)
            })?;

        // Insert associated episodes
        for episode in &season.episodes {
            Self::insert_episode(conn, season_id, episode)?;
        }

        Ok(season_id)
    }

    fn insert_episode(conn: &Connection, season_id: i64, episode: &Episode) -> Result<i64> {
        let mut stmt = conn.prepare_cached(
            "
                INSERT INTO episodes (season_id, episode_number, watched)
                VALUES (?1, ?2, ?3)
                RETURNING id
            ",
        )?;

        let episode_id: i64 = stmt
            .query_row(params![season_id, episode.number, episode.watched], |row| {
                row.get(0)
            })?;

        // Insert associated files
        for file in &episode.files {
            Self::insert_file(conn, file, None, Some(episode_id))?;
        }

        Ok(episode_id)
    }

    fn insert_file(
        conn: &Connection,
        file: &MediaFile,
        media_id: Option<i64>,
        episode_id: Option<i64>,
    ) -> Result<i64> {
        let mut stmt = conn.prepare_cached(
            "
                INSERT INTO files (media_id, episode_id, file_name, path, quality, language_format)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                RETURNING id
            ",
        )?;

        stmt.query_row(
            params![
                media_id,
                episode_id,
                file.file_name,
                file.path,
                file.quality,
                file.language_format
            ],
            |row| row.get(0),
        )
    }
}

// get
impl Sqlite {
    fn get_media_by_id(conn: &Connection, media_id: i64) -> Result<Option<Media>> {
        // Get basic media info
        let mut stmt = conn.prepare_cached(
            "
        SELECT id, name, year, watched, my_ranking, watch_list, imdb_id
        FROM medias
        WHERE id = ?1
    ",
        )?;

        let mut media = stmt.query_row(params![media_id], |row| Ok(Media::from(row)))?;

        // Get IMDb metadata if exists
        if let Some(imdb_id) = Self::get_imdb_id_for_media(conn, media_id)? {
            media.imdb = Self::get_imdb(conn, &imdb_id)?;
        }

        // Get seasons with episodes and files
        media.seasons = Self::get_seasons_for_media(conn, media_id)?;
        media.files = Self::get_files_for_media(conn, media_id)?;

        Ok(Some(media))
    }

    fn get_imdb(conn: &Connection, imdb_id: &str) -> Result<Option<Imdb>> {
        // Get basic metadata

        let mut stmt = conn.prepare_cached(
            "
        SELECT title, year, rated, released, runtime, plot, awards,
               poster, imdb_rating, imdb_votes, box_office, total_seasons, type, imdb_id
        FROM imdbs
        WHERE imdb_id = ?1
    ",
        )?;

        let mut imdb = match stmt
            .query_row(params![imdb_id], |row| Ok(Imdb::from(row)))
            .optional()?
        {
            Some(data) => data,
            None => return Ok(None),
        };

        // Get all related data
        imdb.genres = Self::get_related_names(conn, "genres", "imdb_genres", imdb_id, "genre")?;
        imdb.directors =
            Self::get_related_names(conn, "directors", "imdb_directors", imdb_id, "director")?;
        imdb.writers = Self::get_related_names(conn, "writers", "imdb_writers", imdb_id, "writer")?;
        imdb.actors = Self::get_related_names(conn, "actors", "imdb_actors", imdb_id, "actor")?;
        imdb.languages =
            Self::get_related_names(conn, "languages", "imdb_languages", imdb_id, "language")?;
        imdb.countries =
            Self::get_related_names(conn, "countries", "imdb_countries", imdb_id, "country")?;

        Ok(Some(imdb))
    }

    fn get_media_and_imdb_by_media_id(conn: &Connection, media_id: i64) -> Result<Media> {
        let mut stmt = conn.prepare_cached(
            "
                SELECT id, name, year, watched, my_ranking, watch_list, imdb_id
                FROM medias
                WHERE id = ?1
            ",
        )?;

        let mut media = stmt.query_row(params![media_id], |row| Ok(Media::from(row)))?;
        if let Some(imdb_id) = Self::get_imdb_id_for_media(conn, media_id)? {
            media.imdb = Self::get_imdb(conn, &imdb_id)?;
        }

        Ok(media)
    }

    fn get_seasons_for_media(conn: &Connection, media_id: i64) -> Result<Vec<Season>> {
        // First get all seasons for the media
        let mut stmt = conn.prepare_cached(
            "
            SELECT id, season_number, watched
            FROM seasons
            WHERE media_id = ?1
            ORDER BY season_number
        ",
        )?;

        let mut seasons: Vec<Season> = stmt
            .query_map(params![media_id], |row| Ok(Season::from(row)))?
            .collect::<Result<Vec<_>, _>>()?;

        // Then load episodes for each season
        for season in &mut seasons {
            season.episodes = Self::get_episodes_for_season(conn, season.id)?;
        }

        Ok(seasons)
    }

    fn get_episodes_for_season(conn: &Connection, season_id: i64) -> Result<Vec<Episode>> {
        // First get all episodes for the season
        let mut stmt = conn.prepare_cached(
            "
                SELECT id, episode_number, watched
                FROM episodes
                WHERE season_id = ?1
                ORDER BY episode_number
            ",
        )?;

        let mut episodes: Vec<Episode> = stmt
            .query_map(params![season_id], |row| Ok(Episode::from(row)))?
            .collect::<Result<Vec<_>, _>>()?;

        // Then load files for each episode
        for episode in &mut episodes {
            episode.files = Self::get_files_for_episode(conn, episode.id)?;
        }

        Ok(episodes)
    }

    fn get_files_for_media(conn: &Connection, media_id: i64) -> Result<Vec<MediaFile>> {
        let mut stmt = conn.prepare_cached(
            "
            SELECT id, file_name, path, quality, language_format
            FROM files
            WHERE media_id = ?1
        ",
        )?;

        let files = stmt
            .query_map(params![media_id], |row| Ok(MediaFile::from(row)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(files)
    }

    fn get_files_for_episode(conn: &Connection, episode_id: i64) -> Result<Vec<MediaFile>> {
        let mut stmt = conn.prepare_cached(
            "
            SELECT id, file_name, path, quality, language_format
            FROM files
            WHERE episode_id = ?1
        ",
        )?;

        let files = stmt
            .query_map(params![episode_id], |row| Ok(MediaFile::from(row)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(files)
    }

    fn get_file_by_path(conn: &Connection, path: &Path) -> Result<Option<MediaFile>> {
        let mut stmt = conn.prepare(
            "
            SELECT id, file_name, path, quality, language_format
                FROM files
                WHERE path = ?1",
        )?;

        let path_str = path.to_string_lossy();

        let files = stmt
            .query_row(params![path_str], |row| Ok(MediaFile::from(row)))
            .optional()?;

        Ok(files)
    }

    fn get_countries(conn: &Connection) -> Result<Vec<(usize, String)>> {
        let mut stmt = conn.prepare("SELECT id, name FROM countries ORDER BY name")?;

        let countries = stmt
            .query_map([], |row| {
                Ok((row.get::<_, usize>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(Result::ok)
            .collect();

        Ok(countries)
    }

    fn get_actors(conn: &Connection) -> Result<Vec<(usize, String)>> {
        let mut stmt = conn.prepare("SELECT id, name FROM actors ORDER BY name")?;

        let countries = stmt
            .query_map([], |row| {
                Ok((row.get::<_, usize>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(Result::ok)
            .collect();

        Ok(countries)
    }

    fn get_genres(conn: &Connection) -> Result<Vec<(usize, String)>> {
        let mut stmt = conn.prepare("SELECT id, name FROM genres ORDER BY name")?;

        let genres = stmt
            .query_map([], |row| {
                Ok((row.get::<_, usize>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(Result::ok)
            .collect();

        Ok(genres)
    }

    fn get_all_files(conn: &Connection) -> Result<Vec<MediaFile>> {
        let mut stmt = conn.prepare(
            "SELECT id, file_name, path, quality, language_format
                FROM files",
        )?;

        let files = stmt
            .query_map([], |row| Ok(MediaFile::from(row)))?
            .filter_map(Result::ok)
            .collect();

        Ok(files)
    }

    fn filter_medias(conn: &Connection, filters: &FilterValues) -> Result<Vec<Media>> {
        let mut where_conditions = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut query = r#"
        SELECT DISTINCT vm.id
        FROM medias vm
        LEFT JOIN imdbs im ON vm.imdb_id = im.imdb_id
    "#
        .to_string();

        if filters.r#type != ContentType::All {
            where_conditions.push("im.type = ?".to_string());
            params.push(Box::new(filters.r#type.to_string()));
        }

        if let Some(min_rating) = filters.min_rating {
            where_conditions.push("CAST(im.imdb_rating AS REAL) >= ?".to_string());
            params.push(Box::new(min_rating));
        }

        // Handle country filter (array of IDs)
        if !filters.country.is_empty() {
            query.push_str(" LEFT JOIN imdb_countries ic ON im.imdb_id = ic.imdb_id\n");
            let placeholders: Vec<String> =
                filters.country.iter().map(|_| "?".to_string()).collect();
            let in_clause = placeholders.join(",");
            where_conditions.push(format!("ic.country_id IN ({in_clause})"));
            for country_id in &filters.country {
                params.push(Box::new(country_id.0));
            }
        }

        if !filters.genre.is_empty() {
            query.push_str(" LEFT JOIN imdb_genres ig ON im.imdb_id = ig.imdb_id\n");
            let placeholders: Vec<String> = filters.genre.iter().map(|_| "?".to_string()).collect();
            let in_clause = placeholders.join(",");
            where_conditions.push(format!("ig.genre_id IN ({in_clause})"));
            for genre_id in &filters.genre {
                params.push(Box::new(genre_id.0));
            }
        }

        if !filters.actor.is_empty() {
            query.push_str(" LEFT JOIN imdb_actors ia ON im.imdb_id = ia.imdb_id\n");
            let placeholders: Vec<String> = filters.actor.iter().map(|_| "?".to_string()).collect();
            let in_clause = placeholders.join(",");
            where_conditions.push(format!("ia.actor_id IN ({in_clause})"));
            for actor_id in &filters.actor {
                params.push(Box::new(actor_id.0));
            }
        }

        if let Some(exist_imdb) = filters.exist_imdb {
            if exist_imdb {
                where_conditions.push("im.imdb_id IS NOT NULL".to_string());
            } else {
                where_conditions.push("im.imdb_id IS NULL".to_string());
            }
        }

        if let Some(exist_multi_file) = filters.exist_multi_file {
            query.push_str(" LEFT JOIN files vfd ON vm.id = vfd.media_id\n");
            if exist_multi_file {
                where_conditions.push(
                    "(
                SELECT COUNT(*) 
                FROM files 
                WHERE media_id = vm.id
            ) > 1"
                        .to_string(),
                );
            } else {
                where_conditions.push(
                    "(
                SELECT COUNT(*) 
                FROM files 
                WHERE media_id = vm.id
            ) <= 1"
                        .to_string(),
                );
            }
        }

        /*
        if let Some(exist_multi_file) = filters.exist_multi_file {
            query.push_str(" LEFT JOIN files vfd ON vm.id = vfd.media_id\n");
            query.push_str(" LEFT JOIN seasons vs ON vm.id = vs.media_id\n");
            query.push_str(" LEFT JOIN episodes ve ON vs.id = ve.season_id\n");
            query.push_str(" LEFT JOIN files vfe ON ve.id = vfe.episode_id\n");

            if exist_multi_file {
                where_conditions.push(
                    "(
                (SELECT COUNT(*) FROM files WHERE media_id = vm.id) > 1
                OR EXISTS (
                    SELECT 1 FROM episodes e
                    JOIN files f ON e.id = f.episode_id
                    WHERE e.media_id = vm.id
                    GROUP BY e.id
                    HAVING COUNT(f.id) > 1
                )
            )"
                    .to_string(),
                );
            } else {
                where_conditions.push(
                    "(
                (SELECT COUNT(*) FROM files WHERE media_id = vm.id) <= 1
                AND NOT EXISTS (
                    SELECT 1 FROM episodes e
                    JOIN files f ON e.id = f.episode_id
                    WHERE e.media_id = vm.id
                    GROUP BY e.id
                    HAVING COUNT(f.id) > 1
                )
            )"
                    .to_string(),
                );
            }
        }
        */

        if let Some(watched) = filters.watched {
            where_conditions.push("vm.watched = ?".to_string());
            params.push(Box::new(watched));
        }

        if let Some(watch_list) = filters.watch_list {
            where_conditions.push("vm.watch_list = ?".to_string());
            params.push(Box::new(watch_list));
        }

        if !filters.name.is_empty() {
            let search_pattern = format!("%{}%", filters.name);
            where_conditions.push("(vm.name LIKE ? OR im.title LIKE ?)".to_string());
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern));
        }

        if !where_conditions.is_empty() {
            query.push_str(&format!(" WHERE {}\n", where_conditions.join(" AND ")));
        }

        let sort_direction = &filters.sort_direction;

        let order_by = match filters.sort_by {
            SortByType::Name => format!("im.title {sort_direction}, vm.name {sort_direction}"),
            SortByType::Year => format!(
                "CAST(NULLIF(im.year, '') AS INTEGER) {sort_direction}, vm.year {sort_direction}, vm.imdb_id {sort_direction}, vm.name {sort_direction}"
            ),
            SortByType::Imdb => format!(
                "CAST(NULLIF(im.imdb_rating, '') AS REAL) {sort_direction}, im.title {sort_direction}, vm.name {sort_direction}"
            ),
        };

        query.push_str(&format!(" ORDER BY {order_by}"));

        let mut stmt = conn.prepare(&query)?;
        let results = stmt
            .query_map(
                rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
                |row| {
                    let media_id = row.get(0)?;
                    Self::get_media_and_imdb_by_media_id(conn, media_id)
                },
            )?
            .collect::<Result<_>>()?;

        Ok(results)
    }
}

// Updata
impl Sqlite {
    fn update_media_my_ranking(conn: &Connection, media_id: i64, my_ranking: u8) -> Result<usize> {
        conn.execute(
            "UPDATE medias SET my_ranking = ?1 WHERE id = ?2",
            [&my_ranking.to_string(), &media_id.to_string()],
        )
    }

    fn update_watch_list(conn: &Connection, media_id: i64, watch_list: bool) -> Result<()> {
        conn.execute(
            "UPDATE medias SET watch_list = ?1 WHERE id = ?2",
            params![watch_list, media_id],
        )?;

        Ok(())
    }

    fn update_media_watched(conn: &Connection, media_id: i64, watched: bool) -> Result<()> {
        conn.execute(
            "UPDATE medias SET watched = ?1 WHERE id = ?2",
            params![watched, media_id],
        )?;

        conn.execute(
            "UPDATE seasons SET watched = ?1 WHERE media_id = ?2",
            params![watched, media_id],
        )?;

        conn.execute(
            "UPDATE episodes SET watched = ?1 
         WHERE season_id IN (SELECT id FROM seasons WHERE media_id = ?2)",
            params![watched, media_id],
        )?;

        if watched {
            Self::update_watch_list(conn, media_id, false)?;
        }
        Ok(())
    }

    fn update_season_watched(conn: &Connection, season_id: i64, watched: bool) -> Result<()> {
        conn.execute(
            "UPDATE seasons SET watched = ?1 WHERE id = ?2",
            params![watched, season_id],
        )?;

        conn.execute(
            "UPDATE episodes SET watched = ?1 WHERE season_id = ?2",
            params![watched, season_id],
        )?;

        let media_id: i64 = conn.query_row(
            "SELECT media_id FROM seasons WHERE id = ?1",
            params![season_id],
            |row| row.get(0),
        )?;

        let media_all_watched: bool = conn.query_row(
            "SELECT COUNT(*) = SUM(CASE WHEN watched THEN 1 ELSE 0 END)
         FROM seasons WHERE media_id = ?1",
            params![media_id],
            |row| row.get(0),
        )?;

        conn.execute(
            "UPDATE medias SET watched = ?1 WHERE id = ?2",
            params![media_all_watched, media_id],
        )?;

        if media_all_watched {
            Self::update_watch_list(conn, media_id, false)?;
        }

        Ok(())
    }

    fn update_episode_watched(conn: &Connection, episode_id: i64, watched: bool) -> Result<()> {
        conn.execute(
            "UPDATE episodes SET watched = ?1 WHERE id = ?2",
            params![watched, episode_id],
        )?;

        let season_id: i64 = conn.query_row(
            "SELECT season_id FROM episodes WHERE id = ?1",
            params![episode_id],
            |row| row.get(0),
        )?;

        let all_watched: bool = conn.query_row(
            "SELECT COUNT(*) = SUM(CASE WHEN watched THEN 1 ELSE 0 END) 
         FROM episodes WHERE season_id = ?1",
            params![season_id],
            |row| row.get(0),
        )?;

        conn.execute(
            "UPDATE seasons SET watched = ?1 WHERE id = ?2",
            params![all_watched, season_id],
        )?;

        let media_id: i64 = conn.query_row(
            "SELECT media_id FROM seasons WHERE id = ?1",
            params![season_id],
            |row| row.get(0),
        )?;

        let media_all_watched: bool = conn.query_row(
            "SELECT COUNT(*) = SUM(CASE WHEN watched THEN 1 ELSE 0 END)
         FROM seasons WHERE media_id = ?1",
            params![media_id],
            |row| row.get(0),
        )?;

        conn.execute(
            "UPDATE medias SET watched = ?1 WHERE id = ?2",
            params![media_all_watched, media_id],
        )?;

        if media_all_watched {
            Self::update_watch_list(conn, media_id, false)?;
        }

        Ok(())
    }

    fn update_media_imdb(conn: &Connection, media_id: i64, imdb_id: &str) -> Result<()> {
        conn.execute(
            "UPDATE medias SET imdb_id = ? WHERE id = ?",
            [imdb_id, &media_id.to_string()],
        )?;
        Ok(())
    }
}

// remove
impl Sqlite {
    fn remove_empty_imdb(conn: &Connection) -> Result<()> {
        conn.execute(
            "
            DELETE FROM imdbs
            WHERE NOT EXISTS (
                SELECT 1 FROM medias WHERE imdb_id = imdbs.imdb_id
            )",
            [],
        )?;
        Ok(())
    }

    fn remove_empty_media(conn: &Connection) -> Result<()> {
        conn.execute(
            "
            DELETE FROM medias
            WHERE NOT EXISTS (
                SELECT 1 FROM files WHERE media_id = medias.id
            )
            AND NOT EXISTS (
                SELECT 1 FROM seasons WHERE media_id = medias.id
            )",
            [],
        )?;
        Ok(())
    }

    fn remove_empty_seasons(conn: &Connection) -> Result<()> {
        conn.execute(
            "
            DELETE FROM seasons
            WHERE NOT EXISTS (
                SELECT 1 FROM episodes WHERE season_id = seasons.id
            )",
            [],
        )?;
        Ok(())
    }

    fn remove_empty_episodes(conn: &Connection) -> Result<()> {
        conn.execute(
            "
            DELETE FROM episodes
            WHERE NOT EXISTS (
                SELECT 1 FROM files WHERE episode_id = episodes.id
            )",
            params![],
        )?;

        Ok(())
    }

    fn remove_file_by_path(conn: &Connection, path: &str) -> Result<usize> {
        conn.execute("DELETE FROM files WHERE path = ?", [path])
    }
}

#[cfg(test)]
impl Sqlite {
    fn new_tmp() -> Self {
        use rand::{Rng, rng};

        let charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

        let mut rng = rng();
        let chars: Vec<char> = charset.chars().collect();
        let length = 10;
        let mut random_string = String::with_capacity(length);

        for _ in 0..length {
            let random_index = rng.random_range(0..chars.len());
            random_string.push(chars[random_index]);
        }

        Self {
            path: format!("tmp_{random_string}.db").into(),
        }
    }

    fn remove(&self) {
        use std::fs;

        fs::remove_file(self.path.clone()).unwrap();
    }
}

impl From<&rusqlite::Row<'_>> for Imdb {
    fn from(row: &rusqlite::Row) -> Self {
        Self {
            title: row.get(0).unwrap_or_default(),
            year: row.get(1).unwrap_or_default(),
            rated: row.get(2).unwrap_or_default(),
            released: row.get(3).unwrap_or_default(),
            runtime: row.get(4).unwrap_or_default(),
            plot: row.get(5).unwrap_or_default(),
            awards: row.get(6).unwrap_or_default(),
            poster: row.get(7).unwrap_or_default(),
            imdb_rating: row.get(8).unwrap_or_default(),
            imdb_votes: row.get(9).unwrap_or_default(),
            box_office: row.get(10).unwrap_or_default(),
            total_seasons: row.get(11).unwrap_or_default(),
            r#type: row.get(12).unwrap_or_default(),
            imdb_id: row.get(13).unwrap_or_default(),
            genres: Vec::new(),
            directors: Vec::new(),
            writers: Vec::new(),
            actors: Vec::new(),
            languages: Vec::new(),
            countries: Vec::new(),
        }
    }
}

impl From<&rusqlite::Row<'_>> for MediaFile {
    fn from(row: &rusqlite::Row) -> Self {
        MediaFile {
            id: row.get(0).unwrap_or_default(),
            file_name: row.get(1).unwrap_or_default(),
            path: row.get(2).unwrap_or_default(),
            quality: row.get(3).unwrap_or_default(),
            language_format: row.get(4).unwrap_or_default(),
        }
    }
}

impl From<&rusqlite::Row<'_>> for Episode {
    fn from(row: &rusqlite::Row) -> Self {
        Episode {
            id: row.get(0).unwrap_or_default(),
            number: row.get(1).unwrap_or_default(),
            watched: row.get(2).unwrap_or_default(),
            files: Vec::new(),
        }
    }
}

impl From<&rusqlite::Row<'_>> for Season {
    fn from(row: &rusqlite::Row) -> Self {
        Self {
            id: row.get(0).unwrap_or_default(),
            number: row.get(1).unwrap_or_default(),
            watched: row.get(2).unwrap_or_default(),
            episodes: Vec::new(),
        }
    }
}

impl From<&rusqlite::Row<'_>> for Media {
    fn from(row: &rusqlite::Row) -> Self {
        Self {
            id: row.get(0).unwrap_or_default(),
            name: row.get(1).unwrap_or_default(),
            year: row.get(2).unwrap_or_default(),
            watched: row.get(3).unwrap_or_default(),
            my_ranking: row.get(4).unwrap_or_default(),
            watch_list: row.get(5).unwrap_or_default(),
            imdb: None,
            seasons: Vec::new(),
            files: Vec::new(),
        }
    }
}

impl DB for Sqlite {
    fn exist_file_by_path_from_db(&self, path: &Path) -> Result<bool> {
        let conn = self.get_conn()?;
        Ok(Self::get_file_by_path(&conn, path)?.is_some())
    }

    fn create_table(&self) -> Result<()> {
        let mut conn = self.get_conn()?;
        let tx = conn.transaction()?;

        let stmts = [
            r#"CREATE TABLE IF NOT EXISTS imdbs (
            imdb_id TEXT PRIMARY KEY,
            title TEXT,
            year TEXT,
            rated TEXT,
            released TEXT,
            runtime TEXT,
            plot TEXT,
            awards TEXT,
            poster TEXT,
            imdb_rating TEXT,
            imdb_votes TEXT,
            box_office TEXT,
            total_seasons TEXT,
            type TEXT
        );"#,
            r#"CREATE TABLE IF NOT EXISTS medias (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            year INTEGER,
            watched BOOLEAN DEFAULT 0,
            my_ranking INTEGER DEFAULT 0,
            watch_list BOOLEAN DEFAULT 0,
            imdb_id TEXT UNIQUE,
            UNIQUE (name, year)
            -- FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE
        );"#,
            r#"CREATE TABLE IF NOT EXISTS seasons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_id INTEGER NOT NULL,
            season_number INTEGER NOT NULL,
            watched BOOLEAN DEFAULT 0,
            FOREIGN KEY (media_id) REFERENCES medias (id) ON DELETE CASCADE
        );"#,
            r#"CREATE TABLE IF NOT EXISTS episodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            season_id INTEGER NOT NULL,
            episode_number INTEGER NOT NULL,
            watched BOOLEAN DEFAULT 0,
            FOREIGN KEY (season_id) REFERENCES seasons (id) ON DELETE CASCADE
        );"#,
            r#"CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_id INTEGER,
            episode_id INTEGER,
            file_name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            quality TEXT,
            language_format TEXT,
            CHECK (
                language_format IN ('soft_sub', 'hard_sub', 'dubbed', '')
            ),
            FOREIGN KEY (media_id) REFERENCES medias (id) ON DELETE CASCADE,
            FOREIGN KEY (episode_id) REFERENCES episodes (id) ON DELETE CASCADE,
            CHECK (
                media_id IS NOT NULL
                OR episode_id IS NOT NULL
            )
        );"#,
            r#"CREATE TABLE IF NOT EXISTS actors (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE
        );"#,
            r#"CREATE TABLE IF NOT EXISTS imdb_actors (
            imdb_id TEXT,
            actor_id INTEGER,
            PRIMARY KEY (imdb_id, actor_id)
            -- FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
            -- FOREIGN KEY (actor_id) REFERENCES actors (id) ON DELETE CASCADE
        );"#,
            r#"CREATE TABLE IF NOT EXISTS directors (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE
        );"#,
            r#"CREATE TABLE IF NOT EXISTS imdb_directors (
            imdb_id TEXT,
            director_id INTEGER,
            PRIMARY KEY (imdb_id, director_id)
            -- FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
            -- FOREIGN KEY (director_id) REFERENCES directors (id) ON DELETE CASCADE
        );"#,
            r#"CREATE TABLE IF NOT EXISTS writers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE
        );"#,
            r#"CREATE TABLE IF NOT EXISTS imdb_writers (
            imdb_id TEXT,
            writer_id INTEGER,
            PRIMARY KEY (imdb_id, writer_id)
            -- FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
            -- FOREIGN KEY (writer_id) REFERENCES writers (id) ON DELETE CASCADE
        );"#,
            r#"CREATE TABLE IF NOT EXISTS genres (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE
        );"#,
            r#"CREATE TABLE IF NOT EXISTS imdb_genres (
            imdb_id TEXT,
            genre_id INTEGER,
            PRIMARY KEY (imdb_id, genre_id)
            -- FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
            -- FOREIGN KEY (genre_id) REFERENCES genres (id) ON DELETE CASCADE
        );"#,
            r#"CREATE TABLE IF NOT EXISTS languages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE
        );"#,
            r#"CREATE TABLE IF NOT EXISTS imdb_languages (
            imdb_id TEXT,
            language_id INTEGER,
            PRIMARY KEY (imdb_id, language_id)
            -- FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
            -- FOREIGN KEY (language_id) REFERENCES languages (id) ON DELETE CASCADE
        );"#,
            r#"CREATE TABLE IF NOT EXISTS countries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE
        );"#,
            r#"CREATE TABLE IF NOT EXISTS imdb_countries (
            imdb_id TEXT,
            country_id INTEGER,
            PRIMARY KEY (imdb_id, country_id)
            -- FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
            -- FOREIGN KEY (country_id) REFERENCES countries (id) ON DELETE CASCADE
        );"#,
        ];

        for stmt in stmts {
            tx.execute(stmt, [])?;
        }

        tx.commit()?;

        Ok(())
    }

    fn insert_medias(&self, medias: &[Media]) -> Result<()> {
        let mut conn = self.get_conn()?;
        let tx = conn.transaction()?;

        for media in medias {
            Self::insert_media(&tx, media)?;
        }

        tx.commit()?;
        Ok(())
    }

    fn update_media_my_ranking_to_db(&self, media_id: i64, my_ranking: u8) -> Result<usize> {
        let conn = self.get_conn()?;
        Self::update_media_my_ranking(&conn, media_id, my_ranking)
    }

    fn update_media_watched_to_db(&self, media_id: i64, watched: bool) -> Result<()> {
        let conn = self.get_conn()?;
        Self::update_media_watched(&conn, media_id, watched)
    }

    fn update_season_watched_to_db(&self, season_id: i64, watched: bool) -> Result<()> {
        let conn = self.get_conn()?;
        Self::update_season_watched(&conn, season_id, watched)
    }

    fn update_episode_watched_to_db(&self, episode_id: i64, watched: bool) -> Result<()> {
        let conn = self.get_conn()?;
        Self::update_episode_watched(&conn, episode_id, watched)
    }

    fn update_media_imdb_to_db(&self, media_id: i64, imdb_id: &str) -> Result<()> {
        let conn = self.get_conn()?;
        Self::update_media_imdb(&conn, media_id, imdb_id)
    }

    fn insert_imdb_to_db(&self, imdb: &Imdb) -> Result<()> {
        let mut conn = self.get_conn()?;
        let tx = conn.transaction()?;
        Self::insert_imdb(&tx, imdb)?;
        tx.commit()
    }

    fn clear_empty_data_from_db(&self) -> Result<()> {
        let mut conn = self.get_conn()?;
        let tx = conn.transaction()?;

        Self::remove_empty_episodes(&tx)?;
        Self::remove_empty_seasons(&tx)?;
        Self::remove_empty_media(&tx)?;
        Self::remove_empty_imdb(&tx)?;

        tx.commit()
    }

    fn get_genres_from_db(&self) -> Result<Vec<(usize, String)>> {
        let conn = self.get_conn()?;

        Self::get_genres(&conn)
    }

    fn get_countries_from_db(&self) -> Result<Vec<(usize, String)>> {
        let conn = self.get_conn()?;
        Self::get_countries(&conn)
    }

    fn get_actors_from_db(&self) -> Result<Vec<(usize, String)>> {
        let conn = self.get_conn()?;
        Self::get_actors(&conn)
    }

    fn remove_file_by_path_from_db(&self, paths: &[PathBuf]) -> Result<()> {
        let mut conn = self.get_conn()?;
        let tx = conn.transaction()?;

        paths
            .iter()
            .filter_map(|path| path.to_str())
            .try_for_each(|path| Self::remove_file_by_path(&tx, path).and(Ok(())))?;

        tx.commit()
    }

    fn get_all_files_from_db(&self) -> Result<Vec<MediaFile>> {
        let conn = self.get_conn()?;
        Self::get_all_files(&conn)
    }

    fn filter_medias_on_db(&self, filters: &FilterValues) -> Result<Vec<Media>> {
        let mut conn = self.get_conn()?;
        let tx = conn.transaction()?;
        let re = Self::filter_medias(&tx, filters);
        tx.commit()?;
        re
    }

    fn get_media_by_id_from_db(&self, media_id: i64) -> Result<Option<Media>> {
        let mut conn = self.get_conn()?;
        let tx = conn.transaction()?;
        let re = Self::get_media_by_id(&tx, media_id);
        tx.commit()?;
        re
    }

    fn update_watch_list_to_db(&self, media_id: i64, watch_list: bool) -> Result<()> {
        let conn = self.get_conn()?;
        Self::update_watch_list(&conn, media_id, watch_list)
    }
}

#[cfg(test)]
mod tests_sqlit {
    use crate::metadata_extractor::get_metadata;

    use super::*;

    fn get_sqlite() -> Sqlite {
        let db = Sqlite::new_tmp();
        db.create_table().unwrap();
        db
    }

    #[test]
    fn test_create_table() {
        let db = get_sqlite();

        let conn = db.get_conn().unwrap();

        // Retrieve all table names
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            )
            .unwrap();
        let mut tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        tables.sort();

        let expected_tables = vec![
            "actors",
            "countries",
            "directors",
            "episodes",
            "files",
            "genres",
            "imdb_actors",
            "imdb_countries",
            "imdb_directors",
            "imdb_genres",
            "imdb_languages",
            "imdbs",
            "imdb_writers",
            "languages",
            "medias",
            "seasons",
            "writers",
        ];

        let mut expected = expected_tables
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        expected.sort();

        assert_eq!(tables, expected);

        db.remove();
    }

    #[test]
    fn insert_movie() {
        let db = get_sqlite();

        let m1 = Media::from(PathBuf::from(
            "/film/Who.Am.I.2014.720p.BluRay.HardSub.DigiMoviez.mp4",
        ));

        db.insert_medias(&[m1.clone()]).unwrap();

        assert_eq!(db.get_media_by_id_from_db(1).unwrap(), Some(m1));

        db.remove();
    }

    #[test]
    fn insert_series() {
        let m1 = get_metadata(&[
            "/marvel/loki/S1/Loki.S01E02.720p.WEB.DL.Dubbed.ZarFilm.mkv".into(),
            "/marvel/loki/S2/Loki.S02E02.720p.WEB.DL.Dubbed.ZarFilm.mkv".into(),
            "/marvel/loki/S2/Loki.S02E02.720p.WEB.DL.Dubbed.mkv".into(),
            "/marvel/loki/S1/Loki.S01E02.720p.WEB.DL.Dubbed.mkv".into(),
            "/marvel/loki/S1/Loki.S01E03.720p.WEB.DL.Dubbed.mkv".into(),
            "/marvel/loki/S2/Loki.S02E03.720p.WEB.DL.Dubbed.ZarFilm.mkv".into(),
            "/marvel/loki/S1/Loki.S01E03.720p.WEB.DL.Dubbed.ZarFilm.mkv".into(),
            "/marvel/loki/S2/Loki.S02E03.720p.WEB.DL.Dubbed.mkv".into(),
        ]);

        let db = get_sqlite();
        db.insert_medias(&m1).unwrap();

        assert_eq!(db.get_media_by_id_from_db(1).unwrap(), Some(m1[0].clone()));

        db.remove();
    }
}
