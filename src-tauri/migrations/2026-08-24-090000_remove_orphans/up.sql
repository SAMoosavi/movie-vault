-- Remove rows left behind by deletes that ran without foreign key enforcement
DELETE FROM files WHERE media_id IS NOT NULL AND media_id NOT IN (SELECT id FROM medias);
DELETE FROM files WHERE episode_id IS NOT NULL AND episode_id NOT IN (SELECT id FROM episodes);
DELETE FROM episodes WHERE season_id NOT IN (SELECT id FROM seasons);
DELETE FROM seasons WHERE media_id NOT IN (SELECT id FROM medias);
DELETE FROM media_tags WHERE media_id NOT IN (SELECT id FROM medias);
DELETE FROM media_tags WHERE tag_id NOT IN (SELECT id FROM tags);
DELETE FROM imdb_people WHERE imdb_id NOT IN (SELECT imdb_id FROM imdbs);
DELETE FROM imdb_people WHERE person_id NOT IN (SELECT id FROM people);
DELETE FROM imdb_genres WHERE imdb_id NOT IN (SELECT imdb_id FROM imdbs);
DELETE FROM imdb_genres WHERE genre_id NOT IN (SELECT id FROM genres);
DELETE FROM imdb_countries WHERE imdb_id NOT IN (SELECT imdb_id FROM imdbs);
DELETE FROM imdb_countries WHERE country_id NOT IN (SELECT id FROM countries);
