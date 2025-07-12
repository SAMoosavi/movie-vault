use crate::metadata_extractor::VideoMetaData;

use futures::future::join_all;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OmdbMovie {
    Title: String,
    Year: String,
    Rated: String,
    Released: String,
    Runtime: String,
    Genre: String,
    Director: String,
    Writer: String,
    Actors: String,
    Plot: String,
    Language: String,
    Country: String,
    Awards: String,
    Poster: String,
    Metascore: String,
    imdbRating: String,
    imdbVotes: String,
    imdbID: String,
    r#Type: String,
    BoxOffice: String,
    Response: String,
}

pub async fn get_omdb_metadata(
    videos_metadata: &[VideoMetaData],
    api_key: &str,
) -> Vec<Option<OmdbMovie>> {
    let client = Client::new();

    let futures = videos_metadata.iter().map(|meta| {
        let client = &client;
        let name = &meta.name;
        let year = meta.year.map(|y| y.to_string());

        let url = if let Some(year) = year {
            format!(
                "http://www.omdbapi.com/?apikey={}&t={}&y={}",
                api_key, name, year
            )
        } else {
            format!("http://www.omdbapi.com/?apikey={}&t={}", api_key, name)
        };

        async move {
            match client.get(&url).send().await {
                Ok(response) => match response.json::<OmdbMovie>().await {
                    Ok(data) => Some(data),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        }
    });

    join_all(futures).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata_extractor::VideoMetaData;

    #[tokio::test]
    async fn test_get_omdb_metadata() {
        // Mock server for OMDb
        let test_video = VideoMetaData {
            name: "3 Days To Kill".into(),
            subtitle_path: None,
            year: None,
            files_data: vec![],
            series: None,
        };

        // Override the API URL for testing
        let result = get_omdb_metadata(&[test_video], "4c602a26").await;

        assert_eq!(result.len(), 1);
        let movie = result[0].as_ref().unwrap();
        println!("{movie:?}");
        assert_eq!(movie.Title, "3 Days to Kill");
        assert_eq!(movie.Year, "2014");
        assert_eq!(movie.imdbID, "tt2172934");
    }
}
