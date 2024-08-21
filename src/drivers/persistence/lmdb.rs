use std::path::Path;
use lmdb::{Database, DatabaseFlags, Environment};
use crate::Result;

#[derive(Debug)]
pub struct LMDB(pub Environment, pub Database);

impl LMDB {
  #[tracing::instrument(err)]
  pub fn open(path: &Path) -> Result<Self> {
    let env = Environment::new().open(path)?;
    let db = env.create_db(None, DatabaseFlags::DUP_SORT)?;
    Ok(LMDB(env, db))
  }
}
