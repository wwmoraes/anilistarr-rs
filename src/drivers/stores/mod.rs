use lmdb::{Transaction as _, WriteFlags};

use crate::{entities::Media, usecases::Store, Result};
use crate::drivers::persistence::LMDB;

impl Store for LMDB {
  #[tracing::instrument(skip(self), ret, err)]
  fn get_media(&self, id: String) -> Result<Media> {
    let tx = self.0.begin_ro_txn()?;
    let value = tx.get(self.1, &id)?;
    let media: Media = serde_json::from_slice(value)?;
    Ok(media)
  }

  #[tracing::instrument(skip(self), ret, err)]
  fn get_media_bulk(&self, ids: Vec<String>) -> Result<Vec<Media>> {
    let tx = self.0.begin_ro_txn()?;
    let res: Result<Vec<Media>> = ids.into_iter().filter_map(|source_id| {
      match tx.get(self.1, &source_id) {
        Err(lmdb::Error::NotFound) => None,
        Err(err) => Some(Err(err.into())),
        Ok(data) => Some(Ok(Media {
          source_id,
          target_id: String::from_utf8_lossy(data).to_string(),
        })),
      }
    }).collect();

    res
  }

  #[tracing::instrument(skip(self), err)]
  fn put_media(&self, media: Media) -> Result {
    let mut tx = self.0.begin_rw_txn()?;
    match tx.put(self.1, &media.source_id, &media.target_id, WriteFlags::NO_DUP_DATA) {
      Ok(()) => Ok(()),
      Err(lmdb::Error::KeyExist) => Ok(()),
      Err(err) => Err(err),
    }?;
    tx.commit()?;
    Ok(())
  }

  #[tracing::instrument(skip(self), err)]
  fn put_media_bulk(&self, medias: Vec<Media>) -> Result {
    let mut tx = self.0.begin_rw_txn()?;
    for media in medias {
      match tx.put(self.1, &media.source_id, &media.target_id, WriteFlags::NO_DUP_DATA) {
        Ok(()) => Ok(()),
        Err(lmdb::Error::KeyExist) => Ok(()),
        Err(err) => Err(err),
      }?;
    }
    tx.commit()?;
    Ok(())
  }
}
