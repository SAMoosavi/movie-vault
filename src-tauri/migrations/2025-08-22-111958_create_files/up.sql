CREATE TABLE IF NOT EXISTS files
(
    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    media_id        INTEGER,
    episode_id      INTEGER,
    file_name       TEXT NOT NULL,
    path            TEXT NOT NULL UNIQUE,
    quality         TEXT,
    language_format TEXT NOT NULL,
    FOREIGN KEY (media_id) REFERENCES medias (id) ON DELETE CASCADE,
    FOREIGN KEY (episode_id) REFERENCES episodes (id) ON DELETE CASCADE,
    CHECK (media_id IS NOT NULL OR episode_id IS NOT NULL)
);
