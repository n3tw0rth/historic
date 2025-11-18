use thiserror::Error;

#[derive(Error, Debug)]
pub enum HistoricError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Db error: {0}")]
    Db(#[from] turso::Error),

    #[error("unknown data store error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, HistoricError>;
