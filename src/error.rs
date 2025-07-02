use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    SQLError(#[from] sqlx::Error),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
}
