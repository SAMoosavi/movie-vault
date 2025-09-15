CREATE TABLE IF NOT EXISTS actors
(
    id   TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    url TEXT
);

CREATE TABLE IF NOT EXISTS imdb_actors
(
    imdb_id  TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    PRIMARY KEY (imdb_id, actor_id),
    FOREIGN KEY (imdb_id) REFERENCES imdbs (imdb_id) ON DELETE CASCADE,
    FOREIGN KEY (actor_id) REFERENCES actors (id) ON DELETE CASCADE
);