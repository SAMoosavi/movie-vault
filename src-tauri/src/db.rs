use std::path::PathBuf;

use rusqlite::Result;

use crate::metadata_extractor::VideoFileData;

#[cfg(test)]
mod moke;

#[cfg(test)]
pub use moke::MokeDB;

pub trait DB: Default + Send + Sync {
    fn get_video_file_by_path_from_db(&self, path: &PathBuf) -> Result<Option<VideoFileData>>;
}
