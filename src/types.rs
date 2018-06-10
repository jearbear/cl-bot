use std::{error, result};

pub type Error = Box<error::Error>;
pub type Result<T> = result::Result<T, Error>;
