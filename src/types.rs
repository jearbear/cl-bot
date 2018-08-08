use std::result;

use failure;
pub use failure::ResultExt;

pub type Result<T> = result::Result<T, failure::Error>;
