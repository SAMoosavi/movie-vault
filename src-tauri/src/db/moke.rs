use std::path::PathBuf;

use crate::{db::DB, metadata_extractor};
use rusqlite::Result;

#[derive(Default)]
pub struct MokeDB {}

impl DB for MokeDB {
    fn get_video_file_by_path_from_db(
        &self,
        path: &PathBuf,
    ) -> Result<Option<metadata_extractor::VideoFileData>> {
        if path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .contains("valid")
        {
            Ok(None)
        } else {
            Ok(Some(metadata_extractor::VideoFileData::default()))
        }
    }
}
