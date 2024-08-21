use crate::{entities::{Media, SourceID, TargetID}, usecases::{Getter, Mapper, Metadata, Provider, Store}, Result};

#[derive(Debug)]
pub struct Persistent<M: Metadata> {
  provider: Box<dyn Provider<M> + Send + Sync>,
  store: Box<dyn Store + Send + Sync>,
}

impl<M: Metadata> Persistent<M> {
  pub fn new(provider: Box<dyn Provider<M> + Send + Sync>, store: Box<dyn Store + Send + Sync>) -> Self {
    Persistent {
      provider,
      store,
    }
  }
}

impl<M: Metadata> Mapper for Persistent<M> {
  #[tracing::instrument(skip(self), ret, err)]
  fn map_ids(&self, ids: &[SourceID]) -> Result<Vec<TargetID>> {
    let medias = self.store.get_media_bulk(ids.into())?;
    let res = medias.into_iter().map(|m| m.target_id.to_owned()).collect::<Vec<TargetID>>();
    Ok(res)
  }

  #[tracing::instrument(skip(self, client), err)]
  fn refresh(&self, client: &dyn Getter) -> Result {
    let data = self.provider.fetch(client)?;
    let medias = data.into_iter().map(|m|Media::from(m)).collect();
    self.store.put_media_bulk(medias)?;
    Ok(())
  }
}
