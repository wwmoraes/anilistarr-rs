use lmdb::{self, Transaction, WriteFlags};

use crate::{drivers::persistence::LMDB, usecases::Cache, Result};

impl Cache for LMDB {
  #[tracing::instrument(skip(self), ret, err)]
  fn get_string(&self, key: String) -> Result<String> {
    let tx = self.0.begin_ro_txn()?;
    let value = tx.get(self.1, &key)?;

    Ok(String::from_utf8_lossy(value).into_owned())
  }

  #[tracing::instrument(skip(self), err)]
  fn set_string(&self, key: String, value: String, _options: Option<crate::usecases::CacheOptions>) -> Result {
    let mut tx = self.0.begin_rw_txn()?;
    match tx.put(self.1, &key, &value, WriteFlags::NO_DUP_DATA) {
      Ok(()) => Ok(()),
      Err(lmdb::Error::KeyExist) => Ok(()),
      Err(err) => Err(err),
    }?;
    tx.commit()?;

    Ok(())
  }
}
