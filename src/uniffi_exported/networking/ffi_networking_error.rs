use crate::prelude::*;
use thiserror::Error as ThisError;

#[derive(Debug, PartialEq, Eq, Clone, Error, ThisError)]
pub enum FFINetworkingError {
    #[error("Fail to create Swift 'Foundation.URL' from string: '{string}'")]
    FailedToCreateURLFrom { string: String },

    #[error(
        "Swift 'URLRequest' failed with code '{:?}', error message from Gateway: '{:?}', underlying error (URLSession): '{:?}'",
        status_code,
        error_message_from_gateway,
        url_session_underlying_error
    )]
    RequestFailed {
        status_code: Option<u16>,
        url_session_underlying_error: Option<String>,
        error_message_from_gateway: Option<String>,
    },
}
