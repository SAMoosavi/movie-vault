CREATE TABLE IF NOT EXISTS people
(
    id      TEXT NOT NULL PRIMARY KEY,
    name    TEXT NOT NULL UNIQUE,
    url     TEXT
);

CREATE TABLE IF NOT EXISTS imdb_people
(
    imdb_id     TEXT NOT NULL,
    person_id   TEXT NOT NULL,
    person_type TEXT NOT NULL,
    PRIMARY KEY (imdb_id, person_id),
    FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
    FOREIGN KEY (person_id) REFERENCES people (id) ON DELETE CASCADE
);