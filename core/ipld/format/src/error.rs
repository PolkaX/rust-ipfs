use cid::Codec;
use thiserror::Error;

pub type Result<T> = ::std::result::Result<T, FormatError>;

#[derive(Error, PartialEq, Eq, Clone, Debug)]
pub enum FormatError {
    #[error("this obj can't return NodeStat")]
    NotSupportStat,

    #[error("decode block into node error")]
    DecodeError,

    #[error("this code has not register decoder: {0:?}")]
    DecoderNotRegister(Codec),

    #[error("depth is larger then path, depth: {0}, path len: {1}")]
    DepthError(usize, usize),

    #[error("depth is not init yet")]
    DepthNotInit,

    #[error("can't go down, the child does not exist, depth: {0}, index: {1}, child: {0}")]
    DownNoChild(usize, usize, usize),

    #[error("can't go up, already on root")]
    UpOnRoot,

    #[error("can't go to the next child, no more child nodes in this parent")]
    NextNoChild,

    #[error("child not exist for this index. index: {0}")]
    NoChild(usize),
}
