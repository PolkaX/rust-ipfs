use thiserror::Error;

use rust_cid::{Cid, Error as CidErr};

#[derive(Error, Debug)]
pub enum BlockFormatError {
    #[error("data did not match given hash, fst: {0}, snd: {1}")]
    WrongHash(Cid, Cid),
    #[error("Cid Error {0}")]
    CidError(CidErr),
}
