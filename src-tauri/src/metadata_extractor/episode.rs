use std::path::PathBuf;

use super::file::File;

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize)]
pub struct Episode {
    pub id: i64,
    pub number: i32,
    pub watched: bool,
    pub files: Vec<File>,
}

impl Episode {
    pub fn new(path: PathBuf, number: i32) -> Self {
        Self {
            id: 0,
            number,
            watched: false,
            files: vec![File::from(path)],
        }
    }
}

#[cfg(test)]
mod tests_episode {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn new() {
        let test_path = PathBuf::from("/path/to/Show.Name.S02E03.MKV");
        let test_number = 42;

        let episode = Episode::new(test_path.clone(), test_number);

        assert_eq!(episode.id, 0, "ID should be initialized to 0");
        assert_eq!(episode.number, test_number, "Number should match input");
        assert_eq!(
            episode.watched, false,
            "Watched should be initialized to false"
        );
        assert_eq!(
            episode.files.len(),
            1,
            "Files vector should contain exactly one file"
        );
    }
}
