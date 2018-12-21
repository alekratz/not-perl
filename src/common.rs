use std::{
    path::Path,
    str::FromStr,
};
use crate::util;

pub mod lang;
pub mod strings;
pub mod value;
#[macro_use]
pub mod pos;
pub mod error;

use self::error::Error;

pub mod prelude {
    pub use super::lang::*;
    pub use super::pos::*;
    pub use super::FromPath;
}

pub trait FromPath: Sized {
    type Err;

    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Self::Err>;
}

impl<T: FromStr<Err=Error>> FromPath for T {
    type Err = Error;
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Self::Err> {
        let contents = util::read_file(path)?;
        Ok(Self::from_str(&contents)?)
    }
}
