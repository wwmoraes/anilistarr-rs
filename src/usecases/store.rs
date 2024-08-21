use crate::{entities::Media, Result};

pub trait Store: std::fmt::Debug {
  fn get_media(&self, id: String) -> Result<Media>;
	fn get_media_bulk(&self, ids: Vec<String>) -> Result<Vec<Media>>;
	fn put_media(&self, media: Media) -> Result;
	fn put_media_bulk(&self, medias: Vec<Media>) -> Result;
}
