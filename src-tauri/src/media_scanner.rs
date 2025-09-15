use futures::future::join_all;
use std::path::PathBuf;
use tokio::{fs, task};
use walkdir::WalkDir;

use crate::db::DB;

/// Supported video file extensions.
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi"];

/// Recursively scan a directory to find video
pub async fn find_movies<T: DB + 'static>(
    db: &T,
    root: PathBuf,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    if !root.exists() {
        return Err(format!("Directory does not exist: {}", root.display()).into());
    }

    let files = db.get_all_files()?;
    let videos = task::spawn_blocking(move || {
        WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .filter(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| VIDEO_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
                    .unwrap_or(false)
            })
            .filter(|path| files.iter().all(|x| x.path != path.to_string_lossy()))
            .collect::<Vec<_>>()
    })
    .await?;

    Ok(videos)
}

async fn find_non_existent_paths<T: DB>(
    db: &T,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let files = db.get_all_files()?.into_iter().map(|video| async move {
        let exists = fs::try_exists(&video.path).await.unwrap_or(false);
        if !exists { Some(video) } else { None }
    });

    let paths = join_all(files)
        .await
        .into_iter()
        .flatten()
        .map(|video| video.path.into())
        .collect();

    Ok(paths)
}

pub async fn sync_files<T: DB>(db: &T) -> Result<(), Box<dyn std::error::Error>> {
    let paths = find_non_existent_paths(db).await?;
    db.remove_file_by_path(&paths)?;
    // db.clear_empty_data()?;
    Ok(())
}

#[cfg(test)]
mod find_movies_tests {
    use super::*;
    use crate::db::MokeDB;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn valid_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let mut db = MokeDB::default();

        // Create some test files
        let files = vec![
            ("valid_movie.mp4", true),
            ("invalid_movie.txt", false),
            ("other_valid.mkv", true),
            ("db_existing.avi", false),
            ("no_extension", false),
        ];

        for (name, is_valid) in files {
            let path = temp_dir.path().join(name);
            if !is_valid {
                db.insert_file(path.clone());
            }
            File::create(path)
                .expect("Failed to create file")
                .write_all(b"test")
                .expect("Failed to write");
        }

        // Create a subdirectory with a file
        fs::create_dir(temp_dir.path().join("subdir")).expect("Failed to create subdir");
        File::create(temp_dir.path().join("subdir").join("subdir_movie.mp4"))
            .expect("Failed to create subdir file")
            .write_all(b"test")
            .expect("Failed to write");

        let root = temp_dir.path().to_path_buf();

        let videos = find_movies(&db, root).await.expect("Function failed");

        let video_paths: Vec<String> = videos
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();

        assert_eq!(videos.len(), 3, "Should find exactly 2 valid video files");
        assert!(video_paths.contains(&"valid_movie.mp4".to_string()));
        assert!(video_paths.contains(&"valid_movie.mp4".to_string()));
        assert!(video_paths.contains(&"subdir_movie.mp4".to_string()));
    }

    #[tokio::test]
    async fn no_valid_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        File::create(temp_dir.path().join("invalid.txt"))
            .expect("Failed to create file")
            .write_all(b"test")
            .expect("Failed to write");

        let db = MokeDB::default();
        let videos = find_movies(&db, temp_dir.path().to_path_buf())
            .await
            .expect("Function failed");

        assert_eq!(videos.len(), 0, "Should find no valid video files");
    }

    #[tokio::test]
    async fn non_existent_dir() {
        let db = MokeDB::default();
        let root = PathBuf::from("/non/existent/path");
        let videos = find_movies(&db, root).await;
        assert!(videos.is_err(), "Should fail for non-existent directory");
    }

    #[tokio::test]
    async fn empty_dir() {
        let db = MokeDB::default();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let videos = find_movies(&db, temp_dir.path().to_path_buf())
            .await
            .expect("Function failed");

        assert_eq!(videos.len(), 0, "Should find no files in empty directory");
    }

    #[tokio::test]
    async fn db_existing_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path_file = temp_dir.path().join("db_existing.avi");
        File::create(path_file.clone())
            .expect("Failed to create file")
            .write_all(b"test")
            .expect("Failed to write");

        let mut db = MokeDB::default();
        db.insert_file(path_file);
        let videos = find_movies(&db, temp_dir.path().to_path_buf())
            .await
            .expect("Function failed");

        assert_eq!(videos.len(), 0, "Should exclude files existing in DB");
    }
}
