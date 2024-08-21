// TODO change to optionals
pub trait Metadata: std::fmt::Debug {
  fn source_id(&self) -> String;
  fn target_id(&self) -> String;
}
