pub mod anilist;

#[derive(Debug)]
pub struct Entry(u64, u64);

impl crate::usecases::Metadata for Entry {
  fn source_id(&self) -> String {
    self.0.to_string()
  }

  fn target_id(&self) -> String {
    self.1.to_string()
  }
}
