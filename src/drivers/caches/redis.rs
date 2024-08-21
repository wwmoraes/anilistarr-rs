use std::time::SystemTime;

use redis::{Client, Commands, SetExpiry, SetOptions};

use crate::{usecases::{Cache, CacheOptions, Errors}, Result};

impl Cache for Client {
  #[tracing::instrument(skip(self), ret, err)]
  fn get_string(&self, key: String) -> Result<String> {
    let mut con = self.get_connection()?;
    let value: Option<String> = con.get(key)?;

    value.ok_or(Errors::Unknown("value not cached".to_owned()).into())
  }

  #[tracing::instrument(skip(self), err)]
  fn set_string(&self, key: String, value: String, options: Option<CacheOptions>) -> Result {
    let mut con = self.get_connection()?;
    let res: () = match options {
      Some(options) => con.set_options(key, value, options.into()),
      None => con.set(key, value),
    }?;

    Ok(res)
  }
}

impl From<CacheOptions> for SetOptions {
  fn from(value: CacheOptions) -> Self {
    let set_options = Self::default();

    SystemTime::now()
      .checked_add(value.ttl)
      .and_then(|time| time.duration_since(
        SystemTime::UNIX_EPOCH).ok()
      )
      .map(|time| set_options.with_expiration(
        SetExpiry::PXAT(time.as_millis().max(u64::MAX as u128) as u64)
      ));

    set_options
  }
}
