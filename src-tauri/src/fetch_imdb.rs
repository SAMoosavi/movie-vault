use anyhow::Result;
use serde::Serialize;

use crate::data_model::{Imdb, Media};

mod freeimdb;
mod imdbot;

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImdbSyncStats {
    pub requested: usize,
    pub enriched: usize,
    pub id_lookup_failed: usize,
    pub batch_missing: usize,
    pub batch_fetch_failed: usize,
}

impl ImdbSyncStats {
    pub fn failed_total(self) -> usize {
        self.id_lookup_failed + self.batch_missing + self.batch_fetch_failed
    }
}

pub async fn get_imdb_data_by_id(id: &str) -> Result<Imdb> {
    freeimdb::get_imdb_data_by_id(id).await
}

pub async fn set_imdb_data(medias: &mut [Media]) -> Result<ImdbSyncStats> {
    imdbot::set_imdb_data(medias).await
}
