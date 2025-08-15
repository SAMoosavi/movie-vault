use std::path::PathBuf;

use super::media_file::MediaFile;
use itertools::Itertools;

#[derive(Debug, Clone, Eq, Ord, serde::Serialize)]
pub struct Episode {
    pub id: i64,
    pub number: i32,
    pub watched: bool,
    pub files: Vec<MediaFile>,
}

impl PartialOrd for Episode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.number.partial_cmp(&other.number) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.watched.partial_cmp(&other.watched) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.files.partial_cmp(&other.files)
    }
}

impl PartialEq for Episode {
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
            && self.watched == other.watched
            && self.files.iter().sorted().eq(other.files.iter().sorted())
    }
}

impl Episode {
    pub fn new(path: PathBuf, number: i32) -> Self {
        Self {
            id: 0,
            number,
            watched: false,
            files: vec![MediaFile::from(path)],
        }
    }

    pub fn merge(&mut self, other: &Self) {
        self.files.extend(other.files.iter().cloned());
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

    #[test]
    fn merge_adds_files() {
        let file1 = MediaFile::generate_random_file(1);
        let file2 = MediaFile::generate_random_file(2);
        let file3 = MediaFile::generate_random_file(3);
        let file4 = MediaFile::generate_random_file(4);

        let mut episode1 = Episode {
            id: 0,
            number: 0,
            watched: false,
            files: vec![file1.clone(), file2.clone()],
        };

        let episode2 = Episode {
            id: 0,
            number: 0,
            watched: false,
            files: vec![file3.clone(), file4.clone()],
        };

        episode1.merge(&episode2);

        assert_eq!(episode1.files.len(), 4);
        assert_eq!(episode1.files[0], file1);
        assert_eq!(episode1.files[1], file2);
        assert_eq!(episode1.files[2], file3);
        assert_eq!(episode1.files[3], file4);
    }

    #[test]
    fn test_episode_merge_with_empty_other() {
        let file1 = MediaFile::generate_random_file(1);

        let mut episode = Episode {
            id: 0,
            number: 0,
            watched: false,
            files: vec![file1.clone()],
        };

        let empty_other = Episode {
            id: 0,
            number: 0,
            watched: false,
            files: vec![],
        };

        episode.merge(&empty_other);

        assert_eq!(episode.files.len(), 1);
        assert_eq!(episode.files[0], file1);
    }

    #[test]
    fn test_episode_merge_into_empty() {
        let file1 = MediaFile::generate_random_file(1);
        let mut empty_episode = Episode {
            id: 0,
            number: 0,
            watched: false,
            files: vec![],
        };

        let other = Episode {
            id: 0,
            number: 0,
            watched: false,
            files: vec![file1.clone()],
        };

        empty_episode.merge(&other);

        assert_eq!(empty_episode.files.len(), 1);
        assert_eq!(empty_episode.files[0], file1);
    }
}
