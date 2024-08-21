use std::{error::Error, fmt::{Debug, Display, Formatter}, future::Future};

pub mod adapters;
pub mod drivers;
pub mod entities;
pub mod usecases;

pub type Result<T = (), E = Box<dyn Error>> = std::result::Result<T, E>;

pub fn resolve<T>(fut: impl Future<Output=T>) -> T {
  tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(fut)
    // futures::executor::block_on(fut)
  })
}

#[derive(Default)]
pub struct NotImplementedError(&'static str);

impl Error for NotImplementedError {}

impl Display for NotImplementedError {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "{}: not implemented", self.0)
  }
}

impl Debug for NotImplementedError {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "{}: not implemented", self.0)
  }
}
