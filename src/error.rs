use thiserror::Error;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error(transparent)]
    PgError(#[from] tokio_postgres::error::Error),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("other error: {0}")]
    Other(String),
}
