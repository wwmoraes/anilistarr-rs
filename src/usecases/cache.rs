use std::time::Duration;

use crate::Result;

pub trait Cache: Sync + Send + std::fmt::Debug {
  fn get_string(&self, key: String) -> Result<String>;
  fn set_string(&self, key: String, value: String, options: Option<CacheOptions>) -> Result;
}

#[derive(Default, Debug)]
pub struct CacheOptions {
  pub ttl: Duration,
}
