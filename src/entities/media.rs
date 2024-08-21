use serde::{Deserialize, Serialize};

use crate::usecases::Metadata;

pub type SourceID = String;
pub type TargetID = String;

#[derive(Debug, Deserialize, Serialize)]
pub struct Media {
  #[serde(default)]
  pub source_id: SourceID,
  #[serde(default)]
	pub target_id: TargetID,
}

impl<M: Metadata> From<M> for Media {
  fn from(value: M) -> Self {
    Media {
      source_id: value.source_id(),
      target_id: value.target_id(),
    }
  }
}
