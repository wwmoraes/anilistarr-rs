use crate::{entities::SourceID, Result};

/**
  Tracker provides access to user metadata such as ID and media list from an
  upstream media tracking service
*/
pub trait Tracker: Sync + Send + std::fmt::Debug {
	fn get_user_id(&self, name: &str) -> Result<String>;
	fn get_media_list_ids(&self, user_id: &str) -> Result<Vec<SourceID>>;
}
