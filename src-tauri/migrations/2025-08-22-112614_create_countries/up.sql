CREATE TABLE IF NOT EXISTS countries
(
    id   INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS imdb_countries
(
    imdb_id    TEXT NOT NULL,
    country_id INTEGER NOT NULL,
    PRIMARY KEY (imdb_id, country_id),
    FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
    FOREIGN KEY (country_id) REFERENCES countries (id) ON DELETE CASCADE
);
