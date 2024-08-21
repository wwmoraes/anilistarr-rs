use std::collections::HashMap;

use crate::{entities::{SourceID, TargetID}, usecases::{Getter, Mapper}, Result};

/// Memory implements a volatile mapper.
#[derive(Default, Debug)]
pub struct Memory {
  /// mapping is a public HashMap used as the data source for IDs.
  pub mapping: HashMap<SourceID, TargetID>,
}

impl Mapper for Memory {
  #[tracing::instrument(skip(self), ret, err)]
  fn map_ids(&self, ids: &[SourceID]) -> Result<Vec<TargetID>> {
    Ok(ids.iter().filter_map(|id|
      self.mapping.get(id).map(|v| v.to_owned())
    ).collect::<Vec<TargetID>>())
  }

  #[tracing::instrument(skip(self), err)]
  fn refresh(&self, _client: &dyn Getter) -> Result {
    Ok(())
  }
}
