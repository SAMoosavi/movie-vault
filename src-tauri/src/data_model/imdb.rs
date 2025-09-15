#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Imdb {
    pub title: String,
    pub year: String,
    pub released: String,
    pub genres: Vec<String>,
    pub actors: Vec<Person>,
    pub plot: String,
    pub countries: Vec<String>,
    pub poster: String,
    pub imdb_rating: String,
    pub imdb_votes: String,
    pub imdb_id: String,
    pub r#type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub url: String,
}
