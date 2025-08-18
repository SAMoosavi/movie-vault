CREATE TABLE
    IF NOT EXISTS imdbs (
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
    );

CREATE TABLE
    IF NOT EXISTS medias (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        year INTEGER,
        watched BOOLEAN DEFAULT 0,
        my_ranking INTEGER DEFAULT 0,
        watch_list BOOLEAN DEFAULT 0,
        imdb_id TEXT UNIQUE,
        UNIQUE (name, year)
        FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS seasons (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        media_id INTEGER NOT NULL,
        season_number INTEGER NOT NULL,
        watched BOOLEAN DEFAULT 0,
        FOREIGN KEY (media_id) REFERENCES medias (id) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS episodes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        season_id INTEGER NOT NULL,
        episode_number INTEGER NOT NULL,
        watched BOOLEAN DEFAULT 0,
        FOREIGN KEY (season_id) REFERENCES seasons (id) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS files (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        media_id INTEGER,
        episode_id INTEGER,
        file_name TEXT NOT NULL,
        path TEXT NOT NULL UNIQUE,
        quality TEXT,
        language_format TEXT,
        FOREIGN KEY (media_id) REFERENCES medias (id) ON DELETE CASCADE,
        FOREIGN KEY (episode_id) REFERENCES episodes (id) ON DELETE CASCADE,
        CHECK (
            media_id IS NOT NULL
            OR episode_id IS NOT NULL
        )
    );

CREATE TABLE
    IF NOT EXISTS actors (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE
    );

CREATE TABLE
    IF NOT EXISTS imdb_actors (
        imdb_id TEXT,
        actor_id INTEGER,
        PRIMARY KEY (imdb_id, actor_id),
        FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
        FOREIGN KEY (actor_id) REFERENCES actors (id) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS directors (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE
    );

CREATE TABLE
    IF NOT EXISTS imdb_directors (
        imdb_id TEXT,
        director_id INTEGER,
        PRIMARY KEY (imdb_id, director_id),
        FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
        FOREIGN KEY (director_id) REFERENCES directors (id) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS writers (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE
    );

CREATE TABLE
    IF NOT EXISTS imdb_writers (
        imdb_id TEXT,
        writer_id INTEGER,
        PRIMARY KEY (imdb_id, writer_id),
        FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
        FOREIGN KEY (writer_id) REFERENCES writers (id) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS genres (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE
    );

CREATE TABLE
    IF NOT EXISTS imdb_genres (
        imdb_id TEXT,
        genre_id INTEGER,
        PRIMARY KEY (imdb_id, genre_id),
        FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
        FOREIGN KEY (genre_id) REFERENCES genres (id) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS languages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE
    );

CREATE TABLE
    IF NOT EXISTS imdb_languages (
        imdb_id TEXT,
        language_id INTEGER,
        PRIMARY KEY (imdb_id, language_id),
        FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
        FOREIGN KEY (language_id) REFERENCES languages (id) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS countries (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE
    );

CREATE TABLE
    IF NOT EXISTS imdb_countries (
        imdb_id TEXT,
        country_id INTEGER,
        PRIMARY KEY (imdb_id, country_id),
        FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
        FOREIGN KEY (country_id) REFERENCES countries (id) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS tags (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS media_tags (
        media_id INTEGER NOT NULL,
        tag_id INTEGER NOT NULL,
        PRIMARY KEY (media_id, tag_id),
        FOREIGN KEY (media_id) REFERENCES medias (id) ON DELETE CASCADE,
        FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE
    );