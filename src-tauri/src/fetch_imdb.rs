use anyhow::Result;

use crate::data_model::{Imdb, Media};

mod freeimdb;
mod imdbot;

pub async fn get_imdb_data_by_id(id: &str) -> Result<Imdb> {
    freeimdb::get_imdb_data_by_id(id).await
}

pub async fn set_imdb_data(medias: &mut [Media]) {
    imdbot::set_imdb_data(medias).await
}
