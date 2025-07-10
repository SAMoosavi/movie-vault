use either::Either;
use rayon::prelude::*;
use std::path::PathBuf;
use tokio::task;
use walkdir::WalkDir;

/// Supported video file extensions.
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi"];

/// Holds separated lists of video and subtitle file paths.
#[derive(Debug)]
pub struct FoundFiles {
    pub videos: Vec<PathBuf>,
    pub subtitles: Vec<PathBuf>,
}

/// Recursively scan a directory to find video and subtitle files.
pub async fn find_movies(root: PathBuf) -> FoundFiles {
    // Scan the file system in a blocking thread
    let all_files = task::spawn_blocking(move || {
        WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .collect::<Vec<_>>()
    })
    .await
    .expect("Failed to scan file system");

    // Classify files as videos or subtitles
    let (videos, subtitles): (Vec<_>, Vec<_>) = all_files
        .into_par_iter()
        .filter(|path| path.extension().is_some())
        .partition_map(|path| match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) => {
                let ext = ext.to_ascii_lowercase();
                if VIDEO_EXTENSIONS.contains(&ext.as_str()) {
                    Either::Left(path)
                } else if ext == "srt" {
                    Either::Right(path)
                } else {
                    Either::Right(PathBuf::new())
                }
            }
            None => Either::Right(PathBuf::new()),
        });

    // Filter out placeholder empty paths from ignored extensions
    let subtitles = subtitles
        .into_iter()
        .filter(|p| !p.as_os_str().is_empty())
        .collect();

    FoundFiles { videos, subtitles }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_find_movies() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Create test files
        let video1 = root.join("movie1.mkv");
        let video2 = root.join("movie2.mp4");
        let subtitle1 = root.join("movie1.srt");
        let ignore_file = root.join("notes.txt");

        File::create(&video1).unwrap();
        File::create(&video2).unwrap();
        File::create(&subtitle1).unwrap();
        File::create(&ignore_file).unwrap();

        let result = find_movies(root.to_path_buf()).await;

        assert_eq!(result.videos.len(), 2, "Should detect two video files");
        assert_eq!(result.subtitles.len(), 1, "Should detect one subtitle file");

        assert!(result.videos.contains(&video1));
        assert!(result.videos.contains(&video2));
        assert!(result.subtitles.contains(&subtitle1));
        assert!(!result.videos.contains(&ignore_file));
    }
}
