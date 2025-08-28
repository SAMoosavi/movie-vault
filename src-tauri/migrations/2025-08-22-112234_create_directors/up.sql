CREATE TABLE IF NOT EXISTS directors
(
    id   INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS imdb_directors
(
    imdb_id     TEXT NOT NULL,
    director_id INTEGER NOT NULL,
    PRIMARY KEY (imdb_id, director_id),
    FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
    FOREIGN KEY (director_id) REFERENCES directors (id) ON DELETE CASCADE
);