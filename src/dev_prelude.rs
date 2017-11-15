// the failure `Error` type
pub use prelude::*;
// TODO: add to prelude
pub use std::str::FromStr;


pub use std::result;
pub use failure::{Error, Fail};

pub type Result<V> = result::Result<V, Error>;
