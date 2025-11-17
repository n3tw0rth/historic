use thiserror::Error;

#[derive(Error, Debug)]
pub enum HistoricError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unknown data store error")]
    Unknown,
}

pub type HResult<T> = Result<T, HistoricError>;
