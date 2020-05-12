// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use thiserror::Error;

use std::io;

pub type Result<T> = std::result::Result<T, DSError>;

#[derive(Error, Debug)]
pub enum DSError {
    #[error("not found for key: {0}")]
    NotFound(String),

    #[error("db io error: {0}")]
    DBIoErr(#[from] io::Error),

    #[error("other err: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
