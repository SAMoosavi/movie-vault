CREATE TABLE IF NOT EXISTS languages
(
    id   INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS imdb_languages
(
    imdb_id     TEXT NOT NULL,
    language_id INTEGER NOT NULL,
    PRIMARY KEY (imdb_id, language_id),
    FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
    FOREIGN KEY (language_id) REFERENCES languages (id) ON DELETE CASCADE
);