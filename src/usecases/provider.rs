use crate::Result;
use super::{Getter, Metadata};

pub trait Provider<M: Metadata>: std::fmt::Debug {
  fn fetch(&self, client: &dyn Getter) -> Result<Vec<M>>;
}
