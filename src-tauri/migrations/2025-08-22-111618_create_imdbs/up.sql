CREATE TABLE IF NOT EXISTS imdbs
(
    imdb_id       TEXT NOT NULL PRIMARY KEY,
    title         TEXT NOT NULL,
    year          INTEGER NOT NULL,
    rated         TEXT,
    runtime       TEXT,
    plot          TEXT,
    awards        TEXT,
    poster        TEXT,
    imdb_rating   TEXT,
    imdb_votes    INTEGER NOT NULL,
    box_office    TEXT,
    total_seasons TEXT,
    type          TEXT NOT NULL
);
