use crate::{entities::{CustomEntry, CustomList}, Result};

use super::{
  Getter, Mapper, Tracker
};

pub trait MediaLister: std::fmt::Debug {
  /// generate fetches the user media list from the Tracker and transform the IDs
	/// found to the target service through the Mapper
  fn generate(&self, name: &str) -> Result<CustomList>;

  /// get_user_id searches the Tracker for the user ID by their name/handle
	fn get_user_id(&self, name: &str) -> Result<String>;

  /// refresh requests the Mapper to update its mapping definitions
	fn refresh(&self, client: &dyn Getter) -> Result;
}

#[derive(Debug)]
pub struct TrackerMediaLister {
  tracker: Box<dyn Tracker>,
  mapper: Box<dyn Mapper>,
}

impl TrackerMediaLister {
  pub fn new(tracker: Box<dyn Tracker>, mapper: Box<dyn Mapper>) -> Self {
    Self {
      tracker,
      mapper,
    }
  }
}

impl MediaLister for TrackerMediaLister {
  #[tracing::instrument(skip(self), ret, err)]
  fn generate(&self, name: &str) -> Result<CustomList> {
    let user_id = self.tracker.get_user_id(name)?;
    let source_ids = self.tracker.get_media_list_ids(user_id.as_str())?;
    let target_ids = self.mapper.map_ids(&source_ids)?;

    Ok(target_ids.into_iter().filter_map(|id|
      id.parse().map( |v| CustomEntry {
        tvdb_id: v,
      }).ok()
    ).collect())
  }

  #[tracing::instrument(skip(self), ret, err)]
  fn get_user_id(&self, name: &str) -> Result<String> {
    self.tracker.get_user_id(name)
  }

  #[tracing::instrument(skip(self), err)]
  fn refresh(&self, client: &dyn Getter) -> Result {
    self.mapper.refresh(client)
  }
}
