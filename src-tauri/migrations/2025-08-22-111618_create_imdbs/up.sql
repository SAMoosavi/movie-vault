CREATE TABLE IF NOT EXISTS imdbs
(
    imdb_id       TEXT NOT NULL PRIMARY KEY,
    title         TEXT NOT NULL,
    year          TEXT,
    rated         TEXT,
    released      TEXT,
    runtime       TEXT,
    plot          TEXT,
    awards        TEXT,
    poster        TEXT,
    imdb_rating   TEXT,
    imdb_votes    TEXT,
    box_office    TEXT,
    total_seasons TEXT,
    type          TEXT NOT NULL
);
