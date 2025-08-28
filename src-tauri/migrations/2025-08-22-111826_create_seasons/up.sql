CREATE TABLE IF NOT EXISTS seasons
(
    id            INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    media_id      INTEGER NOT NULL,
    season_number INTEGER NOT NULL,
    watched       BOOLEAN NOT NULL DEFAULT 0,
    UNIQUE (media_id, season_number),
    FOREIGN KEY (media_id) REFERENCES medias (id) ON DELETE CASCADE
);
