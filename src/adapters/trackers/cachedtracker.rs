use crate::{entities::SourceID, usecases::{Cache, Tracker}, Result};

const MEDIA_LIST_SEPARATOR: &str = "|";

#[derive(Debug)]
pub struct CachedTracker {
  cache: Box<dyn Cache>,
  tracker: Box<dyn Tracker>,
}

impl CachedTracker {
  pub fn new(cache: Box<dyn Cache>, tracker: Box<dyn Tracker>) -> Self {
    Self {
      cache,
      tracker,
    }
  }
}

impl Tracker for CachedTracker {
  #[tracing::instrument(skip(self), ret, err)]
  fn get_media_list_ids(&self, user_id: &str) -> Result<Vec<SourceID>> {
    let cache_key = format!("anilist:user:{}:medias", user_id);

    tracing::event!(tracing::Level::INFO, "try cache");

    match self.cache.get_string(cache_key.clone()) {
      Err(err) => tracing::error!(err),
      Ok(id) => {
        tracing::event!(tracing::Level::INFO, "cache hit");
        return Ok(id.split(MEDIA_LIST_SEPARATOR).map(|v|v.to_owned()).collect());
      },
    }

    tracing::event!(tracing::Level::INFO, "cache miss");


    let medias = self.tracker.get_media_list_ids(user_id)?;
    if let Err(err) = self.cache.set_string(cache_key, medias.clone().join(MEDIA_LIST_SEPARATOR), None) {
      tracing::error!(err);
    }

    Ok(medias)
  }

  #[tracing::instrument(skip(self), ret, err)]
  fn get_user_id(&self, name: &str) -> Result<String> {
    let cache_key = format!("anilist:user:{}:id", name);

    tracing::event!(tracing::Level::INFO, "try cache");

    match self.cache.get_string(cache_key.clone()) {
      Err(err) => tracing::error!(err),
      Ok(id) => {
        tracing::event!(tracing::Level::INFO, "cache hit");
        return Ok(id);
      },
    }

    tracing::event!(tracing::Level::INFO, "cache miss");

    let id = self.tracker.get_user_id(name)?;
    if let Err(err) = self.cache.set_string(cache_key, id.clone(), None) {
      tracing::error!(err);
    }

    Ok(id)
  }
}
