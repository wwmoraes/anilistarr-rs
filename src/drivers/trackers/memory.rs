use crate::{entities::SourceID, usecases::Errors};

/// Memory implements a volatile tracker.
#[derive(Debug, Default)]
pub struct Memory {
  pub user_ids: std::collections::HashMap<String, u32>,
  pub media_lists: std::collections::HashMap<u32, Vec<SourceID>>,
}

impl crate::usecases::Tracker for Memory {
  #[tracing::instrument(skip(self), ret, err)]
  fn get_media_list_ids(&self, user_id: &str) -> crate::Result<Vec<SourceID>> {
    let parsed_user_id = user_id.parse::<u32>()
      .map_err(|e|Errors::Unknown(e.to_string()))?;
    let media_list = self.media_lists.get(&parsed_user_id)
      .ok_or(Errors::UserNoMedia(user_id.to_owned()))?;

    Ok(media_list.to_owned())
  }

  #[tracing::instrument(skip(self), ret, err)]
  fn get_user_id(&self, name: &str) -> crate::Result<String> {
    let user_id = self.user_ids.get(name)
      .ok_or(Errors::UserNotFound(name.to_owned()))?;

    Ok(user_id.to_string())
  }
}
