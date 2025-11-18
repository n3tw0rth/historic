use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Db error: {0}")]
    Db(#[from] turso::Error),

    #[error("{msg:?}")]
    Unknown { msg: String },
}

pub type Result<T> = std::result::Result<T, Error>;
