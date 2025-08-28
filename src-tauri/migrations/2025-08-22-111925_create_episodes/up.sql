CREATE TABLE IF NOT EXISTS episodes
(
    id             INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    season_id      INTEGER NOT NULL,
    episode_number INTEGER NOT NULL,
    watched        BOOLEAN NOT NULL DEFAULT 0,
    UNIQUE (season_id, episode_number),
    FOREIGN KEY (season_id) REFERENCES seasons (id) ON DELETE CASCADE
);
