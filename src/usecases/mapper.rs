use crate::entities::{
  SourceID,
  TargetID,
};

use crate::Result;

use super::Getter;

/// Mapper transforms IDs between two services based on the data from a provider
pub trait Mapper: Sync + Send + std::fmt::Debug {
	fn map_ids(&self, ids: &[SourceID]) -> Result<Vec<TargetID>>;
	fn refresh(&self, client: &dyn Getter) -> Result;
}
