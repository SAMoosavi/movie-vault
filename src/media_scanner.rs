use either::Either;
use std::path::PathBuf;
use tokio::task;
use walkdir::WalkDir;
use rayon::prelude::*;

#[derive(Debug)]
pub struct FoundFiles {
    pub videos: Vec<PathBuf>,
    pub subtitles: Vec<PathBuf>,
}

pub async fn find_movies(root: PathBuf) -> FoundFiles {
    // Collect file paths in blocking task
    let entries = task::spawn_blocking(move || {
        WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .collect::<Vec<_>>()
    })
    .await
    .expect("Blocking task failed");

    let (videos, subtitles): (Vec<_>, Vec<_>) = entries.into_par_iter().partition_map(|path| {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) if matches!(ext, "mp4" | "mkv" | "avi") => Either::Left(path),
            Some("srt") => Either::Right(path),
            _ => Either::Right(PathBuf::new()),
        }
    });

    // Filter out dummy values
    let subtitles = subtitles
        .into_iter()
        .filter(|p| !p.as_os_str().is_empty())
        .collect();

    FoundFiles { videos, subtitles }
}
