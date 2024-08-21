pub mod handlers;

pub type State = std::sync::Arc<dyn crate::usecases::MediaLister + Send + Sync>;

#[macro_export]
macro_rules! state {
  ($value:expr) => {
    std::sync::Arc::new($value)
  };
}

pub use state;
