use thiserror::Error;

pub type Result<T> = ::std::result::Result<T, CborError>;

#[derive(Error, Debug)]
pub enum CborError {
    #[error("json error: {0}")]
    JsonErr(
        #[from]
        #[source]
        serde_json::Error,
    ),

    #[error("cid error: {0}")]
    CidErr(
        #[from]
        #[source]
        cid::Error,
    ),

    #[error("link should have been a string")]
    NotLink,
}
