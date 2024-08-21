use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errors {
  // #[error("data store disconnected")]
  // Disconnect(#[from] io::Error),
  #[error("user `{0}` not found")]
  UserNotFound(String),
  #[error("user `{0}` has no media")]
  UserNoMedia(String),
  #[error("unknown error: `{0}`")]
  Unknown(String),
}
