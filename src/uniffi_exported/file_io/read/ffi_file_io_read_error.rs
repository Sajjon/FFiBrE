use crate::prelude::*;
use thiserror::Error as ThisError;

#[derive(Debug, PartialEq, Eq, Clone, Error, ThisError)]
pub enum FFIFileIOReadError {
    #[error("UnknownError: '{underlying}'")]
    Unknown { underlying: String },
}
