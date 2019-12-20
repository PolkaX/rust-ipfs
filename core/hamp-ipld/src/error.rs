use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("other err: {0}")]
    Other(Box<dyn std::error::Error>),
}
