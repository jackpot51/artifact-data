// the failure `Error` type
pub use std::result;
pub use failure::{Error, Fail};

pub type Result<V> = result::Result<V, Error>;
