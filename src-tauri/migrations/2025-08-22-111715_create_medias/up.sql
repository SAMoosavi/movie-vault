CREATE TABLE IF NOT EXISTS medias
(
    id         INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    year       INTEGER,
    watched    BOOLEAN NOT NULL DEFAULT 0,
    my_ranking INTEGER NOT NULL DEFAULT 0,
    watch_list BOOLEAN NOT NULL DEFAULT 0,
    imdb_id    TEXT UNIQUE,
    UNIQUE (name, year),
    FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE
);