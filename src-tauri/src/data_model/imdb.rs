#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct Imdb {
    pub imdb_id: String,
    pub r#type: String,
    pub title: String,
    pub year: i32,
    pub plot: String,
    pub genres: Vec<String>,
    pub countries: Vec<String>,
    pub poster: String,
    pub imdb_rating: String,
    pub imdb_votes: i32,
    pub actors: Vec<Person>,
    pub writers: Vec<Person>,
    pub directors: Vec<Person>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub url: String,
}
