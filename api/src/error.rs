use thiserror::Error;
use thiserror_ext::{Box, Construct};

#[derive(Error, Debug, Box, Construct)]
#[thiserror_ext(newtype(name = Error))]
pub enum ErrorKind {
    #[error("error binding port: {0}")]
    BindPort(String, #[source] std::io::Error),
    #[error("io error")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
