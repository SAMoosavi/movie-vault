CREATE TABLE IF NOT EXISTS writers
(
    id   INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS imdb_writers
(
    imdb_id   TEXT NOT NULL,
    writer_id INTEGER NOT NULL,
    PRIMARY KEY (imdb_id, writer_id),
    FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
    FOREIGN KEY (writer_id) REFERENCES writers (id) ON DELETE CASCADE
);
