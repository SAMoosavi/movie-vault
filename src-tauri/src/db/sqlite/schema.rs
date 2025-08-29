// @generated automatically by Diesel CLI.

diesel::table! {
    actors (id) {
        id -> Integer,
        name -> Text,
        url -> Text,
    }
}

diesel::table! {
    countries (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    episodes (id) {
        id -> Integer,
        season_id -> Integer,
        episode_number -> Integer,
        watched -> Bool,
    }
}

diesel::table! {
    files (id) {
        id -> Integer,
        media_id -> Nullable<Integer>,
        episode_id -> Nullable<Integer>,
        file_name -> Text,
        path -> Text,
        quality -> Nullable<Text>,
        language_format -> Text,
    }
}

diesel::table! {
    genres (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    imdb_actors (imdb_id, actor_id) {
        imdb_id -> Text,
        actor_id -> Integer,
    }
}

diesel::table! {
    imdb_countries (imdb_id, country_id) {
        imdb_id -> Text,
        country_id -> Integer,
    }
}

diesel::table! {
    imdb_genres (imdb_id, genre_id) {
        imdb_id -> Text,
        genre_id -> Integer,
    }
}

diesel::table! {
    imdbs (imdb_id) {
        imdb_id -> Text,
        title -> Text,
        year -> Nullable<Text>,
        rated -> Nullable<Text>,
        released -> Nullable<Text>,
        runtime -> Nullable<Text>,
        plot -> Nullable<Text>,
        awards -> Nullable<Text>,
        poster -> Nullable<Text>,
        imdb_rating -> Nullable<Text>,
        imdb_votes -> Nullable<Text>,
        box_office -> Nullable<Text>,
        total_seasons -> Nullable<Text>,
        #[sql_name = "type"]
        type_ -> Text,
    }
}

diesel::table! {
    media_tags (media_id, tag_id) {
        media_id -> Integer,
        tag_id -> Integer,
    }
}

diesel::table! {
    medias (id) {
        id -> Integer,
        name -> Text,
        year -> Nullable<Integer>,
        watched -> Bool,
        my_ranking -> Integer,
        watch_list -> Bool,
        imdb_id -> Nullable<Text>,
    }
}

diesel::table! {
    seasons (id) {
        id -> Integer,
        media_id -> Integer,
        season_number -> Integer,
        watched -> Bool,
    }
}

diesel::table! {
    tags (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::joinable!(episodes -> seasons (season_id));
diesel::joinable!(files -> episodes (episode_id));
diesel::joinable!(files -> medias (media_id));
diesel::joinable!(imdb_actors -> actors (actor_id));
diesel::joinable!(imdb_actors -> imdbs (imdb_id));
diesel::joinable!(imdb_countries -> countries (country_id));
diesel::joinable!(imdb_countries -> imdbs (imdb_id));
diesel::joinable!(imdb_genres -> genres (genre_id));
diesel::joinable!(imdb_genres -> imdbs (imdb_id));
diesel::joinable!(media_tags -> medias (media_id));
diesel::joinable!(media_tags -> tags (tag_id));
diesel::joinable!(medias -> imdbs (imdb_id));
diesel::joinable!(seasons -> medias (media_id));

diesel::allow_tables_to_appear_in_same_query!(
    actors,
    countries,
    episodes,
    files,
    genres,
    imdb_actors,
    imdb_countries,
    imdb_genres,
    imdbs,
    media_tags,
    medias,
    seasons,
    tags,
);
