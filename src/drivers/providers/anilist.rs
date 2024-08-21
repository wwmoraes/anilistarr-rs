use serde::Deserialize;

use crate::{usecases::{Getter, Provider}, Result};

/// Fribbs provides a data source for IDs between multiple services and TVDB
#[derive(Debug)]
pub struct Fribbs(pub String);

impl Provider<super::Entry> for Fribbs {
  #[tracing::instrument(skip(self, client), ret, err)]
  fn fetch(&self, client: &dyn Getter) -> Result<Vec<super::Entry>> {
    let res = client.get(&self.0)?;
    let data = serde_json::from_slice::<Vec<AnilistData>>(&res[..])?;
    let entries: Vec<super::Entry> = data.into_iter().filter_map(|data| super::Entry::try_from(data).ok()).collect();

    Ok(entries)
  }
}

#[derive(Debug, Deserialize)]
struct AnilistData {
  anilist_id: Option<u64>,
  thetvdb_id: Option<u64>,
}

impl TryFrom<AnilistData> for super::Entry {
  type Error = crate::usecases::Errors;

  fn try_from(value: AnilistData) -> std::result::Result<Self, Self::Error> {
    let anilist_id = value.anilist_id.ok_or(crate::usecases::Errors::Unknown("no anilist id".to_owned()))?;
    let thetvdb_id = value.thetvdb_id.ok_or(crate::usecases::Errors::Unknown("no tvdb id".to_owned()))?;

    Ok(Self(anilist_id, thetvdb_id))
  }
}
