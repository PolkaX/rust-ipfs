use thiserror::Error;

pub type Result<T> = ::std::result::Result<T, FormatError>;

#[derive(Error, PartialEq, Eq, Clone, Debug)]
pub enum FormatError {
    #[error("this obj can't return NodeStat")]
    NotSupportStat,
}
