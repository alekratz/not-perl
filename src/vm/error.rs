use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "unexpected {}", _0)]
    Runtime(String),
}

pub type Result<T> = std::result::Result<T, Error>;
