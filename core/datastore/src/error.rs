// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, DSError>;

#[derive(Error, Debug)]
pub enum DSError {
    #[error("")]
    Other,
}
